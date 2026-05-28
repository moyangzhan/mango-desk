mod commands;
mod output;

use clap::Parser;
use clap::CommandFactory;
use mango_finder_lib::utils::app_util;
use mango_finder_lib::initializer;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "en-US");

fn main() {
    let cli = commands::Cli::parse();

    // 如果没有子命令，显示帮助信息
    if cli.command.is_none() {
        let mut cmd = commands::Cli::command();
        cmd.print_help().ok();
        println!();
        return;
    }

    // 初始化日志（非静默模式）
    if !cli.quiet {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format_timestamp(None)
            .init();
    }

    // 初始化路径（不依赖 Tauri）
    if let Err(e) = app_util::init_paths_standalone() {
        eprintln!("Error: {}", e);
        eprintln!();
        eprintln!("Troubleshooting:");
        eprintln!("  1. Ensure the executable has write permissions to the data directory");
        eprintln!("  2. Check if the data directory is accessible");
        eprintln!("  3. Try running with administrator privileges if needed");
        output::print_error(&e);
    }

    // 初始化核心服务
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        initializer::process().await;
    });

    // 执行命令
    let command = cli.command.unwrap();
    rt.block_on(async {
        match command {
            commands::Commands::Search { query, r#type, device, limit, help_doc } => {
                if help_doc {
                    commands::handle_help(Some("search"));
                    return;
                }
                commands::handle_search(&query, &r#type, device, limit, &cli.output).await;
            }
            commands::Commands::Similar { file_id, device, limit, help_doc } => {
                if help_doc {
                    commands::handle_help(Some("similar"));
                    return;
                }
                commands::handle_similar(file_id, device, limit, &cli.output).await;
            }
            commands::Commands::Index { action, help_doc } => {
                if help_doc {
                    commands::handle_help(Some("index"));
                    return;
                }
                match action {
                    Some(action) => commands::handle_index(action, &cli.output).await,
                    None => commands::handle_help(Some("index")),
                }
            }
            commands::Commands::File { id, open, device, help_doc } => {
                if help_doc {
                    commands::handle_help(Some("file"));
                    return;
                }
                match id {
                    Some(id) => commands::handle_file(id, open, device, &cli.output).await,
                    None => commands::handle_help(Some("file")),
                }
            }
            commands::Commands::Device { action, help_doc } => {
                if help_doc {
                    commands::handle_help(Some("device"));
                    return;
                }
                match action {
                    Some(action) => commands::handle_device(action, &cli.output).await,
                    None => commands::handle_help(Some("device")),
                }
            }
            commands::Commands::Status => {
                commands::handle_status(&cli.output).await;
            }
            commands::Commands::Version => {
                commands::handle_version(&cli.output);
            }
            commands::Commands::Help { command } => {
                commands::handle_help(command.as_deref());
            }
            commands::Commands::Check => {
                commands::handle_check(&cli.output);
            }
            commands::Commands::Locale { value } => {
                commands::handle_locale(value, &cli.output);
            }
        }
    });
}
