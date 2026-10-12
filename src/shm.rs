use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

#[cfg(any(unix, windows))]
use std::ffi::CString;
#[cfg(unix)]
use std::os::fd::FromRawFd;
#[cfg(windows)]
use winapi::shared::minwindef::FALSE;
#[cfg(windows)]
use winapi::um::handleapi::CloseHandle;
#[cfg(windows)]
use winapi::um::memoryapi::{FILE_MAP_READ, FILE_MAP_WRITE, MapViewOfFile};
#[cfg(windows)]
use winapi::um::winbase::OpenFileMappingA;
#[cfg(windows)]
use winapi::um::winnt::HANDLE;

#[cfg(windows)]
pub(crate) struct Shm<T: ?Sized>(HANDLE, NonNull<T>);

#[cfg(unix)]
pub(crate) struct Shm<T>(Option<std::fs::File>, NonNull<T>);

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
        #[cfg(unix)]
        return Self(None, NonNull::new(data.as_mut_ptr()).unwrap().cast());
        #[cfg(windows)]
        Self(
            std::ptr::null_mut(),
            NonNull::new(data.as_mut_ptr()).unwrap().cast(),
        )
    }
}

#[cfg(unix)]
impl<T> Drop for Shm<T> {
    fn drop(&mut self) {
        if self.0.is_some() {
            unsafe { libc::munmap(self.1.as_ptr().cast(), std::mem::size_of::<T>()) };
        }
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

#[cfg(unix)]
pub(crate) fn map_memory<T>(name: &str) -> Option<Shm<T>> {
    let name = CString::new(name).ok()?;
    let fd = unsafe { libc::shm_open(name.as_ptr(), libc::O_RDWR, 0) };
    if fd < 0 {
        return None;
    }
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    let size = std::mem::size_of::<T>();
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    if unsafe { libc::fstat(fd, stat.as_mut_ptr()) } != 0
        || unsafe { stat.assume_init().st_size } < size as libc::off_t
    {
        return None;
    }
    let mapped = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            size,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_SHARED,
            fd,
            0,
        )
    };
    if mapped == libc::MAP_FAILED {
        return None;
    }
    Some(Shm(Some(file), NonNull::new(mapped.cast())?))
}
