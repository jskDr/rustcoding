use crossterm::event::KeyEvent;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Key(KeyEvent),
    Tick,
    // AI events
    StreamChunk(String),
    ThinkingChunk(String),
    ThinkingEnd,
    StreamComplete(String),
    ReviewStatus(String),
    ReviewOk,
    RetryStart,
    FinalReply(String),
    Error(String),
}
