use crate::recording::{RecordingCommand, RecordingStateManager};
use dictara_keyboard::{grab, EventType, Key};
use log::error;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;

/// Configurable trigger keys for recording actions
const RECORDING_TRIGGER: Key = Key::Function;
const LOCK_MODIFIER: Key = Key::Space;

/// Keyboard listener that detects key events and emits recording commands
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
                    EventType::KeyPress(key) if key == RECORDING_TRIGGER => {
                        let _ = command_tx.blocking_send(RecordingCommand::StartRecording);
                        None // Swallow to block emoji picker
                    }
                    EventType::KeyRelease(key) if key == RECORDING_TRIGGER => {
                        let _ = command_tx.blocking_send(RecordingCommand::StopRecording);
                        None // Swallow to block emoji picker
                    }
                    EventType::KeyPress(key) if key == LOCK_MODIFIER => {
                        if state_manager.is_busy() {
                            let _ = command_tx.blocking_send(RecordingCommand::LockRecording);
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
