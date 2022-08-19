use crate::recast::{PolyMesh, PolyMeshDetail};

pub struct RecastNavMeshData {
    pub poly_mesh: PolyMesh,
    pub detail: PolyMeshDetail
}

#[cfg(feature = "detour")]
impl From<&RecastNavMeshData> for recast_sys::ffi::detour::NavMeshCreateParams {
    fn from(data: &RecastNavMeshData) -> recast_sys::ffi::detour::NavMeshCreateParams {
        let mut params = recast_sys::ffi::detour::NavMeshCreateParams::from(&data.poly_mesh);

        params.detail_meshes = recast_sys::ffi::recast::poly_mesh_detail_meshes(data.detail.as_ref());
        params.detail_vertices = recast_sys::ffi::recast::poly_mesh_detail_vertices(data.detail.as_ref());
        params.num_detail_vertices = recast_sys::ffi::recast::poly_mesh_detail_num_vertices(data.detail.as_ref());
        params.detail_triangles = recast_sys::ffi::recast::poly_mesh_detail_triangles(data.detail.as_ref());
        params.num_detail_triangles = recast_sys::ffi::recast::poly_mesh_detail_num_triangles(data.detail.as_ref());

        params
    }
}
