# Mango Finder CLI (mf)

Mango Finder 命令行工具，方便开发者和 AI Agent 使用。

## 安装

CLI 工具随 Mango Finder 一起构建，位于 `src-tauri/target/release/mf.exe`

## 获取帮助

```bash
# 显示帮助信息
mf --help
mf -h

# 显示子命令帮助
mf search --help
mf index --help

# 显示详细文档
mf doc search
mf doc index
```

## 命令列表

### search - 搜索文档

```bash
mf search <query> [--type semantic|keyword] [--device <id>] [--limit <n>]
```

**参数说明：**
- `query`：搜索关键词
- `--type`：搜索类型，`semantic`（语义搜索，默认）或 `keyword`（关键词搜索）
- `--device`：远程设备 ID（可选）
- `--limit`：返回结果数量限制，默认 10

**示例：**
```bash
# 语义搜索
mf search "机器学习"

# 关键词搜索
mf search "report.docx" --type keyword --limit 5
```

### similar - 查找相似文件

```bash
mf similar <file_id> [--device <id>] [--limit <n>]
```

**示例：**
```bash
mf similar 123 --limit 5
```

### index - 索引管理

```bash
mf index <action>
```

**可用操作：**
- `status`：查看索引状态和进度
- `start <paths...>`：在后台启动索引任务
- `stop`：停止索引
- `list [--page <n>] [--page-size <n>]`：列出已索引文件
- `clear`：清空所有索引

**示例：**
```bash
# 查看索引状态（包含当前进度和最近任务历史）
mf index status

# 在后台启动索引（立即返回）
mf index start "C:\Documents" "D:\Projects"

# 列出已索引文件
mf index list --page 1 --page-size 50
```

**说明：**
- `index start` 命令会在后台异步执行，不会阻塞终端
- 可以使用 `index status` 命令随时查询索引进度

### file - 文件操作

```bash
mf file <id> [--open] [--device <id>]
```

**参数说明：**
- `id`：文件 ID
- `--open`：使用系统默认程序打开文件（主要用于图片）
- `--device`：远程设备 ID（可选）

**示例：**
```bash
# 查看文件信息
mf file 123

# 打开图片
mf file 123 --open
```

### device - 设备管理

```bash
mf device <action>
```

**可用操作：**
- `list`：列出在线设备

### status - 应用状态

```bash
mf status
```

### version - 版本信息

```bash
mf version
```

### help-doc - 查看文档

```bash
mf help-doc
```

### doc - 查看命令详细文档

```bash
mf doc <command>
```

**示例：**
```bash
# 查看 search 命令的详细文档
mf doc search

# 查看 index 命令的详细文档
mf doc index
```

### man - 查看 man page 风格文档

```bash
mf man [command]
```

**示例：**
```bash
# 查看完整 man page
mf man

# 查看 search 命令的 man page
mf man search

# 查看 index 命令的 man page
mf man index
```

### check - 检查系统状态

```bash
mf check
```

**检查内容：**
- 系统信息（版本、设备名称、客户端 ID）
- 网络状态（本地 IP、网络接口、端口可用性）
- 存储状态（路径、已索引文件数量、embedding 数量）
- 集群状态（是否启用、已配对设备数量）
- AI 模型状态（模型文件是否存在）

**示例：**
```bash
# 检查系统状态
mf check

# JSON 格式输出
mf check --output json
```

### locale - 获取或设置语言

```bash
mf locale [value]
```

**参数：**
- `value`：要设置的语言（如 `zh-CN`、`en-US`）。如果不指定，则显示当前语言。

**示例：**
```bash
# 显示当前语言
mf locale

# 设置为中文
mf locale zh-CN

# 设置为英文
mf locale en-US
```

## 全局选项

| 选项 | 环境变量 | 说明 |
|------|----------|------|
| `--output json\|table` | `MANGO_FINDER_OUTPUT` | 输出格式，默认 `json` |
| `--quiet` | `MANGO_FINDER_QUIET` | 静默模式，只输出结果，不输出日志 |

## 环境变量

可以通过环境变量配置默认行为：

```bash
# 设置默认输出格式
export MANGO_FINDER_OUTPUT=table

# 启用静默模式
export MANGO_FINDER_QUIET=1
```

## 输出格式

默认输出 JSON 格式：

```json
{
  "success": true,
  "data": {
    "results": [...],
    "total": 10,
    "elapsed_ms": 156
  },
  "error": null
}
```

错误输出：

```json
{
  "success": false,
  "data": null,
  "error": "Error message"
}
```

## AI Agent 使用示例

```bash
# 搜索文档
mf search "机器学习算法" --output json

# 获取文件详情
mf file 123 --output json

# 检查索引状态
mf index status --output json

# 查找相似文件
mf similar 456 --limit 5 --output json
```
