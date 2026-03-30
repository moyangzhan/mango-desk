<sup>[English](./README.md) | [中文](./README_CN.md)</sup>

# Mango Finder

**🥭 Awake your data**

![1691585544443](./screenshots/work.png)

[![Download](./screenshots/download-ready.svg)](https://github.com/moyangzhan/mango-finder/releases)

## 📖 项目简介

Mango Finder（原名 MangoDesk）是一款用自然语言搜索本地文件的桌面应用，支持跨设备搜索功能。

帮助您根据记忆中的内容查找信息，而不需要记住文件名或文件夹结构。

![search](./screenshots/search.gif)

### 📌 使用场景

满足文档、图片、音频等多类型、多设备场景下的智能搜索需求。

- 📝 **个人文档库** - PDF、Word、Markdown 等多年积累的文件
- 🔗 **多设备环境** - 局域网内跨 NAS、Mac、Linux、Windows 搜索
- 🏢 **团队知识库** - 内部文档、项目文档、会议记录等

### ✨ 特性

- 💭 **按内容意思搜索**
  - 通过描述您记得的意思相近内容来查找文档

- 📍 **关键词精确匹配**
  - 通过文件路径或内容中的精确关键词快速定位。当你记得特定的术语或短语时，这是最理想的选择

- 🔍 **相似文件查找**
  - 支持视觉相似图片查找（感知哈希）、语义相似文档查找、音频内容相似查找
  - 一键发现基于视觉、语义或音频指纹相似的相关文件

- 🔗 **跨设备搜索**
  - 在局域网内连接多台设备，实现跨设备搜索
  - 无需手动传输文件，即可搜索其他电脑上的文件

- 🌐 **多语言与跨语言搜索**
  - 无缝支持 100 多种语言。支持跨语种检索（例如使用中文搜索英文内容），无需任何额外配置

- 🔒 **默认保护隐私**
  - 所有数据都保留在您的设备上，确保隐私安全

- 🖥️ **自托管模型支持**
  - 集成 **Ollama** 和 **vLLM**，可部署私有模型服务
  - 适用于团队或企业内网环境，数据不出内网

- ⚡ **快速高效**
  - 通过优化的索引系统提供即时搜索结果

- 👀 **实时文件和目录监控**
  - 自动检测文件和文件夹的变更（添加/修改/删除），并保持索引及搜索结果的实时更新

- 📂 **兼容现有本地文件**
  - 无需重新整理文件夹或重命名文件 — Mango Finder 直接使用您已有的文件

### 🏗️ 架构

**索引**

![indexing](./screenshots/indexing.png)

支持三种处理模式：**本地模式**（完全离线）、**自托管模式**（Ollama/vLLM）、**云端模式**（远程 AI 服务）。

**搜索**

![search](./screenshots/search.png)


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

需要`rust` v1.92.0 及以上

建议使用官方工具安装：[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

### 3. Tauri

在运行项目前，请先根据你的操作系统安装 Tauri 所需依赖：

[https://tauri.app/start/prerequisites/](https://tauri.app/start/prerequisites/)

### 4. 下载模型文件

请从以下任一来源下载所需的模型文件：

1. **GitHub Release**: [model.zip](https://github.com/moyangzhan/mango-finder/releases/download/v0.1.0/model.zip) - 包含所有必需文件
2. **Hugging Face**: [moyangzhan/mango-finder](https://huggingface.co/moyangzhan/mango-finder/tree/main) - 需要手动下载以下文件：
   - *.onnx 模型文件
   - *_tokenizer.json 分词器文件
   - whisper-small-q8_0.bin

下载完成后，请将文件解压到 `src-tauri/assets/model` 目录中。

**所需文件列表**：
- embedding.onnx
- embedding_tokenizer.json
- vision.onnx
- vision_tokenizer.json
- whisper-small-q8_0.bin

### 5. Whisper.cpp 依赖

音频转文字功能使用 [whisper.cpp](https://github.com/ggerganov/whisper.cpp)，不同操作系统需要安装不同的依赖。

#### Windows

在 Windows 上编译需要安装 **CMake** 和 **LLVM/Clang 18**（注意：LLVM 19/20/22 版本存在兼容性问题，请使用 LLVM 18）。

1. **安装 CMake 4.3**

   从 [cmake-4.3.0](https://github.com/Kitware/CMake/releases/tag/v4.3.0-rc2) 下载安装

2. **下载并安装 LLVM 18**
   - 访问 [LLVM 18.1.8 Release](https://github.com/llvm/llvm-project/releases/tag/llvmorg-18.1.8)
   - 下载 `LLVM-18.1.8-win64.exe`
   - 安装时勾选 **"Add LLVM to the system PATH for all users"**

3. **验证安装**
   ```sh
   cmake --version
   clang --version
   ```
   clang 版本应显示 `18.1.8`

4. **设置环境变量（永久）**
   - 按 `Win + R`，输入 `sysdm.cpl`，回车
   - 点击 **"高级"** 选项卡 → **"环境变量"**
   - 在 **"用户变量"** 区域点击 **"新建"**，添加：

   | 变量名 | 值 |
   |--------|-----|
   | `CXXFLAGS` | `/utf-8` |
   | `CFLAGS` | `/utf-8` |

   - 点击确定，**重启终端** 使环境变量生效

5. **编译项目（仅首次需要）**

   打开 **"x64 Native Tools Command Prompt for VS 2022"**（从开始菜单搜索），然后编译：
   ```cmd
   cd your-project-path\src-tauri
   cargo build
   ```

   > ⚠️ **重要提示**：
   > - `/utf-8` 参数是必需的，用于解决中文编码问题
   > - 如果之前编译失败，先运行 `cargo clean -p whisper-rs-sys` 清理缓存
   > - whisper 编译成功后，后续可直接在任意终端使用 `pnpm tauri dev`
   > - VSCode 的 rust-analyzer 插件会在启动时自动检查代码，由于没有 MSVC 环境，whisper-rs-sys 的构建会失败并以红色标识显示在 `target/debug/build` 目录中。如果你已在 "x64 Native Tools Command Prompt for VS 2022" 中构建成功，可以忽略此错误

#### macOS

1. **安装 Xcode Command Line Tools**（如果尚未安装）：
   ```sh
   xcode-select --install
   ```

2. **安装 CMake**：
   ```sh
   brew install cmake
   ```

3. **设置环境变量**（Apple Silicon 必需）：

   对于 Apple Silicon Mac（M1/M2/M3），需要设置以下环境变量：

   | 变量名 | 值 | 用途 |
   |--------|-----|------|
   | `CFLAGS` | `-U__ARM_FEATURE_MATMUL_INT8` | 避免 whisper.cpp 在 ARM 上的编译问题 |
   | `MACOSX_DEPLOYMENT_TARGET` | `10.15` | 设置最低支持的 macOS 版本（Catalina） |

   **临时设置（当前终端会话）：**
   ```sh
   export CFLAGS="-U__ARM_FEATURE_MATMUL_INT8"
   export MACOSX_DEPLOYMENT_TARGET="10.15"
   ```

   **永久设置（添加到 shell 配置）：**
   ```sh
   # zsh（macOS 默认）
   echo 'export CFLAGS="-U__ARM_FEATURE_MATMUL_INT8"' >> ~/.zshrc
   echo 'export MACOSX_DEPLOYMENT_TARGET="10.15"' >> ~/.zshrc
   source ~/.zshrc

   # bash
   echo 'export CFLAGS="-U__ARM_FEATURE_MATMUL_INT8"' >> ~/.bash_profile
   echo 'export MACOSX_DEPLOYMENT_TARGET="10.15"' >> ~/.bash_profile
   source ~/.bash_profile
   ```

4. **添加 Rust target**：

   ```sh
   rustup target add aarch64-apple-darwin
   ```

5. **构建**：

   ```sh
   pnpm tauri build
   # 或显式指定 target
   pnpm tauri build --target aarch64-apple-darwin
   ```

> **注意**：最低支持的 macOS 版本为 10.15（Catalina）。

#### Linux

大多数 Linux 发行版需要安装 C/C++ 编译工具：

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

windows: `src-tauri/target/release/bundle/msi/Mango Finder_0.1.0_x64_en-US.msi`

## ❓ FAQ
### Q: Mango Finder 如何确保数据隐私？

A: Mango Finder 采用本地优先（local-first）架构来确保数据隐私：

#### 本地数据处理
- 所有文档索引和搜索操作都在本地设备上执行
- 默认情况下完全离线运行，不依赖网络
- 仅在用户**手动启用远程模型服务**后，处理图片和音频文件时才会使用远程模型

#### 数据存储
- 默认情况下，所有用户数据都保留在本地设备上

#### 架构详情
如上面的架构图所示，整个处理流程都设计为本地运行，以确保最大程度的隐私和安全性。

### Q: 为什么代码中使用了这么多模型？

A: 代码库包含多个模型，各自服务于不同目的：

#### 1. 本地模型（默认启用）
- `src-tauri/assets/model`中为本地模型文件
- 这些模型在用户计算机上本地运行，用于文档、图片及音频处理
- 优先考虑隐私和性能

#### 2. 自托管模型（可选）
- 通过 **Ollama** 或 **vLLM** 部署私有模型服务
- 适用于团队或企业内部共用模型场景
- 数据不出内网，保障企业级隐私安全

#### 3. 远程模型（可选）
- `gpt-5-mini` 和 `gpt-4o-mini-transcribe`
- 用于图片和音频解析
- 默认禁用，可根据需要启用
- 注意：后续如有本地替代方案，则优先使用本地方案

#### 3. 预留模型（未来功能）
- `qwen-turbo`、`deepseek-chat` 和 `deepseek-reasoner`
- 为即将推出的功能准备，例如：
  - 知识图谱生成
  - 高级文档分析
- 同时也为想要使用这些模型进行自定义开发的开发者提供基础
- 保持对未来功能扩展的灵活性


### Q: 使用多机互联功能时，为什么发现不了局域网中的其他设备？

A: 使用多机互联功能的前提条件：
- 所有设备连接到**同一局域网**
- 所有设备都已**启动 Mango Finder** 并开启了多机互联功能

多机互联功能依赖 mDNS 协议进行设备发现，以下情况可能导致设备无法被发现：

#### 常见原因

1. **网络隔离**
   - 某些路由器或网络环境启用了"AP 隔离"或"客户端隔离"功能
   - 这会阻止局域网内设备之间的直接通信
   - 解决方法：登录路由器管理界面，关闭"AP 隔离"或类似选项

2. **防火墙限制**
   - Windows 防火墙或第三方安全软件可能阻止了入站连接
   - 解决方法：允许 Mango Finder 通过防火墙，或临时关闭防火墙测试

3. **不在同一网段**
   - 设备连接到不同的子网（如 2.4G 和 5G 频段有时会被分配不同网段）
   - 解决方法：确保所有设备连接到同一网段

4. **端口被占用**
   - 默认端口 15678 被其他程序占用
   - 解决方法：在多机互联设置中修改为其他端口

#### 诊断步骤

1. **测试网络连通性**
   ```sh
   # 在设备 A 上 ping 设备 B 的 IP 地址
   ping 192.168.1.xxx
   ```
   如果 ping 不通，说明存在网络隔离问题。

2. **检查端口**
   ```sh
   # 测试目标设备的 HTTP 服务是否可达
   curl http://192.168.1.xxx:7890/ping
   ```

3. **检查防火墙**
   - Windows：控制面板 → Windows Defender 防火墙 → 允许应用通过防火墙
   - macOS：系统偏好设置 → 安全性与隐私 → 防火墙

4. **手动添加设备**
   - 如果自动发现仍不工作，可以在设备列表中点击"添加设备"，手动输入 IP 和端口


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
1. 创建新分支 (git checkout -b feature/xxx)
1. 确保本地可以正常运行 pnpm tauri dev
1. 提交更改 (git commit -m 'feat: xxx')
1. 推送分支 (git push origin feature/xxx)
1. 提交 Pull Request

## ⭐ 支持我们

如果 Mango Finder 对您有帮助，欢迎：
* 在 GitHub 上给项目一个 Star
* 向朋友推荐
* 分享使用体验
