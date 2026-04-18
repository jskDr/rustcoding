use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rig::message::Message;

use crate::event::AppEvent;

#[derive(Debug, PartialEq, Clone)]
pub enum Role {
    User,
    Teacher,
    Reviewer,
    System,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub is_thinking: bool,
}

pub struct App {
    pub messages: Vec<ChatMessage>, // chat history for display
    pub input: String,              // current input text
    pub cursor_pos: usize,          // cursor position in input
    pub review_mode: bool,          // review toggle
    pub history: Vec<Message>,      // rig Message history for AI
    pub status: String,             // status line text
    pub scroll_offset: u16,         // chat view scroll offset
    pub auto_scroll: bool,          // automatically scroll to bottom
    pub should_quit: bool,
    pub is_processing: bool,        // is AI currently responding?
}

impl App {
    pub fn new() -> Self {
        Self {
            messages: vec![ChatMessage {
                role: Role::System,
                content: "English Teacher AI (with reviewer)\nType a message and press Enter. Press Esc or Ctrl+C to quit. Press F2 to toggle review mode.".to_string(),
                is_thinking: false,
            }],
            input: String::new(),
            cursor_pos: 0,
            review_mode: true,
            history: Vec::new(),
            status: "Ready".to_string(),
            scroll_offset: 0,
            auto_scroll: true,
            should_quit: false,
            is_processing: false,
        }
    }

    /// Handles an incoming event (from terminal or background task)
    /// Returns the text to send to the AI if the user submitted a message
    pub fn handle_event(&mut self, event: AppEvent) -> Option<String> {
        match event {
            AppEvent::Key(key_event) => self.handle_key_event(key_event),
            AppEvent::Tick => None, // tick just causes a render in the main loop
            
            // AI streaming updates
            AppEvent::StreamChunk(text) => {
                self.append_to_latest(Role::Teacher, &text);
                None
            }
            AppEvent::ThinkingChunk(text) => {
                self.append_to_latest_thinking(&text);
                None
            }
            AppEvent::ThinkingEnd => {
                self.end_thinking();
                None
            }
            AppEvent::StreamComplete(_full_text) => {
                self.status = "Teacher finished typing".to_string();
                None
            }
            
            // Review loop updates
            AppEvent::ReviewStatus(text) => {
                self.status = format!("Reviewer found issue: {}", text);
                self.messages.push(ChatMessage {
                    role: Role::Reviewer,
                    content: text,
                    is_thinking: false,
                });
                None
            }
            AppEvent::ReviewOk => {
                self.status = "Reviewer approved response".to_string();
                self.messages.push(ChatMessage {
                    role: Role::Reviewer,
                    content: "Response looks good!".to_string(),
                    is_thinking: false,
                });
                None
            }
            AppEvent::RetryStart => {
                self.status = "Teacher retrying...".to_string();
                // Add a visual separator or new message for the retry
                self.messages.push(ChatMessage {
                    role: Role::Teacher,
                    content: String::new(),
                    is_thinking: false,
                });
                None
            }
            AppEvent::FinalReply(_text) => {
                self.is_processing = false;
                self.status = "Ready".to_string();
                // We keep the history updated here since the AI task finished
                // Usually we appended to display dynamically, but we should update the logical rigor history.
                // It will be handled in the task or main loop.
                None
            }
            AppEvent::Error(err) => {
                self.is_processing = false;
                self.status = format!("Error: {}", err);
                self.messages.push(ChatMessage {
                    role: Role::System,
                    content: format!("Error: {}", err),
                    is_thinking: false,
                });
                None
            }
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<String> {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if key_event.code == KeyCode::Char('c') {
                self.should_quit = true;
            }
            return None;
        }

        match key_event.code {
            KeyCode::Esc => {
                self.should_quit = true;
                None
            }
            KeyCode::Enter => {
                if !self.input.is_empty() && !self.is_processing {
                    let text = self.input.clone();
                    self.input.clear();
                    self.cursor_pos = 0;
                    self.messages.push(ChatMessage {
                        role: Role::User,
                        content: text.clone(),
                        is_thinking: false,
                    });
                    self.messages.push(ChatMessage {
                        role: Role::Teacher,
                        content: String::new(),
                        is_thinking: false,
                    });
                    self.is_processing = true;
                    self.status = "Teacher is responding...".to_string();
                    self.scroll_offset = 0;
                    self.auto_scroll = true; // jump to bottom
                    Some(text)
                } else {
                    None
                }
            }
            KeyCode::Char(c) => {
                if !self.is_processing {
                    self.input.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                }
                None
            }
            KeyCode::Backspace => {
                if !self.is_processing && self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos);
                }
                None
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                None
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
                None
            }
            KeyCode::Up => {
                self.auto_scroll = false;
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                None
            }
            KeyCode::Down => {
                self.auto_scroll = false;
                self.scroll_offset = self.scroll_offset.saturating_add(1);
                None
            }
            KeyCode::F(2) => {
                self.review_mode = !self.review_mode;
                self.status = format!("Review mode: {}", if self.review_mode { "ON" } else { "OFF" });
                None
            }
            _ => None,
        }
    }

    fn append_to_latest(&mut self, target_role: Role, text: &str) {
        if let Some(msg) = self.messages.last_mut() {
            if msg.role == target_role {
                if msg.is_thinking {
                    msg.content.push_str("\n\n");
                    msg.is_thinking = false;
                }
                msg.content.push_str(text);
                return;
            }
        }
        // If the last message isn't the target role, add a new one
        self.messages.push(ChatMessage {
            role: target_role,
            content: text.to_string(),
            is_thinking: false,
        });
    }

    fn append_to_latest_thinking(&mut self, text: &str) {
        if let Some(msg) = self.messages.last_mut() {
            if msg.role == Role::Teacher {
                if !msg.is_thinking {
                    msg.content.push_str("\n[thinking]\n");
                    msg.is_thinking = true;
                }
                msg.content.push_str(text);
                return;
            }
        }
    }

    fn end_thinking(&mut self) {
        if let Some(msg) = self.messages.last_mut() {
            if msg.role == Role::Teacher && msg.is_thinking {
                msg.content.push_str("\n[/thinking]\n\n");
                msg.is_thinking = false;
            }
        }
    }
}
