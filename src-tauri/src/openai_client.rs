use async_openai::{
    config::OpenAIConfig,
    types::{AudioResponseFormat, CreateTranscriptionRequestArgs},
    Client,
};
use std::path::PathBuf;

const MIN_AUDIO_DURATION_MS: u64 = 500; // Minimum 0.5 seconds
const MAX_FILE_SIZE_BYTES: u64 = 25 * 1024 * 1024; // 25MB limit

#[derive(Debug)]
pub enum TranscriptionError {
    AudioTooShort { duration_ms: u64 },
    FileTooLarge { size_bytes: u64 },
    FileNotFound(String),
    ApiError(String),
    IoError(std::io::Error),
    ApiKeyMissing,
}

impl From<std::io::Error> for TranscriptionError {
    fn from(err: std::io::Error) -> Self {
        TranscriptionError::IoError(err)
    }
}

impl std::fmt::Display for TranscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TranscriptionError::AudioTooShort { duration_ms } => {
                write!(f, "Audio too short: {}ms (minimum 500ms)", duration_ms)
            }
            TranscriptionError::FileTooLarge { size_bytes } => {
                write!(f, "File too large: {} bytes (maximum 25MB)", size_bytes)
            }
            TranscriptionError::FileNotFound(path) => {
                write!(f, "Audio file not found: {}", path)
            }
            TranscriptionError::ApiError(msg) => {
                write!(f, "OpenAI API error: {}", msg)
            }
            TranscriptionError::IoError(err) => {
                write!(f, "IO error: {}", err)
            }
            TranscriptionError::ApiKeyMissing => {
                write!(f, "OPENAI_API_KEY environment variable not set")
            }
        }
    }
}

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
}

impl Clone for OpenAIClient {
    fn clone(&self) -> Self {
        // Create a new client instance (Client is cheap to clone/recreate)
        OpenAIClient {
            client: Client::new(),
        }
    }
}

impl OpenAIClient {
    /// Create a new OpenAI client
    /// Reads API key from OPENAI_API_KEY environment variable
    pub fn new() -> Result<Self, TranscriptionError> {
        // Check if API key is set
        if std::env::var("OPENAI_API_KEY").is_err() {
            return Err(TranscriptionError::ApiKeyMissing);
        }

        Ok(OpenAIClient {
            client: Client::new(),
        })
    }

    /// Transcribe audio file to text
    ///
    /// # Arguments
    /// * `file_path` - Path to the audio file (WAV, MP3, etc.)
    /// * `duration_ms` - Duration of the recording in milliseconds (for validation)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Error details
    pub async fn transcribe_audio(
        &self,
        file_path: PathBuf,
        duration_ms: u64,
    ) -> Result<String, TranscriptionError> {
        println!(
            "[OpenAI Client] Transcribing: {:?} (duration: {}ms)",
            file_path, duration_ms
        );

        // Validate minimum duration
        if duration_ms < MIN_AUDIO_DURATION_MS {
            eprintln!(
                "[OpenAI Client] Audio too short: {}ms < {}ms",
                duration_ms, MIN_AUDIO_DURATION_MS
            );
            return Err(TranscriptionError::AudioTooShort { duration_ms });
        }

        // Check if file exists
        if !file_path.exists() {
            eprintln!("[OpenAI Client] File not found: {:?}", file_path);
            return Err(TranscriptionError::FileNotFound(
                file_path.to_string_lossy().to_string(),
            ));
        }

        // Check file size
        let metadata = std::fs::metadata(&file_path)?;
        let file_size = metadata.len();

        if file_size > MAX_FILE_SIZE_BYTES {
            eprintln!(
                "[OpenAI Client] File too large: {} bytes > {} bytes",
                file_size, MAX_FILE_SIZE_BYTES
            );
            return Err(TranscriptionError::FileTooLarge {
                size_bytes: file_size,
            });
        }

        println!("[OpenAI Client] File size: {} bytes", file_size);

        let model = "gpt-4o-transcribe"; // "whisper-1"

        // Build transcription request
        let request = CreateTranscriptionRequestArgs::default()
            .file(file_path.to_string_lossy().to_string())
            .prompt("If input is empty do not return anything")
            .model(model)
            .temperature(0.0)
            .response_format(AudioResponseFormat::Json)
            .build()
            .map_err(|e| TranscriptionError::ApiError(format!("Failed to build request: {}", e)))?;

        // Call OpenAI API
        println!("[OpenAI Client] Sending request to OpenAI API...");
        let response = self.client.audio().transcribe(request).await.map_err(|e| {
            eprintln!("[OpenAI Client] API error: {}", e);
            TranscriptionError::ApiError(format!("{}", e))
        })?;

        println!(
            "[OpenAI Client] Transcription successful: {} characters",
            response.text.len()
        );
        println!("[OpenAI Client] Text: {}", response.text);

        Ok(response.text)
    }

    /// Transcribe with verbose output (includes timestamps and metadata)
    /// This is useful for future features like word-level highlighting
    #[allow(dead_code)]
    pub async fn transcribe_audio_verbose(
        &self,
        file_path: PathBuf,
        duration_ms: u64,
    ) -> Result<(String, Option<Vec<String>>), TranscriptionError> {
        use async_openai::types::TimestampGranularity;

        println!("[OpenAI Client] Transcribing (verbose): {:?}", file_path);

        // Same validation as regular transcribe
        if duration_ms < MIN_AUDIO_DURATION_MS {
            return Err(TranscriptionError::AudioTooShort { duration_ms });
        }

        if !file_path.exists() {
            return Err(TranscriptionError::FileNotFound(
                file_path.to_string_lossy().to_string(),
            ));
        }

        let metadata = std::fs::metadata(&file_path)?;
        if metadata.len() > MAX_FILE_SIZE_BYTES {
            return Err(TranscriptionError::FileTooLarge {
                size_bytes: metadata.len(),
            });
        }

        // Build verbose request with timestamps
        let request = CreateTranscriptionRequestArgs::default()
            .file(file_path.to_string_lossy().to_string())
            .model("whisper-1")
            .response_format(AudioResponseFormat::VerboseJson)
            .timestamp_granularities(vec![
                TimestampGranularity::Word,
                TimestampGranularity::Segment,
            ])
            .build()
            .map_err(|e| TranscriptionError::ApiError(format!("Failed to build request: {}", e)))?;

        println!("[OpenAI Client] Sending verbose request to OpenAI API...");
        let response = self
            .client
            .audio()
            .transcribe_verbose_json(request)
            .await
            .map_err(|e| TranscriptionError::ApiError(format!("{}", e)))?;

        let word_count = response.words.as_ref().map(|w| w.len());
        println!(
            "[OpenAI Client] Transcription successful: {} characters, {} words",
            response.text.len(),
            word_count.unwrap_or(0)
        );

        // Extract words for future use
        let words = response
            .words
            .map(|w| w.iter().map(|word| word.word.clone()).collect());

        Ok((response.text, words))
    }
}
