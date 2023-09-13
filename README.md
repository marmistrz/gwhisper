# A speech-recognition helper using Whisper
This project's main aim is to make it easier to input text on your laptop using speech recogntion based on Whisper by OpenAI.
Note that this project is still at the proof-of-concept stage

## Building
### CPU inference
Just run 
```
cargo build --release
```
You can omit the `--release` flag while debugging.

### GPU-accelerated inference
Pass the feature flag as follows 
```
cargo build --release --features opencl
```

## Running
```
cargo run --release --bin BIN
```
where `BIN` is either `gwhisper-gtk` (for the GUI) or `gwhisper-cli` (for the CLI)