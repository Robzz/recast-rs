use cxx::UniquePtr;
use recast_sys::ffi::recast::rcContourSet;

use std::pin::Pin;

use crate::{check_uptr_alloc, RecastError};

pub struct ContourSet {
    ptr: UniquePtr<rcContourSet>
}

impl ContourSet {
    pub fn new() -> Result<ContourSet, RecastError> {
        let contours = recast_sys::ffi::recast::new_contour_set();
        Ok(ContourSet { ptr: check_uptr_alloc(contours)? })
    }

    pub fn pin_mut(&mut self) -> Pin<&mut rcContourSet> {
        self.ptr.pin_mut()
    }
}

impl AsRef<rcContourSet> for ContourSet {
    fn as_ref(&self) -> &rcContourSet {
        self.ptr.as_ref().unwrap()
    }
}
