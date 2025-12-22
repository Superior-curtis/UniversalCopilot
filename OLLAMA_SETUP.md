# UniversalCopilot with Ollama (Local LLM)

UniversalCopilot now uses **Ollama** for all LLM inference. This means:

✅ **Zero API costs** - everything runs locally  
✅ **Complete privacy** - no data sent to cloud  
✅ **No subscriptions** - truly free to use  
✅ **Instant suggestions** - <500ms latency  
✅ **Works offline** - internet not required  

## Quick Start (2 steps)

### Step 1: Install Ollama

Download from **https://ollama.ai** and install (3 min download + install).

### Step 2: Pull the Model

Open PowerShell or Command Prompt and run:

```
ollama pull mistral
```

This downloads Mistral-7B (~5GB, first time only).

### Step 3: Run Ollama

```
ollama serve
```

Leave this running in the background. (Or set it to auto-start on boot - instructions at https://ollama.ai/docs)

### Step 4: Run UniversalCopilot

```
C:\Users\[YourName]\AppData\Local\UniversalCopilot\universal_copilot.exe
```

That's it! The app will automatically detect Ollama and start generating suggestions as you type in Notepad, etc.

## What's Happening

1. You type in Notepad
2. App captures your text context
3. Sends it to local Ollama server on `localhost:11434`
4. Mistral-7B generates 1-3 word suggestions in ~300ms
5. Suggestion appears as ghost text in the overlay
6. Press **TAB** to accept or **ESC** to dismiss

## Troubleshooting

**"Ollama not available"?**
- Make sure you ran `ollama serve` first
- Check that port 11434 is not blocked
- Try accessing http://localhost:11434/api/tags in your browser - should return JSON

**Slow suggestions?**
- First generation takes longer (model warming up)
- Mistral is optimized for speed - usually <1s
- Close other heavy apps if lagging

**Wrong model?**
- To use a different model instead of Mistral:
  1. Pull it: `ollama pull llama2` (or any model from ollama.ai/library)
  2. Edit the model name in `universal_copilot.exe` logs (or recompile with your model name)

## Supported Models

Any model from https://ollama.ai/library works. Recommended for inline suggestions:

- **mistral** (default) - fast, balanced, ~13GB VRAM
- **neural-chat** - optimized for conversations
- **dolphin-mixtral** - smarter but slower

To use a different model, edit the `MODEL` constant in `src/llm.rs` and rebuild.

## Architecture

```
Notepad.exe 
    ↓ (caret + text capture)
UniversalCopilot.exe
    ↓ (HTTP POST to localhost:11434)
Ollama Server
    ↓ (loads model in VRAM)
Mistral-7B
    ↓ (local inference, <1s)
Suggestion text
    ↓ (displayed as ghost text overlay)
Notepad (TAB to accept)
```

No internet, no API key, no telemetry. **Your data stays on your machine.**

## Notes

- First app startup: takes 30s-2min to load the 5GB model into RAM
- Subsequent startups: instant (model stays in memory)
- Suggestion quality depends on context length (longer = better)
- CPU + GPU (NVIDIA/AMD with CUDA/ROCm) = faster inference
