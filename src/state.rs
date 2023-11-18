#[derive(Debug)]
pub enum State {
    RequiresInput { question: String },
    InProgress,
    Completed,
    CompletedWithErrors { error_details: String },
    CompletedWithWarnings,
}
