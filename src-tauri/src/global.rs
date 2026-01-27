use crate::document_loaders::docx::DocxLoader;
use crate::document_loaders::excel::ExcelLoader;
use crate::document_loaders::odp::OdpLoader;
use crate::document_loaders::odt::OdtLoader;
use crate::document_loaders::pdf::PdfLoader;
use crate::document_loaders::plain_text::PlainTextLoader;
use crate::document_loaders::pptx::PptxLoader;
use crate::entities::ModelPlatform;
use crate::structs::fs_watcher_setting::FsWatcherSetting;
use crate::structs::indexer_setting::IndexerSetting;
use crate::structs::indexing_summary::IndexingSummary;
use crate::structs::proxy_setting::ProxyInfo;
use crate::traits::document_loader::DocumentLoader;
use chrono::{DateTime, Local};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Arc, LazyLock, OnceLock};
use tokio::sync::RwLock as AsyncRwLock;

pub const DB_VERSION: i32 = 1;

pub const HUGGINFACE_WEBSITE: &str = "https://huggingface.co";
pub const HUGGINFACE_MIRROR: &str = "https://hf-mirror.com";
// multi-language embedding model(384 dimensions): https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/onnx/model.onnx
// web url: https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/blob/main/onnx/model.onnx
// multi-language tokenzier: https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json
// web url: https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/blob/main/tokenizer.json
pub const MULTI_LANG_MODEL_URL: &str = "https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/onnx/model.onnx";
pub const MULTI_LANG_TOKENIZER_URL: &str = "https://huggingface.co/sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2/resolve/main/tokenizer.json";
// assets/model/model.onnx
pub static MULTI_LANG_EMBEDDING_PATH: OnceLock<String> = OnceLock::new();
// assets/model/tokenizer.json
pub static MULTI_LANG_TOKENIZER_PATH: OnceLock<String> = OnceLock::new();
// English embedding model
// path: assets/model/all-minilm-l6-v2.onnx
pub static EN_EMBEDDING_PATH: OnceLock<String> = OnceLock::new(); // 384 dimensions
// assets/model/all-minilm-l6-v2-tokenizer.json
pub static EN_TOKENIZER_PATH: OnceLock<String> = OnceLock::new();

pub const CONFIG_NAME_CLIENT_ID: &'static str = "client_id";
pub const CONFIG_NAME_PROXY: &'static str = "proxy";
pub const CONFIG_NAME_INDEXER_SETTING: &'static str = "indexer_setting";
pub const CONFIG_NAME_WATCHER_SETTING: &'static str = "fs_watcher_setting";
pub const CONFIG_NAME_ACTIVE_LOCALE: &'static str = "active_locale";

pub static APP_DATA_PATH: LazyLock<AsyncRwLock<String>> =
    LazyLock::new(|| AsyncRwLock::new("".to_string()));
pub static ONNX_EXEC_PROVIDERS_INITIALIZED: OnceLock<bool> = OnceLock::new();
pub static HOME_PATH: OnceLock<String> = OnceLock::new();
pub static STORAGE_PATH: OnceLock<String> = OnceLock::new();
pub static DB_PATH: OnceLock<String> = OnceLock::new();
pub static TMP_PATH: OnceLock<String> = OnceLock::new();
pub static EXIT_APP_SIGNAL: AtomicBool = AtomicBool::new(false);
pub static ACTIVE_MODEL_PLATFORM: LazyLock<AsyncRwLock<ModelPlatform>> =
    LazyLock::new(|| AsyncRwLock::new(ModelPlatform::default()));

pub static PROXY: LazyLock<AsyncRwLock<ProxyInfo>> = LazyLock::new(|| {
    AsyncRwLock::new(ProxyInfo {
        protocal: "".to_string(),
        host: "".to_string(),
        port: 0,
    })
});
pub const DEFAULT_DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
pub const DEFAULT_DATETIME_MICRO_FORMAT: &str = "%Y-%m-%d %H:%M:%S.%f";

pub static CLIENT_ID: LazyLock<AsyncRwLock<String>> =
    LazyLock::new(|| AsyncRwLock::new("".to_string())); //Identifier for this client instance
// Current locale, default is en-US
pub static ACTIVE_LOCALE: LazyLock<AsyncRwLock<String>> =
    LazyLock::new(|| AsyncRwLock::new("en-US".to_string()));

pub static DOWNLOADING: AtomicBool = AtomicBool::new(false);

