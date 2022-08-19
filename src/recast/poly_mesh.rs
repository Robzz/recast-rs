#[cfg(feature = "detour")]
use recast_sys::ffi::detour::NavMeshCreateParams;

use super::PolyMesh;

impl PolyMesh {
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

    pub fn flags(&self) -> &[u16] {
        let flags_buffer = recast_sys::ffi::recast::poly_mesh_get_flags(self.as_ref());
        let n_polys = recast_sys::ffi::recast::poly_mesh_get_poly_count(self.as_ref());
        unsafe { std::slice::from_raw_parts(flags_buffer, n_polys as usize) }
    }

    pub fn flags_mut(&mut self) -> &mut [u16] {
        let flags_buffer = recast_sys::ffi::recast::poly_mesh_get_flags_mut(self.pin_mut());
        let n_polys = recast_sys::ffi::recast::poly_mesh_get_poly_count(self.as_ref());
        unsafe { std::slice::from_raw_parts_mut(flags_buffer, n_polys as usize) }
    }

    pub fn areas(&self) -> &[u8] {
        let areas_buffer = recast_sys::ffi::recast::poly_mesh_get_areas(self.as_ref());
        let n_polys = recast_sys::ffi::recast::poly_mesh_get_poly_count(self.as_ref());
        unsafe { std::slice::from_raw_parts(areas_buffer, n_polys as usize) }
    }
}

#[cfg(feature = "detour")]
impl From<&PolyMesh> for NavMeshCreateParams {
    fn from(mesh: &PolyMesh) -> Self {
        NavMeshCreateParams {
            vertices: recast_sys::ffi::recast::poly_mesh_get_vertices(mesh.as_ref()),
            num_vertices: recast_sys::ffi::recast::poly_mesh_get_vertex_count(mesh.as_ref()),
            polygons: recast_sys::ffi::recast::poly_mesh_get_polys(mesh.as_ref()),
            polygon_flags: recast_sys::ffi::recast::poly_mesh_get_flags(mesh.as_ref()),
            polygon_areas: recast_sys::ffi::recast::poly_mesh_get_areas(mesh.as_ref()),
            num_polys: recast_sys::ffi::recast::poly_mesh_get_poly_count(mesh.as_ref()),
            max_vertices_per_poly: recast_sys::ffi::recast::poly_mesh_max_vertex_count_per_poly(mesh.as_ref()),
            detail_meshes: std::ptr::null(),
            detail_vertices: std::ptr::null(),
            num_detail_vertices: 0,
            detail_triangles: std::ptr::null(),
            num_detail_triangles: 0,
            off_mesh_conn_vertices: std::ptr::null(),
            off_mesh_conn_radii: std::ptr::null(),
            off_mesh_conn_flags: std::ptr::null(),
            off_mesh_conn_areas: std::ptr::null(),
            off_mesh_conn_dir: std::ptr::null(),
            off_mesh_conn_ids: std::ptr::null(),
            off_mesh_conn_count: 0,
            user_id: 0,
            tile_x: 0,
            tile_y: 0,
            tile_layer: 0,
            b_min: [0.; 3],
            b_max: [0.; 3],
            walkable_height: 0.,
            walkable_radius: 0.,
            walkable_climb: 0.,
            cs: 0.,
            ch: 0.,
            build_bv_tree: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::recast::PolyMesh;

    #[test]
    fn test_new_poly_mesh_vertices_empty() {
        let poly_mesh = PolyMesh::new().unwrap();
        assert!(poly_mesh.vertices().is_empty());
    }

    #[test]
    fn test_new_poly_mesh_polygons_empty() {
        let poly_mesh = PolyMesh::new().unwrap();
        assert!(poly_mesh.polygons().is_empty());
    }

    #[test]
    fn test_new_poly_mesh_regions_empty() {
        let poly_mesh = PolyMesh::new().unwrap();
        assert!(poly_mesh.regions().is_empty());
    }

    #[test]
    fn test_new_poly_mesh_flags_empty() {
        let poly_mesh = PolyMesh::new().unwrap();
        assert!(poly_mesh.flags().is_empty());
    }

    #[test]
    fn test_new_poly_mesh_areas_empty() {
        let poly_mesh = PolyMesh::new().unwrap();
        assert!(poly_mesh.areas().is_empty());
    }
}
