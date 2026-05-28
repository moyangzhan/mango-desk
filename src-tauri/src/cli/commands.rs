use clap::{Parser, Subcommand};
use serde_json;
use std::time::Instant;

use mango_finder_lib::searcher;
use mango_finder_lib::repositories::{file_info_repo, file_content_embedding_repo, file_content_fts_repo, file_metadata_embedding_repo, device_repo, indexing_task_repo};
use mango_finder_lib::global::{INDEXING, SCANNING, INDEXING_SUMMARY};

use crate::output;

#[derive(Parser)]
#[command(name = "mf", about = "Mango Finder Command Line Interface")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output format: json (default) or table
    #[arg(long, default_value = "json", env = "MANGO_FINDER_OUTPUT")]
    pub output: String,

    /// Suppress logs, only output result
    #[arg(long, env = "MANGO_FINDER_QUIET")]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Search documents
    Search {
        /// Search query
        query: String,
        /// Search type: semantic (default), keyword
        #[arg(long, default_value = "semantic")]
        r#type: String,
        /// Remote device ID (optional)
        #[arg(long)]
        device: Option<String>,
        /// Max results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Show detailed help for this command
        #[arg(long = "help-doc")]
        help_doc: bool,
    },

    /// Find similar files by file ID
    Similar {
        /// File ID
        file_id: i64,
        /// Source device ID (optional)
        #[arg(long)]
        device: Option<String>,
        /// Max results
        #[arg(long, default_value = "10")]
        limit: usize,
        /// Show detailed help for this command
        #[arg(long = "help-doc")]
        help_doc: bool,
    },

    /// Index management
    Index {
        #[command(subcommand)]
        action: Option<IndexAction>,
        /// Show detailed help for this command
        #[arg(long = "help-doc")]
        help_doc: bool,
    },

    /// File operations
    File {
        /// File ID
        id: Option<i64>,
        /// Open file with system default program
        #[arg(long)]
        open: bool,
        /// Device ID (optional)
        #[arg(long)]
        device: Option<String>,
        /// Show detailed help for this command
        #[arg(long = "help-doc")]
        help_doc: bool,
    },

    /// Device management
    Device {
        #[command(subcommand)]
        action: Option<DeviceAction>,
        /// Show detailed help for this command
        #[arg(long = "help-doc")]
        help_doc: bool,
    },

    /// Show application status
    Status,

    /// Show version
    Version,

    /// Show CLI documentation
    HelpDoc,

    /// Show detailed documentation for a specific command
    Doc {
        /// Command name (search, similar, index, file, device, status, version)
        command: String,
    },

    /// Show man page style documentation
    Man {
        /// Command name (optional, shows full man page if not specified)
        command: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum IndexAction {
    /// Show index status
    Status,
    /// Start indexing
    Start {
        /// Paths to index
        paths: Vec<String>,
    },
    /// Stop indexing
    Stop,
    /// List indexed files
    List {
        #[arg(long, default_value = "1")]
        page: i64,
        #[arg(long, default_value = "20")]
        page_size: i64,
    },
    /// Clear all index
    Clear,
}

#[derive(Subcommand)]
pub enum DeviceAction {
    /// List online devices
    List,
}

pub async fn handle_search(query: &str, search_type: &str, device: Option<String>, limit: usize, output_format: &str) {
    let start = Instant::now();
    
    let results = if let Some(_device_id) = device {
        // TODO: 跨设备搜索
        output::print_error("Remote device search not yet implemented");
        return;
    } else {
        match search_type {
            "keyword" => searcher::keyword_search(query).await,
            _ => searcher::semantic_search(query).await,
        }
    };

    let elapsed = start.elapsed();
    let limited_results: Vec<_> = results.into_iter().take(limit).collect();
    let total = limited_results.len();

    let data = serde_json::json!({
        "results": limited_results,
        "total": total,
        "elapsed_ms": elapsed.as_millis()
    });

    output::print_success(&data, output_format);
}

pub async fn handle_similar(file_id: i64, device: Option<String>, limit: usize, output_format: &str) {
    if let Some(_device_id) = device {
        // TODO: 跨设备相似搜索
        output::print_error("Remote device similarity search not yet implemented");
        return;
    }

    let start = Instant::now();
    match mango_finder_lib::similarity::similarity_service::find_similars_for_local_file(file_id, limit).await {
        Ok(results) => {
            let elapsed = start.elapsed();
            let data = serde_json::json!({
                "results": results,
                "total": results.len(),
                "elapsed_ms": elapsed.as_millis()
            });
            output::print_success(&data, output_format);
        }
        Err(e) => {
            output::print_error(&format!("Failed to find similar files: {}", e));
        }
    }
}

pub async fn handle_index(action: IndexAction, output_format: &str) {
    match action {
        IndexAction::Status => {
            let total = file_info_repo::count().unwrap_or(0);
            let indexed = file_content_embedding_repo::count().unwrap_or(0);
            let is_indexing = INDEXING.load(std::sync::atomic::Ordering::SeqCst);
            let is_scanning = SCANNING.load(std::sync::atomic::Ordering::SeqCst);

            // 获取当前索引任务的进度
            let indexing_progress = if is_indexing || is_scanning {
                let summary = INDEXING_SUMMARY.read().await;
                let embedding_progress = summary.calculate_all_embedding();
                Some(serde_json::json!({
                    "task_id": summary.task_id,
                    "total": summary.total,
                    "processed": embedding_progress.processed,
                    "success": embedding_progress.success,
                    "failed": embedding_progress.failed,
                    "skipped": embedding_progress.skipped,
                    "duration_seconds": summary.duration
                }))
            } else {
                None
            };

            // 获取最近的索引任务历史
            let recent_tasks = indexing_task_repo::list(1, 5, "id", "desc").unwrap_or_default();

            let data = serde_json::json!({
                "total_files": total,
                "indexed_files": indexed,
                "is_indexing": is_indexing,
                "is_scanning": is_scanning,
                "current_progress": indexing_progress,
                "recent_tasks": recent_tasks
            });
            output::print_success(&data, output_format);
        }
        IndexAction::Start { paths } => {
            if paths.is_empty() {
                output::print_error("No paths specified");
                return;
            }

            let paths_clone = paths.clone();
            tokio::spawn(async move {
                match mango_finder_lib::indexer_service::start_indexing(paths_clone.clone(), "cli").await {
                    Ok(_) => {
                        log::info!("Indexing completed for paths: {:?}", paths_clone);
                    }
                    Err(e) => {
                        log::error!("Indexing failed: {}", e);
                    }
                }
            });

            let data = serde_json::json!({
                "message": "Indexing started in background",
                "paths": paths,
                "tip": "Use 'mango-finder-cli index status' to check progress"
            });
            output::print_success(&data, output_format);
        }
        IndexAction::Stop => {
            mango_finder_lib::global::STOP_INDEX_SIGNAL.store(true, std::sync::atomic::Ordering::SeqCst);
            let data = serde_json::json!({
                "message": "Stop signal sent"
            });
            output::print_success(&data, output_format);
        }
        IndexAction::List { page, page_size } => {
            match file_info_repo::list(page, page_size) {
                Ok(files) => {
                    let data = serde_json::json!({
                        "files": files,
                        "page": page,
                        "page_size": page_size
                    });
                    output::print_success(&data, output_format);
                }
                Err(e) => {
                    output::print_error(&format!("Failed to list files: {}", e));
                }
            }
        }
        IndexAction::Clear => {
            let _ = file_content_fts_repo::clear();
            let _ = file_content_embedding_repo::clear();
            let _ = file_metadata_embedding_repo::clear();
            let _ = file_info_repo::clear();
            let data = serde_json::json!({
                "message": "Index cleared"
            });
            output::print_success(&data, output_format);
        }
    }
}

pub async fn handle_file(id: i64, open: bool, device: Option<String>, output_format: &str) {
    if let Some(_device_id) = device {
        // TODO: 从远程设备获取文件
        output::print_error("Remote device file fetch not yet implemented");
        return;
    }

    match file_info_repo::get_by_id(id) {
        Ok(Some(file)) => {
            if open && is_image(&file.file_ext) {
                open_with_system(&file.path);
            }
            output::print_success(&serde_json::to_value(&file).unwrap(), output_format);
        }
        Ok(None) => {
            output::print_error(&format!("File not found: {}", id));
        }
        Err(e) => {
            output::print_error(&format!("Failed to get file: {}", e));
        }
    }
}

pub async fn handle_device(action: DeviceAction, output_format: &str) {
    match action {
        DeviceAction::List => {
            match device_repo::list() {
                Ok(devices) => {
                    let data = serde_json::json!({
                        "devices": devices,
                        "total": devices.len()
                    });
                    output::print_success(&data, output_format);
                }
                Err(e) => {
                    output::print_error(&format!("Failed to list devices: {}", e));
                }
            }
        }
    }
}

pub async fn handle_status(output_format: &str) {
    let total_files = file_info_repo::count().unwrap_or(0);
    let indexed_files = file_content_embedding_repo::count().unwrap_or(0);
    let is_indexing = mango_finder_lib::global::INDEXING.load(std::sync::atomic::Ordering::SeqCst);

    let data = serde_json::json!({
        "total_files": total_files,
        "indexed_files": indexed_files,
        "is_indexing": is_indexing,
        "version": env!("CARGO_PKG_VERSION")
    });

    output::print_success(&data, output_format);
}

pub fn handle_version(output_format: &str) {
    let data = serde_json::json!({
        "name": "mango-finder-cli",
        "version": env!("CARGO_PKG_VERSION")
    });

    output::print_success(&data, output_format);
}

pub fn handle_help_doc() {
    let doc = include_str!("../../../docs/cli.md");
    println!("{}", doc);
}

pub fn handle_command_doc(command: &str) {
    let doc = match command {
        "search" => {
            r#"search - Search documents

USAGE:
    mf search <query> [OPTIONS]

ARGS:
    <query>    Search query string

OPTIONS:
    --type <type>      Search type: semantic (default), keyword
    --device <id>      Remote device ID (optional)
    --limit <n>        Max results (default: 10)
    --output <format>  Output format: json (default), table
    --quiet            Suppress logs
    --help-doc         Show detailed help for this command
    -h, --help         Print help

EXAMPLES:
    # Semantic search
    mf search "machine learning"

    # Keyword search
    mf search "report.docx" --type keyword

    # Limit results
    mf search "AI" --limit 5

    # Table output
    mf search "AI" --output table

NOTES:
    - Semantic search uses AI embeddings for meaning-based matching
    - Keyword search uses full-text search for exact word matching
    - Results are ranked by relevance score (0-100)"#
        }
        "similar" => {
            r#"similar - Find similar files by file ID

USAGE:
    mf similar <file_id> [OPTIONS]

ARGS:
    <file_id>    File ID to find similar files for

OPTIONS:
    --device <id>      Remote device ID (optional)
    --limit <n>        Max results (default: 10)
    --output <format>  Output format: json (default), table
    --quiet            Suppress logs
    --help-doc         Show detailed help for this command
    -h, --help         Print help

EXAMPLES:
    # Find similar files
    mf similar 123

    # Limit results
    mf similar 123 --limit 5

NOTES:
    - Similarity is based on file content, not filename
    - Works for documents (semantic), images (perceptual hash), and audio (fingerprint)"#
        }
        "index" => {
            r#"index - Index management

USAGE:
    mf index <action>

ACTIONS:
    status              Show index status and progress
    start <paths...>    Start indexing in background
    stop                Stop indexing
    list                List indexed files
    clear               Clear all index

OPTIONS:
    --output <format>  Output format: json (default), table
    --quiet            Suppress logs
    --help-doc         Show detailed help for this command
    -h, --help         Print help

EXAMPLES:
    # Show status
    mf index status

    # Start indexing
    mf index start "C:\Documents" "D:\Projects"

    # List files (with pagination)
    mf index list --page 1 --page-size 50

    # Stop indexing
    mf index stop

    # Clear all index
    mf index clear

NOTES:
    - index start runs asynchronously in background
    - Use index status to check progress
    - index clear deletes all indexed data permanently"#
        }
        "file" => {
            r#"file - File operations

USAGE:
    mf file <id> [OPTIONS]

ARGS:
    <id>    File ID

OPTIONS:
    --open             Open file with system default program
    --device <id>      Remote device ID (optional)
    --output <format>  Output format: json (default), table
    --quiet            Suppress logs
    --help-doc         Show detailed help for this command
    -h, --help         Print help

EXAMPLES:
    # Get file info
    mf file 123

    # Open file (images, documents)
    mf file 123 --open

NOTES:
    - --open uses system default program for the file type
    - Works best with images (jpg, png, gif, webp, bmp)"#
        }
        "device" => {
            r#"device - Device management

USAGE:
    mf device <action>

ACTIONS:
    list    List online devices

OPTIONS:
    --output <format>  Output format: json (default), table
    --quiet            Suppress logs
    --help-doc         Show detailed help for this command
    -h, --help         Print help

EXAMPLES:
    # List online devices
    mf device list

NOTES:
    - Shows paired and online devices only
    - Use the GUI to manage device pairing"#
        }
        "status" => {
            r#"status - Show application status

USAGE:
    mf status [OPTIONS]

OPTIONS:
    --output <format>  Output format: json (default), table
    --quiet            Suppress logs
    -h, --help         Print help

EXAMPLES:
    # Show status
    mf status

NOTES:
    - Shows total files, indexed files, and indexing status
    - Also shows CLI version"#
        }
        "version" => {
            r#"version - Show version information

USAGE:
    mf version [OPTIONS]

OPTIONS:
    --output <format>  Output format: json (default), table
    --quiet            Suppress logs
    -h, --help         Print help

EXAMPLES:
    # Show version
    mf version"#
        }
        "help-doc" => {
            r#"help-doc - Show CLI documentation

USAGE:
    mf help-doc [OPTIONS]

OPTIONS:
    -h, --help    Print help

EXAMPLES:
    # Show full documentation
    mf help-doc

NOTES:
    - Shows the complete CLI documentation
    - Use -h or --help for quick command help"#
        }
        "doc" => {
            r#"doc - Show detailed documentation for a command

USAGE:
    mf doc <command>

ARGS:
    <command>    Command name (search, similar, index, file, device, status, version, help-doc)

EXAMPLES:
    # Show search command documentation
    mf doc search

    # Show index command documentation
    mf doc index"#
        }
        "man" => {
            r#"man - Show man page style documentation

USAGE:
    mf man [command]

ARGS:
    [command]    Command name (optional)

EXAMPLES:
    # Show full man page
    mf man

    # Show man page for search command
    mf man search"#
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Available commands: search, similar, index, file, device, status, version, help-doc, doc, man");
            return;
        }
    };
    println!("{}", doc);
}

pub fn handle_man(command: Option<&str>) {
    match command {
        Some(cmd) => {
            // 显示单个命令的 man page
            let man_content = get_man_page(cmd);
            println!("{}", man_content);
        }
        None => {
            // 显示完整的 man page
            let man_content = get_full_man_page();
            println!("{}", man_content);
        }
    }
}

fn get_man_page(command: &str) -> String {
    match command {
        "search" => {
            r#"MF(1)                     Mango Finder CLI                     MF(1)

NAME
       mf search - Search documents using semantic or keyword search

SYNOPSIS
       mf search [OPTIONS] <query>

DESCRIPTION
       Search for documents in the index using either semantic (AI-based) or
       keyword (full-text) search. Semantic search uses AI embeddings to find
       documents based on meaning, while keyword search uses full-text search
       for exact word matching.

OPTIONS
       --type <type>
              Search type: semantic (default), keyword

       --device <id>
              Remote device ID (optional)

       --limit <n>
              Max results (default: 10)

       --output <format>
              Output format: json (default), table

       --quiet
              Suppress logs, only output result

       --help-doc
              Show detailed help for this command

       -h, --help
              Print help

EXAMPLES
       Semantic search:
              mf search "machine learning"

       Keyword search:
              mf search "report.docx" --type keyword

       Limit results:
              mf search "AI" --limit 5

       Table output:
              mf search "AI" --output table

EXIT STATUS
       0      Success
       1      Error (invalid arguments, search failed, etc.)

ENVIRONMENT
       MANGO_FINDER_OUTPUT
              Default output format (json or table)

       MANGO_FINDER_QUIET
              Set to 1 to enable quiet mode

SEE ALSO
       mf(1), mf-similar(1), mf-index(1), mf-file(1)

AUTHORS
       Mango Finder Team

VERSION
       0.12.0"#.to_string()
        }
        "similar" => {
            r#"MF-SIMILAR(1)            Mango Finder CLI            MF-SIMILAR(1)

NAME
       mf similar - Find similar files by file ID

SYNOPSIS
       mf similar [OPTIONS] <file_id>

DESCRIPTION
       Find files similar to the specified file. Similarity is based on file
       content, not filename. Works for documents (semantic), images
       (perceptual hash), and audio (fingerprint).

OPTIONS
       --device <id>
              Remote device ID (optional)

       --limit <n>
              Max results (default: 10)

       --output <format>
              Output format: json (default), table

       --quiet
              Suppress logs, only output result

       --help-doc
              Show detailed help for this command

       -h, --help
              Print help

EXAMPLES
       Find similar files:
              mf similar 123

       Limit results:
              mf similar 123 --limit 5

EXIT STATUS
       0      Success
       1      Error (file not found, search failed, etc.)

SEE ALSO
       mf(1), mf-search(1), mf-file(1)

VERSION
       0.12.0"#.to_string()
        }
        "index" => {
            r#"MF-INDEX(1)              Mango Finder CLI              MF-INDEX(1)

NAME
       mf index - Manage document index

SYNOPSIS
       mf index <action> [OPTIONS]

DESCRIPTION
       Manage the document index. Can start, stop, and monitor indexing
       tasks, as well as list and clear indexed files.

ACTIONS
       status Show index status and progress

       start <paths...>
              Start indexing in background

       stop   Stop indexing

       list   List indexed files

       clear  Clear all index

OPTIONS
       --page <n>
              Page number for list (default: 1)

       --page-size <n>
              Page size for list (default: 20)

       --output <format>
              Output format: json (default), table

       --quiet
              Suppress logs, only output result

       --help-doc
              Show detailed help for this command

       -h, --help
              Print help

EXAMPLES
       Show status:
              mf index status

       Start indexing:
              mf index start "C:\Documents" "D:\Projects"

       List files:
              mf index list --page 1 --page-size 50

       Stop indexing:
              mf index stop

       Clear all index:
              mf index clear

NOTES
       index start runs asynchronously in background
       Use index status to check progress
       index clear deletes all indexed data permanently

EXIT STATUS
       0      Success
       1      Error (indexing failed, etc.)

SEE ALSO
       mf(1), mf-search(1), mf-file(1)

VERSION
       0.12.0"#.to_string()
        }
        "file" => {
            r#"MF-FILE(1)               Mango Finder CLI               MF-FILE(1)

NAME
       mf file - File operations

SYNOPSIS
       mf file [OPTIONS] <id>

DESCRIPTION
       Get information about a specific file by its ID. Can also open the
       file with the system default program.

OPTIONS
       --open Open file with system default program

       --device <id>
              Remote device ID (optional)

       --output <format>
              Output format: json (default), table

       --quiet
              Suppress logs, only output result

       --help-doc
              Show detailed help for this command

       -h, --help
              Print help

EXAMPLES
       Get file info:
              mf file 123

       Open file:
              mf file 123 --open

EXIT STATUS
       0      Success
       1      Error (file not found, etc.)

SEE ALSO
       mf(1), mf-search(1), mf-index(1)

VERSION
       0.12.0"#.to_string()
        }
        "device" => {
            r#"MF-DEVICE(1)            Mango Finder CLI            MF-DEVICE(1)

NAME
       mf device - Device management

SYNOPSIS
       mf device <action>

DESCRIPTION
       Manage devices for cross-device search. Currently only supports
       listing online devices.

ACTIONS
       list   List online devices

OPTIONS
       --output <format>
              Output format: json (default), table

       --quiet
              Suppress logs, only output result

       --help-doc
              Show detailed help for this command

       -h, --help
              Print help

EXAMPLES
       List online devices:
              mf device list

EXIT STATUS
       0      Success
       1      Error (device not found, etc.)

SEE ALSO
       mf(1), mf-search(1)

VERSION
       0.12.0"#.to_string()
        }
        "status" => {
            r#"MF-STATUS(1)            Mango Finder CLI            MF-STATUS(1)

NAME
       mf status - Show application status

SYNOPSIS
       mf status [OPTIONS]

DESCRIPTION
       Show the current status of the Mango Finder application, including
       total files, indexed files, and indexing status.

OPTIONS
       --output <format>
              Output format: json (default), table

       --quiet
              Suppress logs, only output result

       -h, --help
              Print help

EXAMPLES
       Show status:
              mf status

EXIT STATUS
       0      Success

SEE ALSO
       mf(1), mf-version(1)

VERSION
       0.12.0"#.to_string()
        }
        "version" => {
            r#"MF-VERSION(1)           Mango Finder CLI           MF-VERSION(1)

NAME
       mf version - Show version information

SYNOPSIS
       mf version [OPTIONS]

DESCRIPTION
       Show the version of the Mango Finder CLI.

OPTIONS
       --output <format>
              Output format: json (default), table

       --quiet
              Suppress logs, only output result

       -h, --help
              Print help

EXAMPLES
       Show version:
              mf version

EXIT STATUS
       0      Success

SEE ALSO
       mf(1), mf-status(1)

VERSION
       0.12.0"#.to_string()
        }
        _ => {
            format!("No man page available for '{}'. Available commands: search, similar, index, file, device, status, version", command)
        }
    }
}

fn get_full_man_page() -> String {
    r#"MF(1)                     Mango Finder CLI                     MF(1)

NAME
       mf - Mango Finder command line interface

SYNOPSIS
       mf [OPTIONS] [COMMAND]

DESCRIPTION
       Mango Finder CLI is a command line interface for searching and
       managing documents using AI-powered semantic search. It supports
       semantic search, keyword search, file similarity detection, and
       cross-device search capabilities.

OPTIONS
       --output <format>
              Output format: json (default), table

       --quiet
              Suppress logs, only output result

       -h, --help
              Print help

       -V, --version
              Print version

COMMANDS
       search <query>
              Search documents using semantic or keyword search

       similar <file_id>
              Find similar files by file ID

       index <action>
              Manage document index (status, start, stop, list, clear)

       file <id>
              File operations (get info, open)

       device <action>
              Device management (list online devices)

       status Show application status

       version
              Show version information

       help-doc
              Show CLI documentation

       doc <command>
              Show detailed documentation for a specific command

       man [command]
              Show man page style documentation

EXAMPLES
       Search for documents:
              mf search "machine learning"

       Find similar files:
              mf similar 123

       Start indexing:
              mf index start "C:\Documents"

       Get file info:
              mf file 123

       Show status:
              mf status

ENVIRONMENT
       MANGO_FINDER_OUTPUT
              Default output format (json or table)

       MANGO_FINDER_QUIET
              Set to 1 to enable quiet mode

FILES
       ~/.config/mango-finder/
              Configuration directory

       ~/AppData/Roaming/mango-finder/
              Data directory (Windows)

EXIT STATUS
       0      Success
       1      Error (invalid arguments, operation failed, etc.)

SEE ALSO
       mf-search(1), mf-similar(1), mf-index(1), mf-file(1), mf-device(1)

AUTHORS
       Mango Finder Team

VERSION
       0.12.0

REPORTING BUGS
       Report bugs at: https://github.com/moyangzhan/mango-finder/issues

COPYRIGHT
       Copyright © 2026 Mango Finder. Licensed under MIT."#.to_string()
}

fn is_image(ext: &str) -> bool {
    matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "webp" | "bmp")
}

fn open_with_system(path: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(path)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg(path)
            .spawn();
    }
}
