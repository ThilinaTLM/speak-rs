# speak-rs

A powerful desktop application for real-time speech transcription using Rust and Whisper AI. This application provides high-quality speech-to-text conversion with GPU acceleration support and a modern user interface.

## Features

- **Real-time Transcription**: Convert speech to text in real-time using your system's microphone
- **GPU Acceleration**: CUDA support for faster transcription processing
- **Automatic Clipboard Integration**: Automatically copy transcribed text to clipboard
- **Configurable Settings**: Easy customization through TOML configuration
- **Stop Phrase Detection**: Automatically stop transcription when a specific phrase is detected
- **Modern UI**: Built with Slint for a native and responsive user experience
- **Recording Duration Display**: Real-time display of recording duration in minutes and seconds
- **Transcription Status Indicator**: Visual feedback when transcription is in progress
- **Clean Interface**: Simple record/stop button with clear transcription display
- **Error Handling**: Robust error handling with user-friendly error messages
- **Resource Management**: Automatic cleanup of resources on application exit

## Prerequisites

- Rust (latest stable version)
- CUDA toolkit (for GPU acceleration)
- System with a microphone
- Linux with Wayland support (current implementation)

## Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/ThilinaTLM/speak-rs.git
   cd speak-rs
   ```

2. Download the Whisper model:

   - Create a `models` directory
   - Download the desired model (e.g., `ggml-small.en.bin`) into the `models` directory

3. Build the application:
   ```bash
   cargo build --release
   ```

## Configuration

The application can be configured through `config.toml`:

```toml
[whisper]
model_path = "models/ggml-small.en.bin"  # Path to Whisper model
use_gpu = true                           # Enable GPU acceleration
language = "en"                          # Target language
audio_context = 768                      # Audio context size
no_speech_threshold = 0.5                # Threshold for no speech detection
num_threads = 2                          # Number of CPU threads to use

[behavior]
realtime_transcribe = true               # Enable real-time transcription
auto_copy = true                         # Automatically copy text to clipboard
stop_phrase_enabled = true               # Enable stop phrase detection
stop_phrase_pattern = "(?i)that'?s all\\.?$"  # Regex pattern for stop phrase
```

## Usage

1. Run the application:

   ```bash
   cargo run --release
   ```

2. The application will start with a GUI interface
3. Click the record button to start transcription
4. Speak into your microphone
5. The transcribed text will appear in real-time
6. Monitor recording duration in the interface
7. Use the configured stop phrase (default: "that's all") to automatically stop recording
8. Transcribed text is automatically copied to your clipboard if enabled
9. Click the record button again to stop manually, or use the close button to exit

## Features in Detail

### Real-time Transcription

- Continuous transcription during recording
- Updates every few seconds with new content
- Visual indicator when transcription is in progress

### Recording Controls

- Simple record/stop toggle button
- Duration display showing minutes and seconds
- Automatic resource cleanup on stop

### Text Processing

- Automatic removal of stop phrases from final text
- Smart handling of partial transcriptions
- Clipboard integration for easy text sharing

### Error Handling

- Graceful handling of transcription errors
- User-friendly error messages
- Automatic recovery from common issues

## Dependencies

- `whisper-rs`: Rust bindings for OpenAI's Whisper model
- `cpal`: Cross-platform audio library
- `slint`: Modern UI framework
- `rubato`: Audio resampling
- `clap`: Command line argument parsing
- `config`: Configuration management
- And more (see `Cargo.toml` for full list)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- OpenAI for the Whisper model
- The Rust community for excellent libraries and tools
