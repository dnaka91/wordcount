use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

use super::state_id::{DEAD_ID, FAIL_ID};

pub const START_ID: usize = 2;

#[derive(Clone)]
pub struct NFA {
    states: Vec<State>,
}

impl NFA {
    pub fn new<I, P>(patterns: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        Compiler::new().compile(patterns)
    }

    pub fn state_len(&self) -> usize {
        self.states.len()
    }

    pub fn matches(&self, id: usize) -> &[(usize, usize)] {
        &self.states[id].matches
    }

    pub fn iter_all_transitions<F: FnMut(u8, usize)>(&self, id: usize, f: F) {
        self.states[id].trans.iter_all(f);
    }

    pub fn failure_transition(&self, id: usize) -> usize {
        self.states[id].fail
    }

    pub fn next_state(&self, current: usize, input: u8) -> usize {
        self.states[current].next_state(input)
    }

    fn state(&self, id: usize) -> &State {
        &self.states[id]
    }

    fn state_mut(&mut self, id: usize) -> &mut State {
        &mut self.states[id]
    }

    fn start(&self) -> &State {
        self.state(START_ID)
    }

    fn start_mut(&mut self) -> &mut State {
        self.state_mut(START_ID)
    }

    fn iter_transitions_mut(&mut self, id: usize) -> IterTransitionsMut {
        IterTransitionsMut::new(self, id)
    }

    fn copy_matches(&mut self, src: usize, dst: usize) {
        let (src, dst) = get_two_mut(&mut self.states, src, dst);
        dst.matches.extend_from_slice(&src.matches);
    }

    fn copy_empty_matches(&mut self, dst: usize) {
        self.copy_matches(START_ID, dst);
    }

    fn add_dense_state(&mut self, depth: usize) -> usize {
        let trans = Transitions::Dense(Dense::new());
        let id = self.states.len();
        self.states.push(State { trans, fail: START_ID, depth, matches: vec![] });
        id
    }

    fn add_sparse_state(&mut self, depth: usize) -> usize {
        let trans = Transitions::Sparse(vec![]);
        let id = self.states.len();
        self.states.push(State { trans, fail: START_ID, depth, matches: vec![] });
        id
    }
}

#[derive(Clone, Debug)]
pub struct State {
    trans: Transitions,
    fail: usize,
    matches: Vec<(usize, usize)>,
    depth: usize,
}

impl State {
    fn add_match(&mut self, i: usize, len: usize) {
        self.matches.push((i, len));
    }

    fn is_match(&self) -> bool {
        !self.matches.is_empty()
    }

    fn next_state(&self, input: u8) -> usize {
        self.trans.next_state(input)
    }

    fn set_next_state(&mut self, input: u8, next: usize) {
        self.trans.set_next_state(input, next);
    }
}

#[derive(Clone, Debug)]
struct Dense(Vec<usize>);

impl Dense {
    fn new() -> Self {
        Self(vec![FAIL_ID; 256])
    }

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl Index<u8> for Dense {
    type Output = usize;

    #[inline]
    fn index(&self, i: u8) -> &usize {
        &self.0[i as usize]
    }
}

impl IndexMut<u8> for Dense {
    #[inline]
    fn index_mut(&mut self, i: u8) -> &mut usize {
        &mut self.0[i as usize]
    }
}

#[derive(Clone, Debug)]
enum Transitions {
    Sparse(Vec<(u8, usize)>),
    Dense(Dense),
}

impl Transitions {
    fn next_state(&self, input: u8) -> usize {
        match *self {
            Self::Sparse(ref sparse) => {
                for &(b, id) in sparse {
                    if b == input {
                        return id;
                    }
                }
                FAIL_ID
            }
            Self::Dense(ref dense) => dense[input],
        }
    }

    fn set_next_state(&mut self, input: u8, next: usize) {
        match *self {
            Self::Sparse(ref mut sparse) => {
                match sparse.binary_search_by_key(&input, |&(b, _)| b) {
                    Ok(i) => sparse[i] = (input, next),
                    Err(i) => sparse.insert(i, (input, next)),
                }
            }
            Self::Dense(ref mut dense) => {
                dense[input] = next;
            }
        }
    }

    fn iter_all<F: FnMut(u8, usize)>(&self, mut f: F) {
        match *self {
            Self::Sparse(ref sparse) => {
                sparse_iter(sparse, f);
            }
            Self::Dense(ref dense) => {
                for b in 0..=255 {
                    f(b, dense[b]);
                }
            }
        }
    }
}

struct IterTransitionsMut<'a> {
    nfa: &'a mut NFA,
    state_id: usize,
    cur: usize,
}

