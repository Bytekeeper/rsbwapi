use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

#[cfg(windows)]
use std::ffi::CString;
#[cfg(windows)]
use winapi::shared::minwindef::FALSE;
#[cfg(windows)]
use winapi::um::handleapi::CloseHandle;
#[cfg(windows)]
use winapi::um::memoryapi::{MapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE};
#[cfg(windows)]
use winapi::um::winbase::OpenFileMappingA;
#[cfg(windows)]
use winapi::um::winnt::HANDLE;

#[cfg(windows)]
pub(crate) struct Shm<T: ?Sized>(HANDLE, NonNull<T>);

#[cfg(not(windows))]
pub(crate) struct Shm<T: ?Sized>((), NonNull<T>);

impl<T> Deref for Shm<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.1.as_ref() }
    }
}

impl<T> DerefMut for Shm<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.1.as_mut() }
    }
}

impl<T> Shm<T> {
    pub(crate) fn as_ptr(&self) -> *const T {
        self.1.as_ptr()
    }

    pub(crate) fn get(&self) -> &T {
        // Not safe at all
        unsafe { self.1.as_ref() }
    }

    #[cfg(test)]
    pub fn from_mut_slice(data: &mut [u8]) -> Shm<T> {
        Self((), NonNull::new(data.as_mut_ptr()).unwrap().cast())
    }
}

#[cfg(windows)]
impl<T: ?Sized> Drop for Shm<T> {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

#[cfg(windows)]
pub(crate) fn map_memory<T>(name: &str) -> Option<Shm<T>> {
    let memory_size = std::mem::size_of::<T>();
    let lp_name = CString::new(name).unwrap();
    unsafe {
        let handle = OpenFileMappingA(FILE_MAP_READ | FILE_MAP_WRITE, FALSE, lp_name.as_ptr());
        if handle.is_null() {
            // BWAPI Server is most likely not running yet
            return None;
        }
        let mapped =
            MapViewOfFile(handle, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, memory_size) as *mut T;
        Some(Shm(handle, NonNull::new_unchecked(mapped)))
    }
}

#[cfg(not(windows))]
pub(crate) fn map_memory<T>(_name: &str) -> Option<Shm<T>> {
    None
}
