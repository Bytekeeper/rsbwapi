use core::ptr::NonNull;
use std::ffi::CString;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::MapViewOfFile;
use winapi::um::memoryapi::FILE_MAP_READ;
use winapi::um::memoryapi::FILE_MAP_WRITE;
use winapi::um::winbase::OpenFileMappingA;
use winapi::um::winnt::HANDLE;

pub(crate) struct Shm<T: ?Sized>(HANDLE, NonNull<T>);

impl<T> Shm<T> {
    pub(crate) fn get(&self) -> &T {
        // SAFETY: Shm is !Sync
        unsafe { self.1.as_ref() }
    }

    pub(crate) fn get_mut(&mut self) -> &mut T {
        // SAFETY: Shm is !Sync, and self is &mut
        unsafe { self.1.as_mut() }
    }
}

impl<T: ?Sized> Drop for Shm<T> {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub(crate) fn map_memory<T>(name: &str) -> Option<Shm<T>> {
    let memory_size = std::mem::size_of::<T>();
    let lp_name = CString::new(name).unwrap();
    unsafe {
        let handle = OpenFileMappingA(FILE_MAP_READ | FILE_MAP_WRITE, FALSE, lp_name.as_ptr());
        let mapped =
            MapViewOfFile(handle, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, memory_size) as *mut T;
        Some(Shm(handle, NonNull::new_unchecked(mapped)))
    }
}
