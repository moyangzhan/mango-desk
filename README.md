<sup>[English](./README.md) | [中文](./README_CN.md)</sup>

# Mango Finder

**🥭 Awake your data**  

![image](./screenshots/work.png)

[![Download](./screenshots/download-ready.svg)](https://github.com/moyangzhan/mango-finder/releases)

## What is Mango Finder?

Mango Finder (formerly MangoDesk) is a local-first desktop app for searching your local documents with natural language.

It helps you find information based on what you remember, not file names or folder structures.

![search](./screenshots/search.gif)

### 📌 Use Cases

Mango Finder is especially useful in scenarios where you have **a large amount of local documents** and want to retrieve information using natural language.


#### Typical Use Cases

- 📝 **Personal Document Libraries**
  - Years of accumulated notes, PDFs, Word files, Markdown files. etc
  - Example: *“that note about how rust handles memory ownership”*

- 📂 **SVN / Git Repositories**
  - Search through design docs, READMEs, technical proposals, and historical solutions
  - Example: *“the solution we used for the permission system refactor”*

- 🏢 **Team or Company Knowledge Base**
  - Internal documents, project docs, meeting notes, onboarding materials
  - Example: *"Q4 budget planning and team feedback from last year"*

- 📚 **Research and Study Materials**
  - Papers, experiment records, literature notes
  - Example: *“recent breakthroughs in large language model efficiency”*

- ⚖️ **Legal and Financial Documents**
  - Contracts, policy documents, reports
  - Example: *“clauses regarding data privacy and user consent”*

### ✨ Features

- 💭 **Search by meaning**
  - Find documents by describing what you remember, even if you don’t recall exact titles or locations

- 📍 **Exact Keyword Match**
  - Instantly locate files using precise terms from file paths or content, ideal for finding specific phrases or technical strings.

- 🔍 **Find Similar Files**
  - Find visually similar images using perceptual hashing, semantically similar documents, or audio files with matching content
  - One click to discover related files based on visual, semantic, or audio fingerprint similarity

- 🌐 **Multilingual & Cross-language Search**
  - Search across **100+ languages** seamlessly. Find English documents using Chinese queries, or vice versa, with zero configuration required

- 🔒 **Private by default**
  - All data stays on your device, ensuring your privacy

- 🖥️ **Self-Hosted Model Support**
  - Integration with **Ollama** and **vLLM** for image analysis using local vision models (e.g., LLaVA)
  - Keep your data completely private by running vision models on your own hardware

- ⚡ **Fast and efficient**
  - Instant search results with optimized indexing system

- 👀 **Real-time file & directory watching**
  - Automatically detects file and folder changes (add / modify / delete) and keeps index and search results up to date

- 📂 **Works with your existing local files**
  - No need to reorganize folders or rename files — Mango Finder works with what you already have

### 🏗️ Architecture

**Indexing**

![indexing](./screenshots/indexing.png)

Supports three processing modes: **Local** (fully offline), **Self-Hosted** (Ollama/vLLM), and **Cloud** (remote AI services).

**Search**

![search](./screenshots/search.png)

### 🛠️ Technology Stack

* Frontend
  * WebView（Tauri）
  * PNPM
  * Node.js
* Backend
  * Rust
  * Tauri Core

## 🚀 Setting Up

### 1. Frontend
#### Node
`node` v20+ required

It is recommended to use [nvm](https://github.com/nvm-sh/nvm) to manage multiple `node` versions.

#### PNPM

`pnpm` v9+ required

If you haven't installed `pnpm`, you can install it with the following command:

```shell
npm install pnpm -g
```

#### Install dependencies

```sh
pnpm i
```

### 2. Backend(Rust)
`rust` v1.94.0+ required

Install tools: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### 3. Tauri

Install Tauri Prerequisites:
[https://tauri.app/start/prerequisites/](https://tauri.app/start/prerequisites/)

### 4. Whisper.cpp Dependencies

The audio transcription feature uses [whisper.cpp](https://github.com/ggerganov/whisper.cpp). Different operating systems require different dependencies.

#### Windows

Compiling on Windows requires **CMake** and **LLVM/Clang 18** (Note: LLVM 19/20/22 have compatibility issues, please use LLVM 18).

1. **Install CMake 4.3**

   Download from [cmake-4.3.0](https://github.com/Kitware/CMake/releases/tag/v4.3.0-rc2)

2. **Download and Install LLVM 18**
   - Visit [LLVM 18.1.8 Release](https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8)
   - Download `LLVM-18.1.8-win64.exe`
   - Check **"Add LLVM to the system PATH for all users"** during installation

3. **Verify installation**
   ```sh
   cmake --version
   clang --version
   ```
   The clang version should show `18.1.8`

4. **Set environment variables (permanent)**
   - Press `Win + R`, type `sysdm.cpl`, press Enter
   - Click **"Advanced"** tab → **"Environment Variables"**
   - Under **"User variables"**, click **"New"** and add:

   | Variable name | Value |
   |---------------|-------|
   | `CXXFLAGS` | `/utf-8` |
   | `CFLAGS` | `/utf-8` |

   - Click OK and **restart your terminal** for changes to take effect

5. **Build the project (first time only)**

   Open **"x64 Native Tools Command Prompt for VS 2022"** (search from Start Menu), then build:
   ```cmd
   cd your-project-path\src-tauri
   cargo build
   ```

   > ⚠️ **Important Notes**:
   > - The `/utf-8` flag is required to resolve encoding issues
   > - If previous build failed, run `cargo clean -p whisper-rs-sys` to clear cache first
   > - After whisper is compiled successfully, subsequent builds can use `pnpm tauri dev` directly in any terminal
   > - VSCode's rust-analyzer plugin auto-checks code on startup. Without MSVC environment, whisper-rs-sys build will fail and show as red in `target/debug/build` directory. If you've successfully built in "x64 Native Tools Command Prompt for VS 2022", you can ignore this error

#### macOS

macOS usually has Clang built-in. If you encounter issues, install Xcode Command Line Tools:

```sh
xcode-select --install
```

#### Linux

Most Linux distributions require C/C++ build tools:

**Ubuntu/Debian:**
```sh
sudo apt update
sudo apt install build-essential cmake
```

**Fedora/RHEL:**
```sh
sudo dnf install gcc-c++ make cmake
```

**Arch Linux:**
```sh
sudo pacman -S base-devel cmake
```

### 5. Download Model Files

Download the required model files from one of the following sources:

1. **GitHub Release**: [model.zip](https://github.com/moyangzhan/mango-finder/releases/download/v0.1.0/model.zip) - Contains all required files
2. **Hugging Face**: [moyangzhan/mango-finder](https://huggingface.co/moyangzhan/mango-finder/tree/main) - Manually download the following files:
   - *.onnx model files
   - *_tokenizer.json tokenizer files
   - whisper-small-q8_0.bin

After downloading, extract the files to the `src-tauri/assets/model` directory.

**Required Files**:
- embedding.onnx
- embedding_tokenizer.json
- vision.onnx
- vision_tokenizer.json
- whisper-small-q8_0.bin

## 🚀 Getting Started

## ▶️ Development Run

A Tauri app has at least [two processes](https://tauri.app/concept/process-model/):

- the Core Process (`backend`)
- the WebView process (`frontend`)

Both backend and frontend start with a single command:

```sh
pnpm tauri dev
```

## 📦 Building


```sh
pnpm tauri build
```

After building, the executable file is usually located in `src-tauri/target/release/`.

windows: `src-tauri/target/release/bundle/msi/Mango Finder_0.1.0_x64_en-US.msi`

## ❓ FAQ
### Q: How does Mango Finder ensure data privacy?

A: Mango Finder follows a local-first architecture to ensure data privacy:

#### Local Data Processing
- All document indexing and search operations are performed locally on your device
- No data is transmitted to external servers during normal operation

#### Exception Cases
- Only when processing images or audio files, remote models may be used (if enabled)
- These remote models are disabled by default and must be manually enabled by users

#### Data storage
- All user data remains on the local device by default

#### Architecture Details
As shown in the architecture diagram above, the entire processing pipeline is designed to keep data local, ensuring maximum privacy and security.

### Q: Why are so many models used in the code?

A: The codebase includes multiple models serving different purposes:

#### 1. Active Local Models (Enabled by Default)
- `src-tauri/assets/model/*`
- These models run locally on users' computers for basic document processing
- Prioritized for privacy and performance

#### 2. Remote Models (Optional)
- `gpt-5-mini` and `gpt-4o-mini-transcribe`
- Designed for image and audio parsing
- Disabled by default, can be enabled if needed
- Note: We plan to replace these with local alternatives when available
- Kept as optional features for self-hosting scenarios

#### 3. Reserved Models (Future Features)
- `qwen-turbo`, `deepseek-chat`, and `deepseek-reasoner`
- Prepared for upcoming features like:
  - Knowledge graph generation
  - Advanced document analysis
- Also serves as a foundation for developers who want to customize with these models
- Maintains flexibility for future feature expansion

## 📝 License

see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions of all kinds are welcome, including but not limited to:
* 🐛 Reporting bugs
* 💡 Suggesting new features or improvements
* 📖 Improving documentation
* 🔧 Submitting code (pull requests)

Before submitting a pull request, please consider:
1. Fork this repository
1. Create a new branch (git checkout -b feature/xxx)
1. Ensure pnpm tauri dev runs successfully locally
1. Commit changes (git commit -m 'feat: xxx')
1. Push the branch (git push origin feature/xxx)
1. Submit a Pull Request

## ⭐ Support the Project

Support Mango Finder if you find it helpful:
- Starring the repository on GitHub
- Recommending it to others
- Sharing your experience