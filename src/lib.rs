#[cfg(feature = "detour")]
pub mod detour;

#[cfg(feature = "recast")]
pub mod recast;

mod macros;

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

pub fn check_uptr_alloc<T>(ptr: UniquePtr<T>) -> Result<UniquePtr<T>, RecastError>
where
    T: UniquePtrTarget,
{
    if ptr.is_null() {
        return Err(RecastError::OutOfMemoryError);
    }
    Ok(ptr)
}
