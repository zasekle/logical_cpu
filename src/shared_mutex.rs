use std::ops::{Deref, DerefMut};
use std::sync::{Arc, LockResult, Mutex, MutexGuard};
use std::thread;

pub struct LoggingMutexGuard<'a, T: ?Sized> {
    id: i32,
    guard: Option<MutexGuard<'a, T>>,
}

impl<'a, T: ?Sized> LoggingMutexGuard<'a, T> {
    // Constructor for the custom guard
    fn new(id: i32, guard: MutexGuard<'a, T>) -> Self {
        LoggingMutexGuard { id, guard: Some(guard) }
    }

    pub fn take_guard(&mut self) -> MutexGuard<'a, T> {
        self.guard.take().expect("Could not take guard, it was already moved")
    }
}

// Implement Deref and DerefMut to provide access to the data
impl<'a, T: ?Sized> Deref for LoggingMutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().expect("Failed to deref guard (it was moved out)")
    }
}

impl<'a, T: ?Sized> DerefMut for LoggingMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.as_mut().expect("Failed to deref_mut guard (it was moved out)")
    }
}

impl<'a, T: ?Sized> Drop for LoggingMutexGuard<'a, T> {
    fn drop(&mut self) {
        println!("Mutex {} is being unlocked {:?}", self.id, thread::current().id());
    }
}

pub struct LoggingMutex<T: ?Sized> {
    id: i32,
    mutex: Mutex<T>,
}

impl<T: ?Sized> LoggingMutex<T> {
    // Constructor for the custom mutex
    pub fn new(id: i32, data: T) -> Self
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

//TODO: set this back to type Mutex
pub type UsedMutex<T> = Mutex<T>;
// pub type UsedMutex<T> = LoggingMutex<T>;

pub fn new_used_mutex<T>(id: i32, data: T) -> UsedMutex<T> {
    Mutex::new(data)
    // LoggingMutex::new(id, data)
}

pub type SharedMutex<T> = Arc<UsedMutex<T>>;

pub fn new_shared_mutex<T>(id: usize, data: T) -> SharedMutex<T> {
    Arc::new(new_used_mutex(id as i32, data))
}

