use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    RecorderError(crate::recording::RecorderError),

    #[from]
    TranscriptionError(crate::clients::openai::TranscriptionError),

    #[from]
    ClipboardPasteError(crate::clipboard_paste::ClipboardPasteError),

    #[from]
    TauriError(tauri::Error),
}
