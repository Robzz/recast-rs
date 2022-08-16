use cxx::UniquePtr;
use recast_sys::ffi::recast::rcHeightfield;

use std::pin::Pin;

use crate::{RecastError, check_uptr_alloc};

pub struct HeightField {
    ptr: UniquePtr<rcHeightfield>
}

impl HeightField {
    pub fn new() -> Result<HeightField, RecastError> {
        let rc_heightfield = recast_sys::ffi::recast::new_heightfield();
        Ok(HeightField { ptr: check_uptr_alloc(rc_heightfield)? })
    }

    pub fn pin_mut(&mut self) -> Pin<&mut rcHeightfield> {
        self.ptr.pin_mut()
    }
}

impl AsRef<rcHeightfield> for HeightField {
    fn as_ref(&self) -> &rcHeightfield {
        self.ptr.as_ref().unwrap()
    }
}
