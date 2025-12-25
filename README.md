<sup>[English](./README.md) | [中文](./README_CN.md)</sup>

# Mango Desk

**🥭 Awake your data**  

![image](./screenshots/work.png)

## What is Mango Desk?
Mango Desk is a desktop application that helps you search your data using natural language.

Feel free to use this project, star the repo, provide feedback, or contribute ❤️

### ✨ Features

* 🧠 Query data using natural language
* 🖥️ Cross-platform desktop application (based on Tauri)
* ⚡ Rust backend, high performance with low resource usage
* 🔒 Local-first approach, data never leaves your device

### 🛠 Technology Stack

* Frontend
  * WebView（Tauri）
  * PNPM
  * Node.js
* Backend
  * Rust
  * Tauri Core

## Setting Up

### 1️⃣ Frontend
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

### 2️⃣ Backend(Rust)
`rust` v1.90.0+ required

Install tools: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### 3️⃣ Tauri

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
2. Create a new branch for your changes
3. Keep commit messages clear and readable
4. Make sure `pnpm tauri dev` runs successfully in your local environment

