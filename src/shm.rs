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
pub(crate) struct Shm<T: ?Sized>(usize, NonNull<T>);

impl<T> Shm<T> {
    pub(crate) fn get(&self) -> &T {
        // Not safe at all
        unsafe { self.1.as_ref() }
    }

    pub(crate) fn get_mut(&mut self) -> &mut T {
        // Not safe at all
        unsafe { self.1.as_mut() }
    }

    #[cfg(test)]
    pub fn from_mut_slice(data: &mut [u8]) -> Shm<T> {
        Self(data.len(), NonNull::new(data.as_mut_ptr()).unwrap().cast())
    }
}

impl<T: ?Sized> Drop for Shm<T> {
    fn drop(&mut self) {
        #[cfg(windows)]
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
