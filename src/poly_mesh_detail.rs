use cxx::UniquePtr;
use recast_sys::ffi::recast::rcPolyMeshDetail;

use std::pin::Pin;

use crate::{check_uptr_alloc, RecastError};

pub struct PolyMeshDetail {
    ptr: UniquePtr<rcPolyMeshDetail>
}

impl PolyMeshDetail {
    pub fn new() -> Result<PolyMeshDetail, RecastError> {
        let details = recast_sys::ffi::recast::new_poly_mesh_detail();
        Ok(PolyMeshDetail { ptr: check_uptr_alloc(details)? })
    }

    pub fn pin_mut(&mut self) -> Pin<&mut rcPolyMeshDetail> {
        self.ptr.pin_mut()
    }

    pub fn meshes(&self) -> &[u32] {
        let meshes_buffer = recast_sys::ffi::recast::poly_mesh_detail_meshes(self.as_ref());
        let n_meshes = recast_sys::ffi::recast::poly_mesh_detail_num_meshes(self.as_ref());
        unsafe { std::slice::from_raw_parts(meshes_buffer, (n_meshes * 4) as usize) }
    }

    pub fn vertices(&self) -> &[f32] {
        let vertices_buffer = recast_sys::ffi::recast::poly_mesh_detail_vertices(self.as_ref());
        let n_vertices = recast_sys::ffi::recast::poly_mesh_detail_num_vertices(self.as_ref());
        unsafe { std::slice::from_raw_parts(vertices_buffer, (n_vertices * 3) as usize) }
    }

    pub fn triangles(&self) -> &[u8] {
        let triangles_buffer = recast_sys::ffi::recast::poly_mesh_detail_triangles(self.as_ref());
        let n_triangles = recast_sys::ffi::recast::poly_mesh_detail_num_triangles(self.as_ref());
        unsafe { std::slice::from_raw_parts(triangles_buffer, (n_triangles * 4) as usize) }
    }
}

impl AsRef<rcPolyMeshDetail> for PolyMeshDetail {
    fn as_ref(&self) -> &rcPolyMeshDetail {
        self.ptr.as_ref().unwrap()
    }
}
