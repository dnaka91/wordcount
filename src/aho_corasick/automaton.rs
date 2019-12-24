use super::state_id::{DEAD_ID, FAIL_ID};
use super::Match;

pub trait Automaton {
    fn start_state(&self) -> usize;

    fn is_valid(&self, id: usize) -> bool;

    fn is_match_state(&self, id: usize) -> bool;

    fn is_match_or_dead_state(&self, id: usize) -> bool {
        id == DEAD_ID || self.is_match_state(id)
    }

    fn get_match(&self, id: usize, match_index: usize, end: usize) -> Option<Match>;

    fn match_count(&self, id: usize) -> usize;

    fn next_state(&self, current: usize, input: u8) -> usize;

    fn next_state_no_fail(&self, current: usize, input: u8) -> usize {
        let next = self.next_state(current, input);
        debug_assert!(next != FAIL_ID, "automaton should never return fail_id for next state");
        next
    }

    #[inline]
    fn standard_find_at(&self, haystack: &[u8], at: usize, state_id: &mut usize) -> Option<Match> {
        assert!(self.is_valid(*state_id), "{} is not a valid state ID", state_id);
        let mut at = at;
        while at < haystack.len() {
            *state_id = self.next_state_no_fail(*state_id, haystack[at]);
            at += 1;

            debug_assert!(*state_id != DEAD_ID, "standard find should never see a dead state");

            if self.is_match_or_dead_state(*state_id) {
                return if *state_id == DEAD_ID { None } else { self.get_match(*state_id, 0, at) };
            }
        }
        None
    }

    #[inline]
    fn overlapping_find_at(
        &self,
        haystack: &[u8],
        at: usize,
        state_id: &mut usize,
        match_index: &mut usize,
    ) -> Option<Match> {
        let match_count = self.match_count(*state_id);
        if *match_index < match_count {
            let result = self.get_match(*state_id, *match_index, at);
            debug_assert!(result.is_some(), "must be a match");
            *match_index += 1;
            return result;
        }

        *match_index = 0;
        match self.standard_find_at(haystack, at, state_id) {
            None => None,
            Some(m) => {
                *match_index = 1;
                Some(m)
            }
        }
    }
}
