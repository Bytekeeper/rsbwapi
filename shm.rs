extern crate winapi;

use winapi::shared::minwindef::FALSE;
use winapi::um::memoryapi::FILE_MAP_READ;
use winapi::um::memoryapi::FILE_MAP_WRITE;
use winapi::um::memoryapi::MapViewOfFile;
use winapi::um::winbase::OpenFileMappingA;
use std::ffi::CString;

pub fn mapMemory<T>(name: &str) -> &mut T {
    let memorySize = std::mem::size_of::<T>();
    let lpName = CString::new(name).unwrap();
    unsafe {
        let mapped = MapViewOfFile(
            OpenFileMappingA(FILE_MAP_READ | FILE_MAP_WRITE, FALSE, lpName.as_ptr()),
            FILE_MAP_READ | FILE_MAP_WRITE,
            0, 0,
            memorySize,
        ) as *mut T;
        mapped.as_mut().unwrap()
    }
}