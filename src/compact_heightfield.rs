use cxx::UniquePtr;
use recast_sys::ffi::recast::rcCompactHeightfield;

use std::pin::Pin;

use crate::{RecastError, check_uptr_alloc};

pub struct CompactHeightField {
    ptr: UniquePtr<rcCompactHeightfield>
}

impl CompactHeightField {
    pub fn new() -> Result<CompactHeightField, RecastError> {
        let rc_compact_heightfield = recast_sys::ffi::recast::new_compact_heightfield();
        Ok(CompactHeightField { ptr: check_uptr_alloc(rc_compact_heightfield)? })
    }

    pub fn pin_mut(&mut self) -> Pin<&mut rcCompactHeightfield> {
        self.ptr.pin_mut()
    }
}

impl AsRef<rcCompactHeightfield> for CompactHeightField {
    fn as_ref(&self) -> &rcCompactHeightfield {
        self.ptr.as_ref().unwrap()
    }
}
