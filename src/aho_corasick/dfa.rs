use super::automaton::Automaton;
use super::nfa::{NFA, START_ID};
use super::state_id::{DEAD_ID, FAIL_ID};
use super::Match;

pub const ALPHABET_LEN: usize = 256;

#[derive(Clone)]
pub struct DFA {
    premultiplied: bool,
    start_id: usize,
    state_count: usize,
    max_match: usize,
    trans: Vec<usize>,
    matches: Vec<Vec<(usize, usize)>>,
}

impl Automaton for DFA {
    fn start_state(&self) -> usize {
        self.start_id
    }

    fn is_valid(&self, id: usize) -> bool {
        (id / 256) < self.state_count
    }

    fn is_match_state(&self, id: usize) -> bool {
        self.is_match_state(id)
    }

    fn is_match_or_dead_state(&self, id: usize) -> bool {
        self.is_match_or_dead_state(id)
    }

    fn get_match(&self, id: usize, match_index: usize, end: usize) -> Option<Match> {
        if id > self.max_match {
            return None;
        }
        self.matches.get(id / 256).and_then(|m| m.get(match_index)).map(|&(id, len)| Match {
            pattern: id,
            len,
            end,
        })
    }

    fn match_count(&self, id: usize) -> usize {
        let o = id / 256;
        self.matches[o].len()
    }

    fn next_state(&self, current: usize, input: u8) -> usize {
        let o = current + input as usize;
        self.trans[o]
    }
}

impl DFA {
    pub fn new(nfa: &NFA) -> Self {
        let trans = vec![FAIL_ID; ALPHABET_LEN * nfa.state_len()];
        let matches = vec![vec![]; nfa.state_len()];
        let mut dfa = Self {
            premultiplied: false,
            start_id: START_ID,
            state_count: nfa.state_len(),
            max_match: FAIL_ID,
            trans,
            matches,
        };
        for id in 0..nfa.state_len() {
            dfa.matches[id].extend_from_slice(nfa.matches(id));

            let fail = nfa.failure_transition(id);
            nfa.iter_all_transitions(id, |b, mut next| {
                if next == FAIL_ID {
                    next = nfa_next_state_memoized(nfa, &dfa, id, fail, b);
                }
                dfa.set_next_state(id, b, next);
            });
        }
        dfa.shuffle_match_states();
        dfa.premultiply();
        dfa
    }

    fn is_match_state(&self, id: usize) -> bool {
        id <= self.max_match && id > DEAD_ID
    }

    const fn is_match_or_dead_state(&self, id: usize) -> bool {
        id <= self.max_match
    }

    fn next_state(&self, from: usize, byte: u8) -> usize {
        let alphabet_len = ALPHABET_LEN;
        self.trans[from * alphabet_len + byte as usize]
    }

    fn set_next_state(&mut self, from: usize, byte: u8, to: usize) {
        let alphabet_len = ALPHABET_LEN;
        self.trans[from * alphabet_len + byte as usize] = to;
    }

    fn swap_states(&mut self, id1: usize, id2: usize) {
        assert!(!self.premultiplied, "can't swap states in premultiplied DFA");

        let o1 = id1 * ALPHABET_LEN;
        let o2 = id2 * ALPHABET_LEN;
        for b in 0..ALPHABET_LEN {
            self.trans.swap(o1 + b, o2 + b);
        }
        self.matches.swap(id1, id2);
    }

    fn shuffle_match_states(&mut self) {
        assert!(!self.premultiplied, "cannot shuffle match states of premultiplied DFA");

        if self.state_count <= 1 {
            return;
        }

        let mut first_non_match = self.start_id;
        while first_non_match < self.state_count && !self.matches[first_non_match].is_empty() {
            first_non_match += 1;
        }

        let mut swaps: Vec<usize> = vec![FAIL_ID; self.state_count];
        let mut cur = self.state_count - 1;
        while cur > first_non_match {
            if !self.matches[cur].is_empty() {
                self.swap_states(cur, first_non_match);
                swaps[cur] = first_non_match;
                swaps[first_non_match] = cur;

                first_non_match += 1;
                while first_non_match < cur && !self.matches[first_non_match].is_empty() {
                    first_non_match += 1;
                }
            }
            cur -= 1;
        }
        for id in 0..self.state_count {
            let alphabet_len = ALPHABET_LEN;
            let offset = id * alphabet_len;
            for next in &mut self.trans[offset..offset + alphabet_len] {
                if swaps[*next] != FAIL_ID {
                    *next = swaps[*next];
                }
            }
        }
        if swaps[self.start_id] != FAIL_ID {
            self.start_id = swaps[self.start_id];
        }
        self.max_match = first_non_match - 1;
    }

    fn premultiply(&mut self) {
        if self.premultiplied || self.state_count <= 1 {
            return;
        }

        for id in 2..self.state_count {
            let offset = id * ALPHABET_LEN;
            for next in &mut self.trans[offset..offset + ALPHABET_LEN] {
                if *next == DEAD_ID {
                    continue;
                }
                *next *= ALPHABET_LEN;
            }
        }
        self.premultiplied = true;
        self.start_id *= ALPHABET_LEN;
        self.max_match *= ALPHABET_LEN;
    }
}

fn nfa_next_state_memoized(
    nfa: &NFA,
    dfa: &DFA,
    populating: usize,
    mut current: usize,
    input: u8,
) -> usize {
    loop {
        if current < populating {
            return dfa.next_state(current, input);
        }
        let next = nfa.next_state(current, input);
        if next != FAIL_ID {
            return next;
        }
        current = nfa.failure_transition(current);
    }
}
