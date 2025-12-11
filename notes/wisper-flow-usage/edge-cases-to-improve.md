# Edge Cases to Improve - Audio Transcription

This document tracks edge cases and potential improvements for the audio transcription feature.

## Transcription Edge Cases

### 1. What if transcription fails?
**Potential causes:**
- Network issues (no internet connection, timeout)
- Invalid API key or expired key
- OpenAI API rate limits
- Service outage

**Current handling:**
- Show error message to user
- Keep the audio file for retry
- Log error details to console

**Future improvements:**
- [ ] Implement automatic retry with exponential backoff
- [ ] Add manual "retry transcription" button in UI
- [ ] Queue failed transcriptions for batch retry when connection restored
- [ ] Show specific error messages (rate limit vs auth vs network)

### 2. What if audio is too short?
**Issue:**
- API might reject very short audio (< 0.5 seconds)
- User accidentally taps FN key

**Current handling:**
- Add minimum duration check before sending to API

**Future improvements:**
- [ ] Show warning if recording is too short
- [ ] Don't save audio files that are below minimum threshold
- [ ] Add visual/audio feedback when minimum duration is reached

### 3. What if user records in background noise?
**Issue:**
- Whisper is pretty robust but not perfect
- Heavy background noise can reduce transcription accuracy

**Current handling:**
- Rely on Whisper's built-in noise handling

**Future improvements:**
- [ ] Add noise reduction preprocessing
- [ ] Show audio quality indicator during recording
- [ ] Allow user to configure noise sensitivity
- [ ] Provide option to review/edit transcription before finalizing

### 4. What if the file is too large?
**Issue:**
- OpenAI has a 25MB file size limit
- Long recordings can exceed this limit

**Current handling:**
- Your WAV files should be fine for reasonable durations (several minutes)

**Future improvements:**
- [ ] Calculate max recording duration based on sample rate
- [ ] Show recording duration and estimated file size in UI
- [ ] Warn user when approaching file size limit
- [ ] Auto-compress audio before sending (convert to MP3?)
- [ ] Split long recordings into chunks and combine transcriptions

## Additional Edge Cases to Consider

### 5. Multiple languages in one recording
**Future improvement:**
- [ ] Allow user to specify expected language
- [ ] Use language detection
- [ ] Support multi-language transcription

### 6. Poor microphone quality
**Future improvement:**
- [ ] Test microphone quality on first use
- [ ] Recommend settings for different mic types
- [ ] Auto-adjust recording quality based on device

### 7. Slow transcription (API latency)
**Future improvement:**
- [ ] Show progress indicator
- [ ] Implement streaming transcription for real-time feedback
- [ ] Cache common phrases/responses

### 8. Privacy concerns
**Future improvement:**
- [ ] Option to use local Whisper model instead of API
- [ ] Clear indication that audio is sent to OpenAI
- [ ] Option to delete recordings after transcription
- [ ] Auto-delete old recordings

### 9. Cost management
**Future improvement:**
- [ ] Track API usage and costs
- [ ] Set monthly budget limits
- [ ] Warn when approaching budget
- [ ] Option to disable auto-transcription

### 10. Offline usage
**Future improvement:**
- [ ] Queue recordings when offline
- [ ] Auto-transcribe when connection restored
- [ ] Use local Whisper model as fallback
