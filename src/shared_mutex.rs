use std::ops::{Deref, DerefMut};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};
use std::thread;

//TODO: delete all of this and put the normal Mutex<T> back as the type.
pub struct LoggingMutexGuard<'a, T: ?Sized> {
    id: usize,
    guard: MutexGuard<'a, T>,
}

impl<'a, T: ?Sized> LoggingMutexGuard<'a, T> {
    // Constructor for the custom guard
    fn new(id: usize, guard: MutexGuard<'a, T>) -> Self {
        LoggingMutexGuard { id, guard }
    }
}

// Implement Deref and DerefMut to provide access to the data
impl<'a, T: ?Sized> Deref for LoggingMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<'a, T: ?Sized> DerefMut for LoggingMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}

impl<'a, T: ?Sized> Drop for LoggingMutexGuard<'a, T> {
    fn drop(&mut self) {
        println!("Mutex {} is being unlocked {:?}", self.id, thread::current().id());
    }
}

pub struct LoggingMutex<T: ?Sized> {
    id: usize,
    mutex: Mutex<T>,
}

impl<T: ?Sized> LoggingMutex<T> {
    // Constructor for the custom mutex
    pub fn new(id: usize, data: T) -> Self
    where
        T: Sized,
    {
        LoggingMutex {
            mutex: Mutex::new(data),
            id
        }
    }

    // Custom lock method
    pub fn lock(&self) -> LockResult<LoggingMutexGuard<T>> {
        println!("Mutex {} is being locked by thread {:?}", self.id, thread::current().id());
        let guard = self.mutex.lock().unwrap();
        println!("Mutex {} is locked {:?}", self.id, thread::current().id());
        Ok(LoggingMutexGuard::new(self.id, guard))
    }
}

pub type SharedMutex<T> = Arc<LoggingMutex<T>>;

pub fn new_shared_mutex<T>(id: usize, data: T) -> SharedMutex<T> {
    Arc::new(LoggingMutex::new(id, data))
}
