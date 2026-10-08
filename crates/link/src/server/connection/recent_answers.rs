use std::collections::VecDeque;

use protocol::message::Outcome;

const REMEMBERED: usize = 256;

/// The last answers of one connection, so a command sent again with the same id is answered
/// again and not run twice.
#[derive(Default)]
pub(super) struct RecentAnswers {
    answers: VecDeque<(u64, Outcome)>,
}

impl RecentAnswers {
    pub(super) fn find(&self, id: u64) -> Option<&Outcome> {
        self.answers
            .iter()
            .find(|(known, _)| *known == id)
            .map(|(_, outcome)| outcome)
    }

    pub(super) fn remember(&mut self, id: u64, outcome: Outcome) {
        if self.answers.len() == REMEMBERED {
            self.answers.pop_front();
        }
        self.answers.push_back((id, outcome));
    }
}
