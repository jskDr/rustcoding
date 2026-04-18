mod app;
mod event;
mod ui;

use std::error::Error;
use std::time::Duration;

use crossterm::event::EventStream;
use tokio::sync::mpsc;
use futures::StreamExt;
use tokio::time::interval;

use rig::agent::Agent;
use rig::agent::{MultiTurnStreamItem::StreamAssistantItem, StreamingResult};
use rig::client::{CompletionClient, Nothing};
use rig::completion::Prompt;
use rig::message::Message;
use rig::providers::ollama;
use rig::streaming::{StreamedAssistantContent, StreamingChat};
use serde_json::json;

use app::App;
use event::AppEvent;

type OllamaAgent = Agent<ollama::CompletionModel>;

const MAX_RETRIES: usize = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = ollama::Client::new(Nothing).unwrap();

    let teacher = client
        .agent("gemma4:e2b")
        .preamble(include_str!("prompts/system_prompt_en_teacher.md"))
        .additional_params(json!({"think": true}))
        .build();

    let reviewer = client
        .agent("gemma4:e2b")
        .preamble(include_str!("prompts/system_prompt_reviewer.md"))
        .build();

    // Setup Ratatui
    let mut terminal = ratatui::init();

    // Channels for events
    let (tx, mut rx) = mpsc::channel(100);

    // Spawn event reader task
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        let mut tick_interval = interval(Duration::from_millis(50));
        
        loop {
            tokio::select! {
                _ = tick_interval.tick() => {
                    let _ = tx_clone.send(AppEvent::Tick).await;
                }
                Some(Ok(event)) = reader.next() => {
                    if let crossterm::event::Event::Key(key) = event {
                        let _ = tx_clone.send(AppEvent::Key(key)).await;
                    }
                }
            }
        }
    });

    let mut app_state = App::new();

    // Main event loop
    loop {
        // Render UI
        terminal.draw(|f| ui::render(f, &app_state))?;
        
        let should_quit = app_state.should_quit;
        if should_quit {
            break;
        }

        tokio::select! {
            Some(event) = rx.recv() => {
                // If it's a render tick, just break out of select to loop and draw
                if matches!(event, AppEvent::Tick) {
                    // Let the loop redraw
                } else {
                    // It's a state-mutating event
                    let submit_input = app_state.handle_event(event);
                    
                    if let Some(user_input) = submit_input {
                        // User submitted a message, spawn AI task
                        let tx_ai = tx.clone();
                        let teacher_clone = teacher.clone();
                        let reviewer_clone = reviewer.clone();
                        let mut task_history = app_state.history.clone();
                        let review_mode = app_state.review_mode;
                        
                        tokio::spawn(async move {
                            // Run the AI logic
                            let result = run_ai_task(
                                teacher_clone,
                                reviewer_clone,
                                user_input.clone(),
                                &mut task_history,
                                review_mode,
                                tx_ai.clone(),
                            ).await.map_err(|e| e.to_string());
                            
                            match result {
                                Ok(final_reply) => {
                                    // task_history is already updated inside run_ai_task
                                    let _ = tx_ai.send(AppEvent::FinalReply(final_reply)).await;
                                }
                                Err(error_msg) => {
                                    let _ = tx_ai.send(AppEvent::Error(error_msg)).await;
                                }
                            }
                        });
                    }
                }
            }
        }
    }

    ratatui::restore();
    Ok(())
}

async fn run_ai_task(
    teacher: OllamaAgent,
    reviewer: OllamaAgent,
    input: String,
    history: &mut Vec<Message>,
    review_mode: bool,
    tx: mpsc::Sender<AppEvent>,
) -> Result<String, Box<dyn Error>> {
    
    // 1. Get initial streaming response
    let stream = teacher.stream_chat(&input, history.clone()).await;
    let assistant_reply = stream_and_send(stream, &tx).await?;

    let final_reply = if review_mode {
        // 2. Review and potentially retry
        review_response_with_events(teacher, reviewer, &input, assistant_reply, history, &tx).await?
    } else {
        assistant_reply
    };

    history.push(Message::user(&input));
    history.push(Message::assistant(&final_reply));

    Ok(final_reply)
}

async fn stream_and_send<R>(
    mut stream: StreamingResult<R>,
    tx: &mpsc::Sender<AppEvent>,
) -> Result<String, Box<dyn Error>> {
    let mut full_reply = String::new();

    while let Some(chunk) = stream.next().await {
        match chunk? {
            StreamAssistantItem(StreamedAssistantContent::Text(text)) => {
                let _ = tx.send(AppEvent::StreamChunk(text.text.clone())).await;
                full_reply.push_str(&text.text);
            }
            StreamAssistantItem(StreamedAssistantContent::Reasoning(reasoning)) => {
                let text = reasoning.display_text();
                let _ = tx.send(AppEvent::ThinkingChunk(text)).await;
            }
            StreamAssistantItem(StreamedAssistantContent::ReasoningDelta {
                reasoning, ..
            }) => {
                let _ = tx.send(AppEvent::ThinkingChunk(reasoning)).await;
            }
            StreamAssistantItem(StreamedAssistantContent::Final(_)) => {
                let _ = tx.send(AppEvent::ThinkingEnd).await;
            }
            _ => {}
        }
    }

    let _ = tx.send(AppEvent::StreamComplete(full_reply.clone())).await;
    Ok(full_reply)
}

async fn review_response_with_events(
    teacher: OllamaAgent,
    reviewer: OllamaAgent,
    input: &str,
    initial_reply: String,
    history: &[Message],
    tx: &mpsc::Sender<AppEvent>,
) -> Result<String, Box<dyn Error>> {
    let mut assistant_reply = initial_reply;

    for retry in 0..MAX_RETRIES {
        let review_prompt = format!(
            "Student said: \"{input}\"\n\nTeacher replied: \"{assistant_reply}\"\n\nIs this response OK?"
        );

        let review_result = reviewer.prompt(&review_prompt).await?;
        let review_text = review_result.to_string();

        if review_text.trim().starts_with("OK") {
            let _ = tx.send(AppEvent::ReviewOk).await;
            break;
        } else {
            let _ = tx.send(AppEvent::ReviewStatus(review_text.clone())).await;

            if retry < MAX_RETRIES - 1 {
                let _ = tx.send(AppEvent::RetryStart).await;
                
                let correction_prompt = format!(
                    "A reviewer found a problem with your previous response. \
                    Feedback: {review_text}\n\n\
                    Please correct your response to the student who said: \"{input}\""
                );

                let mut retry_history = history.to_vec();
                retry_history.push(Message::user(input));
                retry_history.push(Message::assistant(&assistant_reply));

                let stream = teacher
                    .stream_chat(&correction_prompt, retry_history)
                    .await;
                assistant_reply = stream_and_send(stream, tx).await?;
            }
        }
    }

    Ok(assistant_reply)
}