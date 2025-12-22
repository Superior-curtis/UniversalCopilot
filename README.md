# UniversalCopilot

A Windows AI agent powered by Ollama that provides real-time ghost text suggestions and intelligent assistance as you type—anywhere on your screen.

## Features

- **Always-On AI Assistant**: Continuously monitors and provides suggestions as you type in any Windows application
- **Ghost Text Overlay**: Non-intrusive overlay shows AI predictions near your cursor
- **Keyboard Shortcuts**:
  - `Ctrl+Shift+C` — Toggle chat window for deeper AI conversations with screen context
  - `Ctrl+Shift+S` — Show a concise summary overlay of current window content
  - `TAB` — Accept overlay suggestion and insert it
  - `ESC` — Dismiss overlay suggestion
- **Context-Aware**: Understands what you're typing and the current window context
- **Local LLM**: Runs entirely on Ollama (mistral model by default); no cloud dependency
- **Multiple Deployment Options**: Installer, portable ZIP, or direct build

## Requirements

- **Windows 10/11** (x64)
- **Ollama** — Download from [ollama.com](https://ollama.com)
- **Rust** (only if building from source)

## Quick Start

### Option 1: Installer (Recommended)

```powershell
# Download or build install.exe from installer/dist/
# Run install.exe and follow prompts
# Ollama will be installed silently if bundled
```

### Option 2: Portable ZIP

```powershell
# Download or build UniversalCopilot-portable.zip from installer/portable/dist/
# Extract and run:
./first_run.ps1
# This will install Ollama (if bundled), prepare the model, and start the app
```

### Option 3: Build from Source

```powershell
# Clone repo
git clone https://github.com/yourusername/UniversalCopilot.git
cd UniversalCopilot

# Ensure Ollama is running
ollama serve &

# Build and run
cargo build --release
./target/release/universal_copilot.exe
```

## How It Works

1. **Continuous Monitoring** — The app listens to all keyboard input globally
2. **Context Capture** — Every ~1.2 seconds (or on typing pause), it captures the active window text
3. **AI Generation** — Sends context to Ollama and generates 2–3 sentence continuations
4. **Ghost Text Display** — Shows suggestion in an overlay near your cursor
5. **User Control** — Press TAB to accept, ESC to dismiss, or keep typing to ignore

## Configuration

### Change the AI Model

Edit `src/llm.rs` and change `MODEL`:

```rust
const MODEL: &str = "mistral:latest";  // Change to "llama2", "neural-chat", etc.
```

### Adjust Response Timing

Edit `src/main.rs` inference loop:
- **Debounce**: 400ms (how long after typing stops before generating)
- **Periodic refresh**: 1.2 seconds (how often to check if idle)

## Deployment

### Building the Installer

```powershell
# Requires Inno Setup 6 (https://jrsoftware.org/isinfo.php)
cd installer
./build.ps1
# Output: installer/dist/install.exe
```

### Building the Portable ZIP

```powershell
cd installer/portable
./build-zip.ps1
# Output: installer/portable/dist/UniversalCopilot-portable.zip
```

## Architecture

- **`src/main.rs`** — Main event loop, inference orchestration
- **`src/keyboard.rs`** — Low-level keyboard hook (WH_KEYBOARD_LL), hotkey handlers
- **`src/overlay.rs`** — Layered window rendering, ghost text display
- **`src/context.rs`** — Active window text capture via WinAPI
- **`src/llm.rs`** — Ollama HTTP client (sync and async)
- **`src/chatbot.rs`** — Chat UI window with screen/text context preview
- **`src/infer.rs`** — Streaming LLM inference pipeline

## Keyboard Hook Safety

The app uses `WH_KEYBOARD_LL` (low-level keyboard hook) which is Windows-standard for global hotkey capture. It does NOT log keystrokes; it only detects hotkey patterns and passes all other input through immediately.

## Troubleshooting

### Ghost text not appearing
- Ensure Ollama is running: `ollama serve`
- Check that the active window is a text field (not all controls expose text)
- Verify model is pulled: `ollama pull mistral`
- Check logs for errors

### "Ollama not found" error
- Install Ollama from https://ollama.com/download
- Run `ollama serve` in a terminal
- Restart UniversalCopilot

### Installer fails with "access denied"
- Run as Administrator
- Ensure no other copy of the app is running

## Future Enhancements

- [ ] Support for more LLM backends (LM Studio, vLLM, etc.)
- [ ] Custom prompt templates
- [ ] Multi-language support
- [ ] Code completion mode
- [ ] Keyboard shortcut customization UI
- [ ] Settings window

## License

MIT

## Contributing

Contributions welcome! Please open an issue or submit a PR.

---

**Developed with ❤️ using Rust and Windows API**
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
