# UniversalCopilot

A Windows AI agent powered by Ollama that provides real-time ghost text suggestions and intelligent assistance as you type—anywhere on your screen.

## What It Does

UniversalCopilot is like having a personal AI assistant that watches what you're writing and offers helpful suggestions. It appears as non-intrusive "ghost text" (faint suggestions) next to your cursor, similar to GitHub Copilot or IDE autocomplete—but it works **everywhere on Windows**: emails, chat, documents, code editors, social media, etc.

### Key Capabilities

- **Always-On Assistance** — Monitors your typing in real-time across any Windows application
- **Ghost Text Overlay** — Shows AI predictions as semi-transparent text near your cursor
- **Smart Context Awareness** — Understands the active window and what you're writing
- **Chat with Context** — Open a chat window to ask questions with full screen context visible to the AI
- **Zero Cloud Dependency** — Runs entirely local using Ollama; your data never leaves your computer
- **Non-Intrusive** — Works silently in the background; dismiss suggestions with ESC or just keep typing

## How It Works (Under the Hood)

```
┌─────────────────────────────────────────────────────────────┐
│ You type in any Windows app (Gmail, Discord, VS Code, etc.) │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ↓
            ┌────────────────┐
            │ Keyboard Hook  │ ← Listens globally for typing
            └────────┬───────┘
                     │
                     ↓
          ┌──────────────────────┐
          │ Wait 400ms (debounce)│ ← User still typing?
          └────────┬─────────────┘   If yes → wait more
                   │ No more typing
                   ↓
        ┌────────────────────────┐
        │ Capture Window Context │ ← Read text before cursor
        └────────┬───────────────┘
                 │
                 ↓
      ┌──────────────────────────┐
      │ Send to Ollama LLM       │ ← "Continue this text..."
      │ (mistral model)          │
      └────────┬─────────────────┘
               │
               ↓
      ┌──────────────────────────┐
      │ Get AI Suggestion        │ ← 2-3 sentences continuation
      │ (2-3 seconds typically)  │
      └────────┬─────────────────┘
               │
               ↓
      ┌──────────────────────────┐
      │ Display Ghost Text       │ ← Semi-transparent overlay
      │ Near Cursor              │
      └────────┬─────────────────┘
               │
     ┌─────────┴──────────┐
     ↓                    ↓
  [TAB]               [ESC] or
  Insert              Dismiss
  Text                (or ignore)
```

## Installation

### Prerequisites

