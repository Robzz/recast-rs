#[cfg(feature = "detour")]
pub mod detour;

#[cfg(feature = "recast")]
pub mod recast;

mod macros;

use std::ptr::NonNull;

#[cfg(feature = "detour")]
use detour::Error as DetourError;
#[cfg(feature = "recast")]
use recast::RecastError;

use cxx::{private::UniquePtrTarget, UniquePtr};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "recast")]
    #[error("A recast error occurred: {0}")]
    Recast(#[from] RecastError),
    #[cfg(feature = "detour")]
    #[error("A detour error occurred: {0}")]
    Detour(#[from] DetourError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn check_uptr_alloc<T>(ptr: UniquePtr<T>) -> Result<UniquePtr<T>>
where
    T: UniquePtrTarget,
{
    if ptr.is_null() {
        return Err(RecastError::OutOfMemoryError)?;
    }
    Ok(ptr)
}

fn slice_from_raw_parts_or_dangling<'a, T>(data: *const T, len: usize) -> &'a [T] {
    if data.is_null() {
        unsafe { std::slice::from_raw_parts(NonNull::dangling().as_ptr(), 0usize) }
    }
    else {
        unsafe { std::slice::from_raw_parts(data, len) }
    }
}

fn slice_from_raw_parts_mut_or_dangling<'a, T>(data: *mut T, len: usize) -> &'a mut [T] {
    if data.is_null() {
        unsafe { std::slice::from_raw_parts_mut(NonNull::dangling().as_ptr(), 0usize) }
    }
    else {
        unsafe { std::slice::from_raw_parts_mut(data, len) }
    }
}
