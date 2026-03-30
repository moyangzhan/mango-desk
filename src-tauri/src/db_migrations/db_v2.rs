use anyhow::Result;
use log::info;
use rusqlite::Connection;

/// DB_VERSION = 2
/// Add audio_type and image_hash columns to file_info table for efficient similarity search
/// Add self_hosted_platform table for self-hosted model platforms (Ollama, vLLM, etc.)
pub fn exec_ddl_with_conn(conn: &Connection) -> Result<()> {
    info!("dbv2.exec_ddl_with_conn");

    // Add audio_type column for audio classification
    // Values: 0=Unknown, 1=Speech, 2=Music, 3=Mixed
    // Add image_hash column for perceptual hash (8 bytes for 8x8 hash)
    conn.execute_batch(
        r#"
        ALTER TABLE file_info ADD COLUMN audio_type INTEGER DEFAULT 0;
        ALTER TABLE file_info ADD COLUMN image_hash BLOB;
        CREATE INDEX IF NOT EXISTS idx_file_info_audio_type ON file_info(category, audio_type);
        "#,
    )?;

    // self_hosted_platform table
    conn.execute_batch(
        r#"
        create table if not exists self_hosted_platform (
            id                          integer     primary key autoincrement,
            name                        text        default ''  unique   not null, -- e.g., ollama, vllm
            title                       text        default ''           not null, -- A more readable name, e.g., Ollama, vLLM
            host                        text        default '127.0.0.1'  not null,
            port                        integer     default 11434        not null,
            remark                      text        default ''           not null,
            create_time                 text        default ''           not null,
            update_time                 text        default ''           not null
        );
        create trigger if not exists self_hosted_platform_create_time
        after insert on self_hosted_platform
        for each row
        begin
            update self_hosted_platform
            set create_time = datetime('now', 'localtime'),
                update_time = datetime('now', 'localtime')
            where id = new.id;
        end;
        create trigger if not exists self_hosted_platform_update_time
        after update on self_hosted_platform
        for each row
        begin
            update self_hosted_platform set update_time = datetime('now', 'localtime')
            where id = new.id;
        end;
        "#,
    )?;

    Ok(())
}

/// Initialize self_hosted_platform and ai_model data
pub fn init_data_with_conn(conn: &Connection) -> Result<()> {
    info!("dbv2.init_data_with_conn");

    // Self-hosted platform init
    conn.execute_batch(
        r#"
        insert or ignore into self_hosted_platform (name, title, host, port) values ('ollama', 'Ollama', '127.0.0.1', 11434);
        insert or ignore into self_hosted_platform (name, title, host, port) values ('vllm', 'vLLM', '127.0.0.1', 8000);
        "#,
    )?;

    // Active self-hosted platform config
    conn.execute(
        "insert or ignore into config (name, value) VALUES ('active_self_hosted_platform', 'ollama')",
        [],
    )?;

    // Self-hosted vision model init (llava for image analysis)
    conn.execute_batch(
        r#"
        -- Ollama vision model
        insert or ignore into ai_model (name, title, model_types, platform, input_types, remark, is_enable)
        values ('llava', 'LLaVA', 'vision', 'ollama', 'text,image', 'LLaVA 是一个开源的视觉语言模型，能够理解图片内容并进行对话。需要在 Ollama 中先运行 ollama pull llava 下载模型。| LLaVA is an open-source vision-language model that can understand image content and conduct conversations. Run ollama pull llava to download the model first.', true);

        -- vLLM vision model
        insert or ignore into ai_model (name, title, model_types, platform, input_types, remark, is_enable)
        values ('llava-hf/llava-1.5-7b-hf', 'LLaVA 1.5 7B', 'vision', 'vllm', 'text,image', 'LLaVA 1.5 是一个开源的视觉语言模型，能够理解图片内容。需要在 vLLM 中启动服务时指定此模型。| LLaVA 1.5 is an open-source vision-language model that can understand image content. Specify this model when starting vLLM server.', true);
        "#,
    )?;

    Ok(())
}
