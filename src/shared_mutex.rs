use std::sync::{Arc, LockResult, Mutex, MutexGuard};
use std::thread;

//TODO: delete all of this and put the normal Mutex<T> back as the type.
pub struct LoggingMutex<T: ?Sized> {
    mutex: Mutex<T>,
}

impl<T: ?Sized> LoggingMutex<T> {
    pub fn new(data: T) -> Self
    where
        T: Sized,
    {
        LoggingMutex {
            mutex: Mutex::new(data),
        }
    }

    pub fn lock(&self) -> LockResult<MutexGuard<T>> {
        println!("Thread_id {:?} Mutex is being locked", thread::current().id());
        self.mutex.lock()
    }
}

pub type SharedMutex<T> = Arc<LoggingMutex<T>>;

pub fn new_shared_mutex<T>(data: T) -> SharedMutex<T> {
    Arc::new(LoggingMutex::new(data))
}
