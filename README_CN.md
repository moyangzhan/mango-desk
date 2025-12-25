<sup>[English](./README.md) | [中文](./README_CN.md)</sup>

# Mango Desk

**🥭 Awake your data**

![1691585544443](./screenshots/work.png)

## 📖 项目简介

Mango Desk 是一个基于 Tauri + Rust + Web 前端的桌面应用，它允许你使用自然语言来搜索你的本地数据。

欢迎使用、🌟点赞、反馈以及参与贡献 ❤️

### ✨ 特性

* 🧠 使用自然语言查询数据
* 🖥️ 跨平台桌面应用（基于 Tauri）
* ⚡ Rust 后端，高性能、低资源占用
* 🔒 本地优先，数据不离开你的设备

### 🛠 技术栈

* Frontend
  * WebView（Tauri）
  * PNPM
  * Node.js
* Backend
  * Rust
  * Tauri Core

## 🚀 快速开始（开发环境）

### 1️⃣ 前端环境准备

#### Node

``node` **v20 及以上版本**

推荐使用 [nvm](https://github.com/nvm-sh/nvm) 来管理多个 `node` 版本。

#### PNPM

需要 `pnpm` **v9 及以上版本**

如果你还没有安装 `pnpm`，可以使用以下命令安装：

```shell
npm install pnpm -g
```

#### 安装依赖

```sh
pnpm i
```

### 2️⃣ 后端环境准备（Rust）

需要`rust` v1.90.0 及以上

建议使用官方工具安装：[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### 3️⃣ Tauri

在运行项目前，请先根据你的操作系统安装 Tauri 所需依赖：

[https://tauri.app/start/prerequisites/](https://tauri.app/start/prerequisites/)

## ▶️ 运行项目（开发模式）

Tauri 应用至少包含两个进程（详见 [官方文档](https://tauri.app/concept/process-model/)）：

* **Core Process** ：Rust 后端
* **WebView Process** ：前端界面

使用一条命令即可同时启动前后端：

```sh
pnpm tauri dev
```


## 📦 构建发布版本

```sh
pnpm tauri build
```

构建完成后，可执行文件通常位于：

```sh
src-tauri/target/release/
```

不同平台生成的安装包格式可能有所不同。

## 🤝 贡献指南

欢迎任何形式的贡献，包括但不限于：
* 🐛 提交 Bug 报告
* 💡 提出功能建议
* 📖 改进文档
* 🔧 提交代码（PR）

在提交 PR 之前，建议：
1. Fork 本仓库
1. 创建新的分支
1. 保持提交信息清晰、可读
1. 确保本地可以正常运行 pnpm tauri dev
