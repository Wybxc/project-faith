use crate::utils::Timer;

pub struct GlobalState {
    /// The current round number.
    pub round: u32,

    /// Indicates if the game is finished.
    pub finished: bool,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            round: 0,
            finished: false,
        }
    }
}

pub struct TurnTimer(pub Timer);

#[derive(Default, Clone)]
pub struct DebugLog {
    pub entries: Vec<String>,
}

impl DebugLog {
    pub fn push(&mut self, entry: impl Into<String>) {
        self.entries.push(entry.into());
    }
}
