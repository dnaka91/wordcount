use super::automaton::Automaton;
use super::dfa::{self, Dfa};
use super::nfa;
use super::Match;

#[derive(Clone)]
pub struct AhoCorasick {
    imp: Dfa,
}

impl AhoCorasick {
    pub fn new<I, P>(patterns: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<[u8]>,
    {
        let nfa = nfa::Nfa::new(patterns);
        let dfa = dfa::Dfa::new(&nfa);

        Self { imp: dfa }
    }

    pub fn find_overlapping_iter<'a, 'b, B: ?Sized + AsRef<[u8]>>(
        &'a self,
        haystack: &'b B,
    ) -> FindIter<'a, 'b> {
        FindIter::new(self, haystack.as_ref())
    }
}

pub struct FindIter<'a, 'b> {
    fsm: &'a Dfa,
    haystack: &'b [u8],
    pos: usize,
    state_id: usize,
    match_index: usize,
}

impl<'a, 'b> FindIter<'a, 'b> {
    fn new(ac: &'a AhoCorasick, haystack: &'b [u8]) -> FindIter<'a, 'b> {
        FindIter { fsm: &ac.imp, haystack, pos: 0, state_id: ac.imp.start_state(), match_index: 0 }
    }
}

impl<'a, 'b> Iterator for FindIter<'a, 'b> {
    type Item = Match;

    fn next(&mut self) -> Option<Match> {
        let result = self.fsm.overlapping_find_at(
            self.haystack,
            self.pos,
            &mut self.state_id,
            &mut self.match_index,
        );
        match result {
            None => None,
            Some(m) => {
                self.pos = m.end();
                Some(m)
            }
        }
    }
}