// Scanning related
pub static SCANNING: AtomicBool = AtomicBool::new(false);
pub static SCANNING_TOTAL: AtomicUsize = AtomicUsize::new(0);

macro_rules! define_document_exts {
    ($($name:ident: [$($ext:literal),*];)*) => {
        $(
            pub const $name: &[&str] = &[$($ext),*];
        )*
        pub const SUPPORTED_DOCS_EXTS: &[&str] = &[$($($ext),*),*];
    }
}
define_document_exts! {
    DOCX_EXTS: ["docx"];
    EXCEL_EXTS: ["xlsx", "xls", "xlsm", "xlsb", "xla", "xlam", "ods"];
    ODP_EXTS: ["odp"];
    ODT_EXTS: ["odt"];
    PDF_EXTS: ["pdf"];
    PPTX_EXTS: ["pptx"];
    PLAIN_TEXT_EXTS: ["txt", "log", "md", "mdx", "ini"];
}
pub const SUPPORTED_IMAGE_EXTS: [&str; 5] = ["jpg", "jpeg", "png", "gif", "webp"];
pub const SUPPORTED_AUDIO_EXTS: [&str; 8] =
    ["mp3", "wav", "aac", "flac", "ogg", "m4a", "wma", "amr"];
pub const SUPPORTED_VIDEO_EXTS: [&str; 4] = ["mp4", "avi", "mov", "mkv"];

// Document related
type DocHandler = Arc<dyn DocumentLoader + Send + Sync>;
pub static EXT_TO_DOC_LOADER: LazyLock<AsyncRwLock<HashMap<String, DocHandler>>> =
    LazyLock::new(|| {
        let loaders: Vec<Arc<dyn DocumentLoader + Send + Sync>> = vec![
            Arc::new(DocxLoader::default()) as DocHandler,
            Arc::new(ExcelLoader::default()) as DocHandler,
            Arc::new(OdpLoader::default()) as DocHandler,
            Arc::new(OdtLoader::default()) as DocHandler,
            Arc::new(PdfLoader::default()) as DocHandler,
            Arc::new(PptxLoader::default()) as DocHandler,
            Arc::new(PlainTextLoader::default()) as DocHandler,
        ];
        // key: extension, value: document loader
        let mut ext_to_loader = HashMap::new();
        for loader in loaders {
            for ext in loader.get_exts() {
                ext_to_loader.insert(ext.clone(), Arc::clone(&loader));
            }
        }
        AsyncRwLock::new(ext_to_loader)
    });

// Chunking related
pub const DOCUMENT_CHUNK_SIZE: usize = 1024;
pub const DOCUMENT_CHUNK_OVERLAP: usize = 20;
pub const MAX_DOCUMENT_LOAD_CHARS: usize = 30000;

// Indexing related
pub static INDEXER_SETTING: LazyLock<AsyncRwLock<IndexerSetting>> =
    LazyLock::new(|| AsyncRwLock::new(IndexerSetting::default()));
pub static INDEXING: AtomicBool = AtomicBool::new(false);
pub static STOP_INDEX_SIGNAL: AtomicBool = AtomicBool::new(false);
pub static FS_WATCHER_SETTING: LazyLock<AsyncRwLock<FsWatcherSetting>> =
    LazyLock::new(|| AsyncRwLock::new(FsWatcherSetting::default()));

// Running indexing task summary
pub static INDEXING_SUMMARY: LazyLock<AsyncRwLock<IndexingSummary>> =
    LazyLock::new(|| AsyncRwLock::new(IndexingSummary::default()));

// Ignore dot-prefixed directories, such as .git, .vscode, etc.
pub const IGNORE_HIDDEN_DIRS: bool = true;
// Ignore dot-prefixed files, such as .gitignore, .env, etc.
pub const IGNORE_HIDDEN_FILES: bool = true;

// For user defined document types
// pub async fn supported_doc_exts() -> Vec<String> {
//     let guard = EXT_TO_DOC_LOADER.read().await;
//     guard.keys().cloned().collect()
// }

pub const TRAY_ID: &'static str = "main";
pub const UI_MOUNTED: AtomicBool = AtomicBool::new(false);

pub static PATHS_CACHE: LazyLock<AsyncRwLock<Vec<String>>> =
    LazyLock::new(|| AsyncRwLock::new(vec![]));
pub static PATHS_CACHE_BUILD_TIME: LazyLock<AsyncRwLock<DateTime<Local>>> =
    LazyLock::new(|| AsyncRwLock::new(Local::now()));
