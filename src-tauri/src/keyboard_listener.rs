use crate::recording::RecordingCommand;
use rdev::{listen, Event, EventType, Key, ListenError};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;

/// Stateful FN key listener
pub struct KeyListener {
    _thread_handle: Option<JoinHandle<()>>,
}

impl KeyListener {
    /// Create and start a new FN key listener that sends commands through a channel
    ///
    /// # Arguments
    /// * `command_tx` - Channel sender for recording commands
    pub fn start(command_tx: mpsc::Sender<RecordingCommand>) -> Self {
        let thread_handle = thread::spawn(move || {
            println!("[FN Key Listener] Starting global keyboard listener...");

            let listen_res = listen(move |event: Event| {
                // Only handle Function key events
                match event.event_type {
                    EventType::KeyPress(Key::Function) => {
                        println!("[✅] 🔑 FN Key");

                        // Send Start command through channel (blocking since we're in sync thread)
                        if let Err(e) = command_tx.blocking_send(RecordingCommand::Start) {
                            eprintln!("[FN Key Listener] Failed to send Start command: {}", e);
                        }
                    }
                    EventType::KeyRelease(Key::Function) => {
                        println!("[🔓] 🔑 FN Key: RELEASED");

                        // Send Stop command through channel (blocking since we're in sync thread)
                        if let Err(e) = command_tx.blocking_send(RecordingCommand::Stop) {
                            eprintln!("[FN Key Listener] Failed to send Stop command: {}", e);
                        }
                    }
                    _ => {}
                }
            });

            match listen_res {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("[FN Key Listener] Error: {:?}", error);
                    let error_msg = match error {
                        ListenError::EventTapError => {
                            "macOS Accessibility permission denied. Please grant permission and restart."
                        }
                        _ => "Keyboard listener failed",
                    };
                    eprintln!("[FN Key Listener] {}", error_msg);
                }
            }

            println!("[FN Key Listener] Thread exiting");
        });

        Self {
            _thread_handle: Some(thread_handle),
        }
    }
}
