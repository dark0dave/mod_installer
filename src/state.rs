#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    RequiresInput { question: String },
    InProgress,
    TimedOut { timeout: usize, weidu_log: String },
    Completed,
    CompletedWithErrors { weidu_log: String },
    CompletedWithWarnings { weidu_log: String },
}
