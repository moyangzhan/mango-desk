<sup>[English](./README.md) | [中文](./README_CN.md)</sup>

# Mango Desk

**🥭 Awake your data**

![1691585544443](./screenshots/work.png)

[![Download](./screenshots/download-ready.svg)](https://github.com/moyangzhan/mango-desk/releases)

## 📖 项目简介

Mango Desk 是一款用自然语言搜索本地文档的桌面应用。

帮助您根据记忆中的内容查找信息，而不需要记住文件名或文件夹结构。

### 📌 使用场景

拥有**大量本地文档**并希望通过自然语言检索信息时。

- 📝 **个人文档库**
  - 多年来积累的笔记、PDF、Word 文件、Markdown 文件等
  - 示例：*"我总结 Rust 所有权规则的那份笔记"*

- 📂 **SVN / Git 仓库**
  - 搜索设计文档、README、技术方案和历史解决方案
  - 示例：*"关于权限重构的文档在哪里？"*

- 🏢 **团队或公司知识库**
  - 内部文档、项目文档、会议记录、入职材料
  - 示例：*"查找所有关于预算规划的第四季度会议记录"*
  - 示例：*"公司关于远程工作的政策是什么？"*

- 📚 **研究与学术资料**
  - 论文、实验记录、文献笔记
  - 示例：*"关于 AI 的最新研究有哪些？"*

- ⚖️ **法律与财务文档**
  - 合同、政策文件、报告
  - 示例：*"最新的公司数据隐私政策是什么？"*

### ✨ 特性

- 🔍 **按内容搜索，而非文件名**
  - 通过描述您记得的内容来查找文档，即使不记得确切的标题或位置

- 📂 **兼容现有本地文件**
  - 无需重新整理文件夹或重命名文件 — Mango Desk 直接使用您已有的文件

- ⚡ **快速轻量**
  - 即时获得搜索结果，不会拖慢系统运行

- 🔒 **默认保护隐私**
  - 所有数据都保留在您的设备上，确保隐私安全

### 🛠️ 技术栈

* Frontend
  * WebView（Tauri）
  * PNPM
  * Node.js
* Backend
  * Rust
  * Tauri Core

## 🚀 快速开始（开发环境）

### 1. 前端环境准备

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

### 2. 后端环境准备（Rust）

需要`rust` v1.90.0 及以上

建议使用官方工具安装：[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### 3. Tauri

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

不同平台生成的安装包格式可能有所不同，如

windows: `src-tauri/target/release/bundle/msi/Mango Desk_0.1.0_x64_en-US.msi`

## 📝 LICENSE

[LICENSE](LICENSE)

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

## ⭐ 支持我们

如果 Mango Desk 对您有帮助，欢迎：
* 在 GitHub 上给项目一个 Star
* 向朋友推荐
* 分享使用体验

## 💻 截图

![home](./screenshots/home.png)