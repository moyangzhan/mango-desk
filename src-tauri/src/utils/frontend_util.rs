use crate::global::APP_HANDLE;
use serde::Serialize;
use tauri::Emitter;
use tauri::ipc::Channel;
use tauri::ipc::IpcResponse;

/// Notify frontend with data
pub fn send_to_frontend<T: IpcResponse>(on_event: &Channel<T>, data: T) {
    if let Err(e) = on_event.send(data) {
        log::error!("Send channel message error: {}", e);
    }
}

pub fn send_event<T: Serialize>(name: &str, data: &T) {
    match APP_HANDLE.get() {
        Some(app_handle) => match serde_json::to_string(data) {
            Ok(json_string) => {
                if let Err(e) = app_handle.emit(name, json_string) {
                    log::error!("Send channel message error: {}", e);
                }
            }
            Err(e) => {
                log::error!("Failed to serialize data to JSON: {}", e);
            }
        },
        None => {}
    }
}
