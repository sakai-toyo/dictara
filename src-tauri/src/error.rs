use derive_more::From;

#[derive(Debug, From)]
#[allow(dead_code)]
pub enum Error {
    #[from]
    Recorder(crate::recording::RecorderError),

    #[from]
    Transcription(crate::clients::openai::TranscriptionError),

    #[from]
    ClipboardPaste(crate::clipboard_paste::ClipboardPasteError),

    #[from]
    Tauri(tauri::Error),
}
