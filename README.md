<sup>[English](./README.md) | [中文](./README_CN.md)</sup>

# Mango Desk

**🥭 Awake your data**  

![image](./screenshots/work.png)

[![Download](./screenshots/download-ready.svg)](https://github.com/moyangzhan/mango-desk/releases)

## What is Mango Desk?

Mango Desk is a local-first desktop app for searching your local documents with natural language.

It helps you find information based on what you remember, not file names or folder structures.

![search](./screenshots/search.gif)

### 📌 Use Cases

Mango Desk is especially useful in scenarios where you have **a large amount of local documents** and want to retrieve information using natural language.


#### Typical Use Cases

- 📝 **Personal Document Libraries**
  - Years of accumulated notes, PDFs, Word files, Markdown files. etc
  - Example: *“That note where I summarized Rust ownership rules”*

- 📂 **SVN / Git Repositories**
  - Search through design docs, READMEs, technical proposals, and historical solutions
  - Example: *“Where is the document about the permission refactor?”*

- 🏢 **Team or Company Knowledge Base**
  - Internal documents, project docs, meeting notes, onboarding materials
  - Example: *"Find all Q4 meeting notes about budget planning"*
  - Example: *"What are the company policies regarding remote work?"*

- 📚 **Research and Study Materials**
  - Papers, experiment records, literature notes
  - Example: *“What is the latest research on AI?”*

- ⚖️ **Legal and Financial Documents**
  - Contracts, policy documents, reports
  - Example: *“What is the latest company policy on data privacy?”*

### ✨ Features

- 💭 **Search by meaning, not file names**
  - Find documents by describing what you remember, even if you don’t recall exact titles or locations

- 📍 **Search by path**
  - Find documents by multiple keywords if you remember some pecific parts of the file path

- 📂 **Works with your existing local files**
  - No need to reorganize folders or rename files — Mango Desk works with what you already have

- 👀 **Real-time file & directory watching**
  - Automatically detects file and folder changes (add / modify / delete) and keeps index and search results up to date

- ⚡ **Fast and lightweight**
  - Instant search results without slowing down your system

- 🔒 **Private by default**
  - All data stays on your device, ensuring your privacy

### 🏗️ Architecture

**Indexing**

![indexing](./screenshots/mango-desk-indexing.png)

`The self-hosted model part is under development and will be integrated according to the ollama interface.`

**Search**

![search](./screenshots/mango-desk-search.png)

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
`rust` v1.90.0+ required

Install tools: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### 3. Tauri

Install Tauri Prerequisites: 
[https://tauri.app/start/prerequisites/](https://tauri.app/start/prerequisites/)

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

windows: `src-tauri/target/release/bundle/msi/Mango Desk_0.1.0_x64_en-US.msi`

## ❓ FAQ
### Q: How does Mango Desk ensure data privacy?

A: Mango Desk follows a local-first architecture to ensure data privacy:

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
- `all-minilm-l6-v2` or `paraphrase-multilingual-MiniLM-L12-v2`
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

Support Mango Desk if you find it helpful:
- Starring the repository on GitHub
- Recommending it to others
- Sharing your experience