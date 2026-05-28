# Mango Finder CLI (mf)

Mango Finder Command Line Interface for searching and managing documents.

## Installation

CLI tool is built with Mango Finder, located at `src-tauri/target/release/mf.exe`

## Getting Help

```bash
# Show help
mf --help
mf -h

# Show subcommand help
mf search --help
mf index --help

# Show detailed documentation
mf doc search
mf doc index
```

## Command List

### search - Search Documents

```bash
mf search <query> [--type semantic|keyword] [--device <id>] [--limit <n>]
```

**Parameters:**
- `query`: Search query string
- `--type`: Search type, `semantic` (default) or `keyword`
- `--device`: Remote device ID (optional)
- `--limit`: Max results, default 10

**Examples:**
```bash
# Semantic search
mf search "machine learning"

# Keyword search
mf search "report.docx" --type keyword --limit 5
```

### similar - Find Similar Files

```bash
mf similar <file_id> [--device <id>] [--limit <n>]
```

**Examples:**
```bash
mf similar 123 --limit 5
```

### index - Index Management

```bash
mf index <action>
```

**Actions:**
- `status`: Show index status and progress
- `start <paths...>`: Start indexing in background
- `stop`: Stop indexing
- `list [--page <n>] [--page-size <n>]`: List indexed files
- `clear`: Clear all index

**Examples:**
```bash
# Show index status
mf index status

# Start indexing in background
mf index start "C:\Documents" "D:\Projects"

# List indexed files
mf index list --page 1 --page-size 50
```

**Notes:**
- `index start` runs asynchronously in background
- Use `index status` to check progress

### file - File Operations

```bash
mf file <id> [--open] [--device <id>]
```

**Parameters:**
- `id`: File ID
- `--open`: Open file with system default program (mainly for images)
- `--device`: Remote device ID (optional)

**Examples:**
```bash
# Get file info
mf file 123

# Open image
mf file 123 --open
```

### device - Device Management

```bash
mf device <action>
```

**Actions:**
- `list`: List online devices

### status - Application Status

```bash
mf status
```

### version - Version Info

```bash
mf version
```

### help-doc - View Documentation

```bash
mf help-doc
```

### doc - View Command Documentation

```bash
mf doc <command>
```

**Examples:**
```bash
# View search command documentation
mf doc search

# View index command documentation
mf doc index
```

### man - View Man Page

```bash
mf man [command]
```

**Examples:**
```bash
# View full man page
mf man

# View search command man page
mf man search
```

### check - Check System Status

```bash
mf check
```

**Check items:**
- System info (version, device name, client ID)
- Network status (local IP, network interfaces, port availability)
- Storage status (path, indexed files count, embedding count)
- Cluster status (enabled, paired devices count)
- AI model status (model files existence)

**Examples:**
```bash
# Check system status
mf check

# JSON output
mf check --output json
```

### locale - Get or Set Locale

```bash
mf locale [value]
```

**Parameters:**
- `value`: Locale to set (e.g., `zh-CN`, `en-US`). If not specified, show current locale.

**Examples:**
```bash
# Show current locale
mf locale

# Set locale to Chinese
mf locale zh-CN

# Set locale to English
mf locale en-US
```

## Global Options

| Option | Environment Variable | Description |
|--------|---------------------|-------------|
| `--output json\|table` | `MANGO_FINDER_OUTPUT` | Output format, default `json` |
| `--quiet` | `MANGO_FINDER_QUIET` | Suppress logs, only output result |

## Environment Variables

Configure default behavior via environment variables:

```bash
# Set default output format
export MANGO_FINDER_OUTPUT=table

# Enable quiet mode
export MANGO_FINDER_QUIET=1
```

## Output Format

Default JSON output:

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

Error output:

```json
{
  "success": false,
  "data": null,
  "error": "Error message"
}
```

## AI Agent Examples

```bash
# Search documents
mf search "machine learning algorithms" --output json

# Get file details
mf file 123 --output json

# Check index status
mf index status --output json

# Find similar files
mf similar 456 --limit 5 --output json
```
