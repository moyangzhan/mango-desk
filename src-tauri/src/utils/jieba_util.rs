use jieba_rs::Jieba;
use std::sync::LazyLock;
use tokio::sync::RwLock as AsyncRwLock;

pub static JIEBA: LazyLock<AsyncRwLock<Jieba>> = LazyLock::new(|| AsyncRwLock::new(Jieba::new()));

pub async fn tokenize(text: &str) -> String {
    let jieba = JIEBA.read().await;
    jieba
        .cut(text, true)
        .into_iter()
        .map(|w| w.to_lowercase())
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}
