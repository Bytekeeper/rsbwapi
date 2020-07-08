use core::ops::Deref;
use core::ops::DerefMut;
use core::ptr::NonNull;
use std::ffi::CString;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::MapViewOfFile;
use winapi::um::memoryapi::FILE_MAP_READ;
use winapi::um::memoryapi::FILE_MAP_WRITE;
use winapi::um::winbase::OpenFileMappingA;
use winapi::um::winnt::HANDLE;

pub struct Shm<T: ?Sized>(HANDLE, NonNull<T>);

impl<T: ?Sized> Deref for Shm<T> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: Handing out a mutable ref to T should be safe
        unsafe { self.1.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Shm<T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: We're mutable, so handing out a mutable ref to T should be safe
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

pub fn map_memory<T>(name: &str) -> Option<Shm<T>> {
    let memory_size = std::mem::size_of::<T>();
    let lp_name = CString::new(name).unwrap();
    unsafe {
        let handle = OpenFileMappingA(FILE_MAP_READ | FILE_MAP_WRITE, FALSE, lp_name.as_ptr());
        let mapped =
            MapViewOfFile(handle, FILE_MAP_READ | FILE_MAP_WRITE, 0, 0, memory_size) as *mut T;
        Some(Shm(handle, NonNull::new_unchecked(mapped)))
    }
}