- **Windows 10/11** (x64 only)
- **Ollama** — Download from [ollama.com](https://ollama.com)
  - After installing, run `ollama pull mistral` to download the AI model (~4GB)
- **Administrator privileges** (for installer only; app doesn't need it after install)

### Option 1: Installer (Easiest)

1. **Download** [install.exe](../../releases) from Releases
2. **Run** the installer (admin required)
3. **Follow prompts** — it will install Ollama if missing and set up the app
4. **Start menu** → UniversalCopilot

### Option 2: Portable ZIP (No Installation)

1. **Download** [UniversalCopilot-portable.zip](../../releases) from Releases
2. **Extract** to a folder
3. **Run** `first_run.ps1` (right-click → Run with PowerShell)
4. **Done** — app starts automatically after setup

### Option 3: Build from Source

```powershell
# Clone the repo
git clone https://github.com/Superior-curtis/UniversalCopilot.git
cd UniversalCopilot

# Ensure Rust is installed (https://rustup.rs/)
# Install Ollama and pull mistral model
ollama pull mistral

# Build release binary
cargo build --release

# Run
./target/release/universal_copilot.exe
```

## Usage

### Basic Usage

1. **Open any text editor** (Notepad, VS Code, Gmail, Discord, etc.)
2. **Start typing** — the AI watches what you write
3. **Wait ~400ms after typing stops** — a ghost text suggestion appears
4. **Press TAB** to insert the suggestion, or **ESC** to dismiss
5. **Or just keep typing** — the suggestion vanishes automatically

### Keyboard Shortcuts

| Hotkey | Action |
|--------|--------|
| `Tab` | Accept and insert the overlay suggestion |
| `Esc` | Dismiss the suggestion |
| `Ctrl+Shift+C` | Toggle chat window (ask AI questions with screen context) |
| `Ctrl+Shift+S` | Show a summary overlay of the current window content |
| `##` | (Optional) Trigger inline suggestion for the current context |

### Chat Window

Press `Ctrl+Shift+C` to open the chat window. It shows:
- **"What I Can See"** — Current window title and captured text
- **Chat history** — Your questions and AI responses
- Type a message and press Enter to chat

The AI can see exactly what's on your screen, making responses highly relevant.

## Configuration

### Change the AI Model

Ollama supports many models. To use a different one:

1. **Pull the model** (e.g., `ollama pull llama2`)
2. **Edit** `src/llm.rs`:
   ```rust
   const MODEL: &str = "llama2";  // Change from "mistral"
   ```
3. **Rebuild**: `cargo build --release`

**Popular models:**
- `mistral` — Fast, good quality (default)
- `llama2` — Strong reasoning, slower
- `neural-chat` — Optimized for conversation
- `dolphin-mixtral` — Very capable, slower

### Adjust Timing

Edit `src/main.rs` to change when suggestions appear:

```rust
// Debounce: how long to wait after you stop typing
tokio::select! {
    _ = sleep(Duration::from_millis(400)) => {}  // Change 400ms here
    ...
}

// Periodic: how often to refresh if you're not typing
_ = sleep(Duration::from_millis(1200)) => {}  // Change 1200ms here
```

- **Shorter debounce** (e.g., 200ms) = faster response, but interrupts typing
- **Longer debounce** (e.g., 800ms) = less intrusive, slower to suggest
- **Shorter periodic** = refreshes more often, more CPU usage

## Architecture

```
src/
├── main.rs          — Main event loop, coordination
├── keyboard.rs      — Global keyboard hook, hotkey handlers
├── overlay.rs       — Ghost text rendering (layered window)
├── context.rs       — Active window text capture
├── llm.rs           — Ollama HTTP API client
├── chatbot.rs       — Chat UI window
├── infer.rs         — Streaming inference pipeline
├── caret.rs         — Caret position tracking
└── logger.rs        — Debug logging

installer/
├── UniversalCopilot.iss      — Inno Setup installer script
├── build.ps1                 — Build installer.exe
└── portable/
    ├── build-zip.ps1         — Create portable ZIP
    └── first_run.ps1         — Setup script for portable version
```

### Key Technologies

- **Rust** — Safe, fast system-level code
- **Windows API** — Low-level keyboard hook (WH_KEYBOARD_LL), window text capture, layered window rendering
- **Ollama** — Local LLM inference via HTTP API
- **Tokio** — Async runtime for concurrent tasks
- **Serde** — JSON parsing for Ollama responses

## Troubleshooting

### Ghost text not showing

**Check 1: Is Ollama running?**
```powershell
ollama serve
```
You should see "Listening on 127.0.0.1:11434"

**Check 2: Is mistral model downloaded?**
```powershell
ollama pull mistral
ollama list  # should show mistral:latest
```

**Check 3: Is the active window a text field?**
- Works with: Notepad, Word, Gmail, Discord, VS Code, most text inputs
- May not work with: PDF readers, image editors, some custom controls

**Check 4: Look for errors in console**
- Run app from terminal to see debug logs
- Look for "llm: generating suggestion" or error messages

### "Ollama not found at localhost:11434"

1. Download Ollama from https://ollama.com/download
2. Install and run `ollama serve` in a separate terminal
3. Restart UniversalCopilot

### Suggestions are very slow (5+ seconds)

**Reason**: Mistral model is slow on your hardware  
**Solution**: Use a faster model
```powershell
ollama pull neural-chat  # Smaller, faster
# Then change MODEL in src/llm.rs and rebuild
```

### "access denied" when installing

- Run the installer as **Administrator** (right-click → Run as administrator)
- Close any running instance of UniversalCopilot first

### App crashes on startup

Check logs for missing dependencies. Run from terminal to see full error:
```powershell
cd "C:\Program Files\UniversalCopilot"
./universal_copilot.exe
```

## Performance & Privacy

### Performance
- **Memory**: ~200MB base + LLM model size (~4-7GB for mistral)
- **CPU**: Minimal when idle, high during inference (normal for LLM)
- **Disk**: ~4GB for Ollama model cache
- **Network**: None (fully local, no telemetry)

### Privacy
- **All data stays local** — Nothing is sent to cloud servers
- **No logging of keystrokes** — The app only detects hotkeys and reads window context
- **No telemetry** — Zero data collection
- Your content is only sent to your local Ollama instance

## Uninstall

### Using Windows Control Panel

1. **Settings** → **Apps** → **Apps & Features**
2. Find **UniversalCopilot**
3. Click → **Uninstall**
4. (Optional) Uninstall Ollama the same way

### Manual Uninstall (Portable)

Just delete the extracted folder.

## Future Roadmap

- [ ] GUI settings window for configuration
- [ ] More LLM backends (LM Studio, vLLM, Text Generation WebUI)
- [ ] Custom prompt templates for different use cases
- [ ] Code-specific suggestions mode
- [ ] Multi-language support
- [ ] Keyboard shortcut customization
- [ ] Model auto-download and management UI

## Limitations

- **Windows only** (no Mac/Linux support currently)
- **Text-field only** — Won't work in all custom UI controls
- **Single GPU** — Uses whatever GPU Ollama can access
- **English focus** — Default prompts in English (configurable)

## Contributing

We welcome contributions! Ways to help:

1. **Bug reports** — Open an issue on GitHub
2. **Feature requests** — Describe what you'd like to see
3. **Code improvements** — Fork, improve, submit a PR
4. **Documentation** — Help improve this README or add examples

## Building & Deployment

### Build Installer

Requires [Inno Setup 6](https://jrsoftware.org/isinfo.php) to be installed:

```powershell
cd installer
./build.ps1
# Output: installer/dist/install.exe
```

### Build Portable ZIP

```powershell
cd installer/portable
./build-zip.ps1
# Output: installer/portable/dist/UniversalCopilot-portable.zip
```

### Cross-Compile from Linux/Mac (Future)

Not currently supported, but contributions welcome!

## License

MIT License — See [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Ollama](https://ollama.ai) — Fantastic local LLM platform
- [Mistral AI](https://mistral.ai) — Great open-source models
- [Tokio](https://tokio.rs) — Excellent async Rust runtime
- Windows API documentation and community examples

## Support & Contact

- **GitHub Issues** — Report bugs or request features: https://github.com/Superior-curtis/UniversalCopilot/issues
- **Discussions** — Ask questions: https://github.com/Superior-curtis/UniversalCopilot/discussions

---

**Made with ❤️ using Rust and Windows API**

**Questions? Open an issue on GitHub!**

  - Single suggestion at a time
  - Inline ghost text only
  - Feels like an extension of user's thoughts

## Architecture

**Modules:**
- `main.rs`: Orchestrates overlay, caret poller, keyboard hook, and inference loop
- `overlay.rs`: Transparent topmost window rendering ghost text (Windows UI)
- `caret.rs`: Polls foreground window caret position
- `keyboard.rs`: Global keyboard hook for TAB/ESC/typing notifications
- `context.rs`: Captures text before caret from focused control (context for inference)
- `infer.rs`: Streaming OpenAI-compatible API client with truncation heuristics

## Build Requirements

**Windows only:**
- Rust 1.70+
- MSVC C++ Build Tools 2019 or later (with C++ workload)
  - Includes: `link.exe` (linker), `cl.exe` (compiler), Windows SDK

## Setup & Build

### 1. Install Rust (if not already installed)
```powershell
# Download and run rustup installer
Invoke-WebRequest -Uri https://win.rustup.rs -OutFile rustup-init.exe
.\rustup-init.exe -y
```

### 2. Install Visual C++ Build Tools
**Option A: Full Visual Studio Community**
- Download: https://visualstudio.microsoft.com/downloads/
- Select "Visual Studio Community"
- During install, check **Desktop development with C++** workload
- Install

**Option B: Build Tools only**
- Download: https://visualstudio.microsoft.com/downloads/
- Select "Build Tools for Visual Studio"
- During install, check **C++ build tools** workload
- Install

**Verify the install:**
```powershell
# In a new PowerShell session
where link.exe
where cl.exe
```

Both commands should return paths (not "not found").

### 3. Build
```powershell
cd "C:\Users\huach\Downloads\Code\UniversalCopilot"
cargo build
```

**Release build (optimized):**
```powershell
cargo build --release
```

### 4. Run
```powershell
# Set API credentials (using OpenAI API as example)
$env:VITE_OPENAI_API_KEY = "sk-..."
$env:VITE_OPENAI_API_URL = "https://api.openai.com/v1/chat/completions"

# Run
.\target\debug\universal_copilot.exe
```

Then open Notepad and start typing — ghost text suggestions should appear inline at the caret!

## Configuration

Edit `src/infer.rs` constants to tune behavior:

```rust
pub const MAX_SUGGESTION_CHARS: usize = 200;    // max ghost text length
pub const MAX_TOKENS: usize = 64;               // max tokens per stream
pub const MAX_IDLE_MS_BETWEEN_TOKENS: u64 = 400; // timeout before stop
```

## Environment Variables

- `VITE_OPENAI_API_KEY`: OpenAI API key (or compatible provider)
- `VITE_OPENAI_API_URL`: API endpoint (e.g., `https://api.openai.com/v1/chat/completions`)

If not set, the system uses a placeholder suggestion for testing.

## Limitations (Phase A)

- Windows only (macOS support deferred)
- No installer or autostart
- No settings UI
- Tested on standard edit controls; some custom apps may not work
- Suggestions sent to remote API (no local model support yet)
- No multi-suggestion ranking
- Ghost text may not align perfectly on all DPI scales (fixable via font metrics)

## Next Steps (Phase B+)

- Cross-platform support (macOS, Linux)
- Installer with autostart and background service
- Settings UI (model selection, sensitivity, excluded apps)
- Local inference support (llama.cpp integration)
- Advanced model tuning and ranking
- Accessibility & permissions flow
- Code signing and distribution

## Debugging

If the build fails:

1. **Verify MSVC is installed:**
   ```powershell
   where link.exe
   ```
   If not found, reinstall Visual Studio / Build Tools with C++ workload.

2. **Clean and rebuild:**
   ```powershell
   cd "C:\Users\huach\Downloads\Code\UniversalCopilot"
   cargo clean
   cargo build
   ```

3. **Check Rust version:**
   ```powershell
   rustc --version
   cargo --version
   ```
   Should be 1.70+.

## Code Structure

```
src/
├── main.rs              # Entry point, task orchestration
├── overlay.rs           # Transparent overlay rendering
├── caret.rs             # Caret position polling
├── keyboard.rs          # Global keyboard hook
├── context.rs           # Active control text capture
└── infer.rs             # Streaming inference + heuristics
```

## License & Attribution

This is a prototype implementation of a system-wide writing assistant inspired by GitHub Copilot's inline suggestion UX.

## Troubleshooting

**Q: "linker `link.exe` not found"**  
A: Install Visual C++ Build Tools or Visual Studio with the C++ workload.

**Q: Suggestions don't appear**  
A: Check that `VITE_OPENAI_API_KEY` is set correctly; keyboard hook may need elevation (restart as Admin).

**Q: Ghost text alignment is off**  
A: This is a known limitation in Phase A. Font metrics and DPI scaling will be improved in Phase B.

**Q: Program crashes on TAB/ESC**  
A: Likely an issue with `SendInput` on your system. Keyboard hook may need different encoding or timing. File an issue with OS info.

## Development

For local iteration:
```powershell
# Watch and rebuild on changes
cargo watch -x build

# Run with verbose output
RUST_LOG=debug cargo run
```

Enjoy using Universal Copilot! 🚀
