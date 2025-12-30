use crate::embedding_service_manager::embedding_service_manager;
use crate::global::EXIT_APP_SIGNAL;
use log::{info, warn};
use std::sync::atomic::Ordering;
use std::time::Instant;
use tokio::time::Duration;

pub fn start_after_ui_mounted() {
    let _ = tokio::spawn(async {
        embedding_service_cleanup().await;
        // match catch_unwind(AssertUnwindSafe(|| {
        //     tokio::task::block_in_place(|| {
        //         tokio::runtime::Handle::current().block_on(embedding_service_cleanup())
        //     })
        // })) {
        //     Ok(_) => info!("embedding_service_cleanup finished without panic"),
        //     Err(e) => error!("embedding_service_cleanup panicked: {:?}", e),
        // }
    });
}

async fn embedding_service_cleanup() {
    let mut interval = tokio::time::interval(Duration::from_millis(200));
    let mut last_service_check = Instant::now();
    loop {
        if EXIT_APP_SIGNAL.load(Ordering::SeqCst) {
            info!("Exiting embedding_service_cleanup loop");
            break;
        }
        interval.tick().await;
        if last_service_check.elapsed() >= Duration::from_secs(30) {
            let manager = embedding_service_manager();
            match manager.try_write() {
                Ok(mut guard) => {
                    last_service_check = Instant::now();
                    guard.remove_if_expired();
                }
                Err(_) => {
                    warn!("Failed to acquire write lock on service manager");
                }
            }
        }
    }
}