impl<'a> IterTransitionsMut<'a> {
    fn new(nfa: &'a mut NFA, state_id: usize) -> IterTransitionsMut<'a> {
        IterTransitionsMut { nfa, state_id, cur: 0 }
    }

    fn nfa(&mut self) -> &mut NFA {
        self.nfa
    }
}

impl<'a> Iterator for IterTransitionsMut<'a> {
    type Item = (u8, usize);

    fn next(&mut self) -> Option<(u8, usize)> {
        match self.nfa.states[self.state_id].trans {
            Transitions::Sparse(ref sparse) => {
                if self.cur >= sparse.len() {
                    return None;
                }
                let i = self.cur;
                self.cur += 1;
                Some(sparse[i])
            }
            Transitions::Dense(ref dense) => {
                while self.cur < dense.len() {
                    debug_assert!(self.cur < 256);

                    let b = self.cur as u8;
                    let id = dense[b];
                    self.cur += 1;
                    if id != FAIL_ID {
                        return Some((b, id));
                    }
                }
                None
            }
        }
    }
}

struct Compiler {
    nfa: NFA,
}

impl Compiler {
    fn new() -> Self {
        Self { nfa: NFA { states: Vec::with_capacity(3200) } }
    }

    fn compile<I, P>(mut self, patterns: I) -> NFA
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        self.add_state(0); // the fail state, which is never entered
        self.add_state(0); // the dead state, only used for leftmost
        self.add_state(0); // the start state
        self.build_trie(patterns);
        self.add_start_state_loop();
        self.add_dead_state_loop();
        self.fill_failure_transitions_standard();
        self.nfa
    }

    fn build_trie<I, P>(&mut self, patterns: I)
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        for (pati, pat) in patterns.into_iter().enumerate() {
            let pat = pat.as_ref();

            let mut prev = START_ID;
            let mut saw_match = false;
            for (depth, &b) in pat.iter().enumerate() {
                saw_match = saw_match || self.nfa.state(prev).is_match();

                let next = self.nfa.state(prev).next_state(b);
                if next == FAIL_ID {
                    let next = self.add_state(depth + 1);
                    self.nfa.state_mut(prev).set_next_state(b, next);
                    prev = next;
                } else {
                    prev = next;
                }
            }

            self.nfa.state_mut(prev).add_match(pati, pat.len());
        }
    }

    fn fill_failure_transitions_standard(&mut self) {
        let mut queue = VecDeque::with_capacity(720);
        for b in 0..=255 {
            let next = self.nfa.start().next_state(b);
            if next != START_ID {
                queue.push_back(next);
            }
        }
        while let Some(id) = queue.pop_front() {
            let mut it = self.nfa.iter_transitions_mut(id);
            while let Some((b, next)) = it.next() {
                queue.push_back(next);

                let mut fail = it.nfa().state(id).fail;
                while it.nfa().state(fail).next_state(b) == FAIL_ID {
                    fail = it.nfa().state(fail).fail;
                }
                fail = it.nfa().state(fail).next_state(b);
                it.nfa().state_mut(next).fail = fail;
                it.nfa().copy_matches(fail, next);
            }

            it.nfa().copy_empty_matches(id);
        }
    }

    fn add_start_state_loop(&mut self) {
        let start = self.nfa.start_mut();
        for b in 0..=255 {
            if start.next_state(b) == FAIL_ID {
                start.set_next_state(b, START_ID);
            }
        }
    }

    fn add_dead_state_loop(&mut self) {
        let dead = self.nfa.state_mut(DEAD_ID);
        for b in 0..=255 {
            dead.set_next_state(b, DEAD_ID);
        }
    }

    fn add_state(&mut self, depth: usize) -> usize {
        if depth < 2 {
            self.nfa.add_dense_state(depth)
        } else {
            self.nfa.add_sparse_state(depth)
        }
    }
}

fn sparse_iter<F: FnMut(u8, usize)>(trans: &[(u8, usize)], mut f: F) {
    let mut byte = 0_u16;
    for &(b, id) in trans {
        while byte < u16::from(b) {
            f(byte as u8, FAIL_ID);
            byte += 1;
        }
        f(b, id);
        byte += 1;
    }
    for b in byte..256 {
        f(b as u8, FAIL_ID);
    }
}

fn get_two_mut<T>(xs: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j, "{} must not be equal to {}", i, j);
    if i < j {
        let (before, after) = xs.split_at_mut(j);
        (&mut before[i], &mut after[0])
    } else {
        let (before, after) = xs.split_at_mut(i);
        (&mut after[0], &mut before[j])
    }
}
