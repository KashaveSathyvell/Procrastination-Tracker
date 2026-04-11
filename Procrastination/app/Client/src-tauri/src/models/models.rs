use std::sync::Arc;
use std::sync::atomic::AtomicBool;

#[derive(Debug)]
pub struct ThreadStop {
    pub running_collect: Arc<AtomicBool>,
}