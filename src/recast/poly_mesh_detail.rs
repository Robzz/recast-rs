use crate::slice_from_raw_parts_or_dangling;

impl super::PolyMeshDetail {
    pub fn meshes(&self) -> &[u32] {
        let meshes_buffer = recast_sys::ffi::recast::poly_mesh_detail_meshes(self.as_ref());
        let n_meshes = recast_sys::ffi::recast::poly_mesh_detail_num_meshes(self.as_ref());
        slice_from_raw_parts_or_dangling(meshes_buffer, (n_meshes * 4) as usize)
    }

    pub fn vertices(&self) -> &[f32] {
        let vertices_buffer = recast_sys::ffi::recast::poly_mesh_detail_vertices(self.as_ref());
        let n_vertices = recast_sys::ffi::recast::poly_mesh_detail_num_vertices(self.as_ref());
        slice_from_raw_parts_or_dangling(vertices_buffer, (n_vertices * 3) as usize)
    }

    pub fn triangles(&self) -> &[u8] {
        let triangles_buffer = recast_sys::ffi::recast::poly_mesh_detail_triangles(self.as_ref());
        let n_triangles = recast_sys::ffi::recast::poly_mesh_detail_num_triangles(self.as_ref());
        slice_from_raw_parts_or_dangling(triangles_buffer, (n_triangles * 4) as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::recast::PolyMeshDetail;

    #[test]
    fn test_new_poly_mesh_detail_meshes_empty() {
        let detail = PolyMeshDetail::new().unwrap();
        assert!(detail.meshes().is_empty());
    }

    #[test]
    fn test_new_poly_mesh_detail_vertices_empty() {
        let detail = PolyMeshDetail::new().unwrap();
        assert!(detail.vertices().is_empty());
    }

    #[test]
    fn test_new_poly_mesh_detail_triangles_empty() {
        let detail = PolyMeshDetail::new().unwrap();
        assert!(detail.triangles().is_empty());
    }
}
