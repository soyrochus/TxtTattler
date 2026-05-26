# TxtTattler 🦀🗣️

> Your documents have never sounded so dramatic.

A fast, cross-platform, pure-Rust command-line text-to-speech tool that reads your files out loud using OpenAI's (or Azure OpenAI's) TTS models.

```
txttattler chapter-7.txt --voice fable --speed 0.9
```

## Features (MVP)

- **Beautiful modern CLI** powered by `clap` v4 with colors, help, and fun personality
- **Clean hexagonal architecture** — adding `.docx` or `.pdf` support later requires *zero* changes to the core logic
- Full support for **OpenAI** and **Azure OpenAI**
- Intelligent text chunking (respects the ~4096 char TTS limit while preserving sentences)
- Cross-platform audio playback via `rodio`
- Save the generated MP3 with `--output`
- TOML config + environment variable overrides
- Progress bars, structured logging (`-V`), excellent errors

## Installation

```bash
git clone https://github.com/yourname/txttattler
cd txttattler/grok
cargo build --release
# binary lands at target/release/txttattler
```

Or install directly (once published):

```bash
cargo install txttattler
```

## Quick Start

```bash
# 1. Set your key
export OPENAI_API_KEY=sk-...

# 2. Tattle!
txttattler my-notes.txt

# With personality
txttattler novel.md --voice fable --speed 0.85

# Save to file (no speakers)
txttattler report.txt --output report.mp3 --no-play

# Force Azure
txttattler board-minutes.txt --azure
```

## CLI Reference

```
txttattler <FILE>

Options:
  -v, --voice <VOICE>     alloy|echo|fable|onyx|nova|shimmer (default: alloy)
  -m, --model <MODEL>     tts-1 | tts-1-hd (default: tts-1)
  -s, --speed <FLOAT>     0.25–4.0 (default: 1.0)
  -o, --output <PATH>     Save MP3 instead of (or with) playing
      --no-play           Generate but do not play audio
      --azure             Force Azure OpenAI
      --config <PATH>     Custom TOML config
      --list-voices       Show all voices with descriptions
  -V, --verbose           More logging (repeat for trace)
```

## Configuration

### Environment variables via .env (recommended for development)

TxtTattler automatically loads a `.env` file from your **current working directory** if one is present. This is the easiest way to manage your API key:

```bash
# In your project directory (or wherever you run txttattler from)
echo 'OPENAI_API_KEY=sk-...' > .env
txttattler document.txt
```

Supported variables:
- `OPENAI_API_KEY`
- `AZURE_OPENAI_API_KEY`, `AZURE_OPENAI_ENDPOINT`, etc.
- `TXTTATTLER_*` overrides (see below)

The file is loaded **very early** — before CLI parsing or any other config.

### MP3 Caching (the tattler remembers)

By default, TxtTattler saves every synthesized result to a content-addressed cache:

- Linux: `~/.cache/txttattler/`
- macOS: `~/Library/Caches/txttattler/`
- Windows: `%LOCALAPPDATA%\txttattler\`

The cache key includes the **exact cleaned text**, voice, model, speed, and an internal processing version. This means:

- Re-running the exact same file with the same settings is **instant** and makes **zero** OpenAI calls.
- Changing even one word, the voice, or the speed automatically generates a new cache entry.

Useful flags:

```bash
txttattler long-doc.txt --refresh          # Regenerate and replace the cached version
txttattler long-doc.txt --no-cache         # Skip cache completely (one-off run)
txttattler long-doc.txt --cache-dir ./my-cache   # Use a custom location
```

When a cache hit occurs you will see a friendly `💾 Cache hit!` message.

### TOML configuration file

TxtTattler also looks for `~/.config/txttattler/config.toml` (or `%APPDATA%\txttattler\config.toml` on Windows).

Example:

```toml
[tts]
voice = "fable"
model = "tts-1"
speed = 0.95

[openai]
api_key = "sk-..."          # or rely on OPENAI_API_KEY env

[azure]
azure_endpoint = "https://yourname.openai.azure.com/"
api_key = "..."
```

Environment variables always win over the file (except when `--config` is explicit).

## Architecture

TxtTattler follows **ports & adapters** (hexagonal) architecture:

```
src/
├── domain/
│   ├── entities.rs      # Voice, TtsOptions, ProcessedDocument, chunking logic
│   └── ports.rs         # FileReader, TtsProvider, AudioPlayer, TextToSpeechService traits
├── application/
│   └── text_to_speech.rs# The orchestrator (the "use case")
├── infrastructure/
│   ├── file_readers/
│   │   ├── txt.rs       # Current: UTF-8 + BOM + cleaning
│   │   ├── docx.rs      # Placeholder (easy to implement)
│   │   └── pdf.rs       # Placeholder
│   ├── tts/openai.rs    # async-openai (OpenAI + Azure)
│   └── audio/player.rs  # rodio MP3 playback
├── cli.rs               # clap definition + banner
├── config.rs            # TOML + env merging
├── adapters.rs          # Thin re-exports / DI spot
└── main.rs
```

**Why this matters**: The use-case (`TextToSpeechOrchestrator`) depends only on traits. Swapping the TTS backend or adding a new file format never touches `main.rs` or the business logic.

## Adding a New File Reader (example: DOCX)

1. Add the crate to `Cargo.toml`
2. Implement `FileReader` for your type in `infrastructure/file_readers/docx.rs`
3. Register it in one line inside `file_readers/mod.rs`:

```rust
reg.register(Box::new(docx::DocxReader));
```

Done. The CLI and core never change.

## Development

```bash
cargo run -- myfile.txt -V
cargo test
cargo build --release
```

## Personality

The comments in the source are deliberately cheeky. The tool is called **TxtTattler** for a reason — it's a little gossip that loves reading your private thoughts out loud. Use responsibly.

## License

MIT / Apache-2.0

---

Made with 🦀 and too much curiosity about what your files would sound like.
