//! 锁安全访问宏
//!
//! 提供统一的锁中毒恢复模式。

/// 安全获取 Mutex 锁（自动恢复中毒）
///
/// 当锁被毒化时，自动恢复并记录警告日志。
///
/// # 示例
/// ```ignore
/// let mut guard = mutex_lock!(self.db);
/// let guard = mutex_lock!(state.parse_control);
/// ```
#[macro_export]
macro_rules! mutex_lock {
    ($mutex:expr) => {
        $mutex.lock().unwrap_or_else(|e| {
            log::warn!("Lock poisoned, recovering: {}", e);
            e.into_inner()
        })
    };
}

/// 安全获取 RwLock 读锁
#[macro_export]
macro_rules! read_lock {
    ($rwlock:expr) => {
        $rwlock.read().unwrap_or_else(|e| {
            log::warn!("RwLock read poisoned, recovering: {}", e);
            e.into_inner()
        })
    };
}

/// 安全获取 RwLock 写锁
#[macro_export]
macro_rules! write_lock {
    ($rwlock:expr) => {
        $rwlock.write().unwrap_or_else(|e| {
            log::warn!("RwLock write poisoned, recovering: {}", e);
            e.into_inner()
        })
    };
}
