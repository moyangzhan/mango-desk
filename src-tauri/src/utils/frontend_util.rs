use tauri::ipc::Channel;
use tauri::ipc::IpcResponse;

/// Notify frontend with data
pub fn send_to_frontend<T: IpcResponse>(on_event: &Channel<T>, data: T) {
    if let Err(e) = on_event.send(data) {
        println!("Send channel message error:{}", e);
    }
}
