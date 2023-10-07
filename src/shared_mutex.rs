use std::sync::{Arc, Mutex};

pub type SharedMutex<T> = Arc<Mutex<T>>;

pub fn new_shared_mutex<T>(data: T) -> SharedMutex<T> {
    Arc::new(Mutex::new(data))
}
