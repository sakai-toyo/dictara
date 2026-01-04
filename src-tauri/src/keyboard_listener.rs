use crate::recording::{RecordingCommand, RecordingState, RecordingStateManager};
use dictara_keyboard::{grab, EventType, Key};
use log::error;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;

/// Stateful FN key listener
pub struct KeyListener {
    _thread_handle: Option<JoinHandle<()>>,
}

impl KeyListener {
    pub fn start(
        command_tx: mpsc::Sender<RecordingCommand>,
        state_manager: Arc<RecordingStateManager>,
    ) -> Self {
        let thread_handle = thread::spawn(move || {
            if let Err(err) = grab(move |event| {
                match event.event_type {
                    EventType::KeyPress(Key::Function) => {
                        let _ = command_tx.blocking_send(RecordingCommand::FnDown);
                        None // Swallow to block emoji picker
                    }
                    EventType::KeyRelease(Key::Function) => {
                        let _ = command_tx.blocking_send(RecordingCommand::FnUp);
                        None // Swallow to block emoji picker
                    }
                    EventType::KeyPress(Key::Space) => {
                        if state_manager.current() == RecordingState::Recording {
                            let _ = command_tx.blocking_send(RecordingCommand::Lock);
                            None // Avoid inserting a space while recording
                        } else {
                            Some(event) // Pass through
                        }
                    }
                    _ => Some(event), // Pass through all other events
                }
            }) {
                error!(
                    "Keyboard grab failed: {}. Keyboard shortcuts will not work.",
                    err
                );
            }
        });

        Self {
            _thread_handle: Some(thread_handle),
        }
    }
}
