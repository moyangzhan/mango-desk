use crate::embedding_service::EmbeddingService;
use crate::errors::AppError;
use std::sync::OnceLock;
use std::time::Instant;
use tokio::sync::RwLock as AsyncRwLock;
use tokio::time::Duration;

pub static EMBEDDING_SERVICE_MANAGER: OnceLock<AsyncRwLock<EmbeddingServiceManager>> =
    OnceLock::new();
pub static SERVICE_DURATION: Duration = Duration::from_secs(60 * 30);

pub fn embedding_service_manager() -> &'static AsyncRwLock<EmbeddingServiceManager> {
    EMBEDDING_SERVICE_MANAGER.get_or_init(|| AsyncRwLock::new(EmbeddingServiceManager::default()))
}

pub struct EmbeddingServiceManager {
    pub service: Option<EmbeddingService>,
    pub last_used: Instant,
}

impl Default for EmbeddingServiceManager {
    fn default() -> Self {
        Self {
            service: None,
            last_used: Instant::now(),
        }
    }
}

impl EmbeddingServiceManager {
    pub async fn warmup(&mut self) -> Result<(), AppError> {
        if self.service.is_none() {
            match EmbeddingService::new().await {
                Ok(service) => {
                    self.service = Some(service);
                }
                Err(e) => {
                    log::error!("EmbeddingService::new failed: {:?}", e);
                    return Err(e);
                }
            }
        }
        self.update_last_used();
        Ok(())
    }

    pub async fn service(&mut self) -> Result<&EmbeddingService, AppError> {
        self.warmup().await?;
        return self.service.as_ref().ok_or(AppError::EmbeddingError(
            "Embedding service is none".to_string(),
        ));
    }

    pub fn remove_if_expired(&mut self) {
        if self.last_used.elapsed() > SERVICE_DURATION {
            self.service = None;
        }
    }

    pub fn clear(&mut self) {
        self.service = None;
    }

    pub async fn embed(&mut self, text: &str) -> Result<Vec<f32>, AppError> {
        let service = self.service().await?;
        let result = service.embed(text);
        self.update_last_used();
        result
    }

    fn update_last_used(&mut self) {
        self.last_used = Instant::now();
    }
}
