use cxx::UniquePtr;
use recast_sys::ffi::recast::rcPolyMesh;

use std::pin::Pin;

use crate::{RecastError, check_uptr_alloc};

pub struct PolyMesh {
    ptr: UniquePtr<rcPolyMesh>
}

impl PolyMesh {
    pub fn new() -> Result<PolyMesh, RecastError> {
        let poly_mesh = recast_sys::ffi::recast::new_poly_mesh();
        Ok(PolyMesh { ptr: check_uptr_alloc(poly_mesh)? })
    }

    pub fn pin_mut(&mut self) -> Pin<&mut rcPolyMesh> {
        self.ptr.pin_mut()
    }

    pub fn vertices(&self) -> &[u16] {
        let vertices_buffer = recast_sys::ffi::recast::poly_mesh_get_vertices(self.as_ref());
        let n_vertices = recast_sys::ffi::recast::poly_mesh_get_vertex_count(self.as_ref());
        unsafe { std::slice::from_raw_parts(vertices_buffer, (n_vertices * 3) as usize) }
    }

    pub fn polygons(&self) -> &[u16] {
        let polygons_buffer = recast_sys::ffi::recast::poly_mesh_get_polys(self.as_ref());
        let n_polys = recast_sys::ffi::recast::poly_mesh_get_poly_count(self.as_ref());
        let n_vpp = recast_sys::ffi::recast::poly_mesh_max_vertex_count_per_poly(self.as_ref());
        unsafe { std::slice::from_raw_parts(polygons_buffer, (n_polys * 2 * n_vpp) as usize) }
    }

    pub fn regions(&self) -> &[u16] {
        let regions_buffer = recast_sys::ffi::recast::poly_mesh_get_regions(self.as_ref());
        let n_polys = recast_sys::ffi::recast::poly_mesh_get_poly_count(self.as_ref());
        unsafe { std::slice::from_raw_parts(regions_buffer, n_polys as usize) }
    }

    pub fn areas(&self) -> &[u8] {
        let areas_buffer = recast_sys::ffi::recast::poly_mesh_get_areas(self.as_ref());
        let n_polys = recast_sys::ffi::recast::poly_mesh_get_poly_count(self.as_ref());
        unsafe { std::slice::from_raw_parts(areas_buffer, n_polys as usize) }
    }
}

impl AsRef<rcPolyMesh> for PolyMesh {
    fn as_ref(&self) -> &rcPolyMesh {
        self.ptr.as_ref().unwrap()
    }
}
