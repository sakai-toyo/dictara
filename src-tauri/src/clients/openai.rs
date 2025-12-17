use async_openai::{
    config::OpenAIConfig,
    types::{AudioResponseFormat, CreateTranscriptionRequestArgs},
    Client,
};
use crate::keychain;
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
    /// Loads API key from macOS Keychain and sets it in environment
    /// Always succeeds - key will be checked at transcription time
    pub fn new() -> Self {
        println!("[OpenAI Client] Initializing client");

        // Try to load API key from keychain
        match keychain::load_api_key() {
            Ok(Some(api_key)) => {
                println!("[OpenAI Client] ✅ API key loaded from keychain");
                std::env::set_var("OPENAI_API_KEY", &api_key);
            }
            Ok(None) => {
                println!("[OpenAI Client] ⚠️  No API key found in keychain");
            }
            Err(e) => {
                eprintln!("[OpenAI Client] ❌ Failed to load API key from keychain: {:?}", e);
            }
        }

        OpenAIClient {
            client: Client::new(),
        }
    }

    /// Check if an API key is configured in the keychain
    pub fn has_api_key() -> bool {
        match keychain::load_api_key() {
            Ok(Some(_)) => true,
            _ => false,
        }
    }

    /// Test if an API key is valid by calling the OpenAI /v1/models endpoint
    ///
    /// # Arguments
    /// * `key` - The API key to test
    ///
    /// # Returns
    /// * `Ok(true)` - Key is valid
    /// * `Ok(false)` - Key is invalid (401 Unauthorized)
    /// * `Err(TranscriptionError)` - Network or other API error
    pub fn test_api_key(key: &str) -> Result<bool, TranscriptionError> {
        println!("[OpenAI Client] Testing API key validity...");

        let client = reqwest::blocking::Client::new();
        let response = client
            .get("https://api.openai.com/v1/models")
            .bearer_auth(key)
            .send()
            .map_err(|e| {
                eprintln!("[OpenAI Client] Request failed: {}", e);
                TranscriptionError::ApiError(format!("Request failed: {}", e))
            })?;

        let status = response.status();
        println!("[OpenAI Client] API test response status: {}", status);

        if status.is_success() {
            println!("[OpenAI Client] ✅ API key is valid");
            Ok(true)
        } else if status.as_u16() == 401 {
            println!("[OpenAI Client] ❌ API key is invalid (401 Unauthorized)");
            Ok(false)
        } else {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            eprintln!(
                "[OpenAI Client] Unexpected API response ({}): {}",
                status, error_text
            );
            Err(TranscriptionError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )))
        }
    }

    /// Transcribe audio file to text (blocking/synchronous version)
    ///
    /// # Arguments
    /// * `file_path` - Path to the audio file (WAV, MP3, etc.)
    /// * `duration_ms` - Duration of the recording in milliseconds (for validation)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Error details
    pub fn transcribe_audio_sync(
        &self,
        file_path: PathBuf,
        duration_ms: u64,
    ) -> Result<String, TranscriptionError> {
        println!(
            "[OpenAI Client] Transcribing (sync): {:?} (duration: {}ms)",
            file_path, duration_ms
        );

        // Validate minimum duration
        if duration_ms < MIN_AUDIO_DURATION_MS {
            eprintln!(
                "[OpenAI Client] Audio too short: {}ms < {}ms",
                duration_ms, MIN_AUDIO_DURATION_MS
            );
            return Ok("".to_string());
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

        // Load API key from keychain each time (in case it was just saved)
        let api_key = match keychain::load_api_key() {
            Ok(Some(key)) => {
                println!("[OpenAI Client] Using API key from keychain");
                // Set in env for this request
                std::env::set_var("OPENAI_API_KEY", &key);
                key
            }
            Ok(None) => {
                eprintln!("[OpenAI Client] No API key configured");
                return Err(TranscriptionError::ApiKeyMissing);
            }
            Err(e) => {
                eprintln!("[OpenAI Client] Failed to load API key: {:?}", e);
                return Err(TranscriptionError::ApiKeyMissing);
            }
        };

        let model = "gpt-4o-transcribe";

        // Build multipart form
        let form = reqwest::blocking::multipart::Form::new()
            .file("file", &file_path)
            .map_err(|e| {
                TranscriptionError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to read file: {}", e),
                ))
            })?
            .text("model", model)
            .text("temperature", "0.0")
            .text("prompt", "If input is empty do not return anything.")
            .text("response_format", "json");

        // Call OpenAI API
        println!("[OpenAI Client] Sending request to OpenAI API...");
        let client = reqwest::blocking::Client::new();
        let response = client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(api_key)
            .multipart(form)
            .send()
            .map_err(|e| {
                eprintln!("[OpenAI Client] API request error: {}", e);
                TranscriptionError::ApiError(format!("Request failed: {}", e))
            })?;

        // Check response status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());
            eprintln!(
                "[OpenAI Client] API error response ({}): {}",
                status, error_text
            );
            return Err(TranscriptionError::ApiError(format!(
                "API returned status {}: {}",
                status, error_text
            )));
        }

        // Parse JSON response
        let json: serde_json::Value = response.json().map_err(|e| {
            eprintln!("[OpenAI Client] Failed to parse response: {}", e);
            TranscriptionError::ApiError(format!("Failed to parse response: {}", e))
        })?;

        let text = json["text"].as_str().unwrap_or("").to_string();

        println!(
            "[OpenAI Client] Transcription successful: {} characters",
            text.len()
        );
        println!("[OpenAI Client] Text: {}", text);

        Ok(text)
    }

    /// Transcribe audio file to text (async version)
    ///
    /// # Arguments
    /// * `file_path` - Path to the audio file (WAV, MP3, etc.)
    /// * `duration_ms` - Duration of the recording in milliseconds (for validation)
    ///
    /// # Returns
    /// * `Ok(String)` - Transcribed text
    /// * `Err(TranscriptionError)` - Error details
    #[allow(dead_code)]
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

        // let model = "gpt-4o-transcribe"; // "whisper-1"
        let model = "whisper-1";

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
}
