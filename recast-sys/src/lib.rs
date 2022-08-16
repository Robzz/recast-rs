pub struct RecastConfig {
    pub tile_size: i32,
    pub border_size: i32,
    /// Cell size on the XZ plane.
    pub cs: f32,
    /// Vertical cell size.
    pub ch: f32,
    /// Minimum boundary of the AABB containing the input geometry.
    pub bmin: [f32; 3],
    /// Maximum boundary of the AABB containing the input geometry.
    pub bmax: [f32; 3],
    pub walkable_slope_angle: f32,
    pub walkable_height: i32,
    pub walkable_climb: i32,
    pub walkable_radius: i32,
    pub max_edge_len: i32,
    pub max_simplification_error: f32,
    pub min_region_area: i32,
    pub merge_region_area: i32,
    pub max_verts_per_poly: i32,
    pub details_sample_dist: f32,
    pub details_sample_max_error: f32,
}

impl Default for RecastConfig {
    fn default() -> Self {
        RecastConfig {
            tile_size: 0,
            border_size: 0,
            cs: 0.3,
            ch: 0.2,
            bmin: [0.; 3],
            bmax: [0.; 3],
            walkable_slope_angle: 45.,
            walkable_height: 10,
            walkable_climb: 4,
            walkable_radius: 3,
            max_edge_len: 40,
            max_simplification_error: 1.3,
            min_region_area: 8,
            merge_region_area: 20,
            max_verts_per_poly: 6,
            details_sample_dist: 6.,
            details_sample_max_error: 1.,
        }
    }
}

/// The module containing the Recast FFI definitions.
///
/// We do not expose the `dt/rcAlloc*` functions and expose our own which
/// construct a `UniquePtr` instead, as these are safer and more convenient to use.
#[allow(clippy::too_many_arguments)]
pub mod ffi {
    #[cxx::bridge]
    #[cfg(feature = "recast")]
    pub mod recast {
        unsafe extern "C++" {
            include!("recast-sys/include/recast.h");

            /// Recast context.
            pub type rcContext;

            /// Recast heightfield.
            pub type rcHeightfield;

            /// Recast compact heightfield.
            pub type rcCompactHeightfield;

            /// Recast contour set.
            pub type rcContourSet;

            /// Recast polygon mesh.
            pub type rcPolyMesh;

            /// Recast polygon mesh height detail.
            pub type rcPolyMeshDetail;

            #[rust_name = "calc_grid_size"]
            pub unsafe fn rcCalcGridSize(
                bmin: *const f32,
                bmax: *const f32,
                cs: f32,
                width: *mut i32,
                height: *mut i32,
            );

            #[rust_name = "new_context"]
            pub fn newRcContext(diagnostics: bool) -> UniquePtr<rcContext>;

            #[rust_name = "new_heightfield"]
            pub fn newRcHeightfield() -> UniquePtr<rcHeightfield>;

            #[rust_name = "new_compact_heightfield"]
            pub fn newRcCompactHeightfield() -> UniquePtr<rcCompactHeightfield>;

            #[rust_name = "new_contour_set"]
            pub fn newRcContourSet() -> UniquePtr<rcContourSet>;

            #[rust_name = "new_poly_mesh"]
            pub fn newRcPolyMesh() -> UniquePtr<rcPolyMesh>;

            #[rust_name = "new_poly_mesh_detail"]
            pub fn newRcPolyMeshDetail() -> UniquePtr<rcPolyMeshDetail>;

            #[rust_name = "free_heightfield"]
            /// Free a heightfield allocated by Recast.
            ///
            /// # Safety
            ///
            /// The pointer passed to this method must have been obtained by the `new_heightfield`
            /// function and must not have been already free'd.
            pub unsafe fn rcFreeHeightField(heightfield: *mut rcHeightfield);

            #[rust_name = "free_compact_heightfield"]
            /// Free a `rcCompactHeightfield` allocated by Recast.
            ///
            /// # Safety
            ///
            /// The pointer passed to this method must have been obtained by the `new_compact_heightfield`
            /// function and must not have been already free'd.
            pub unsafe fn rcFreeCompactHeightfield(heightfield: *mut rcCompactHeightfield);

            #[rust_name = "free_contour_set"]
            /// Free a `rcContourSet` allocated by Recast.
            ///
            /// # Safety
            ///
            /// The pointer passed to this method must have been obtained by the `new_contour_set`
            /// function and must not have been already free'd.
            pub unsafe fn rcFreeContourSet(heightfield: *mut rcContourSet);

            #[rust_name = "free_poly_mesh"]
            /// Free a `rcPolyMesh` allocated by Recast.
            ///
            /// # Safety
            ///
            /// The pointer passed to this method must have been obtained by the
            /// `new_poly_mesh` function and must not have been already free'd.
            pub unsafe fn rcFreePolyMesh(poly_mesh: *mut rcPolyMesh);

            #[rust_name = "free_poly_mesh_detail"]
            /// Free a `rcPolyMeshDetail` allocated by Recast.
            ///
            /// # Safety
            ///
            /// The pointer passed to this method must have been obtained by the
            /// `new_poly_mesh_detail` function and must not have been already free'd.
            pub unsafe fn rcFreePolyMeshDetail(heightfield: *mut rcPolyMeshDetail);

            #[rust_name = "create_heightfield"]
            pub unsafe fn rcCreateHeightfield(
                context: *mut rcContext,
                heightfield: Pin<&mut rcHeightfield>,
                width: i32,
                height: i32,
                b_min: *const f32,
                b_max: *const f32,
                cs: f32,
                ch: f32,
            ) -> bool;

            #[rust_name = "mark_walkable_triangles"]
            pub unsafe fn rcMarkWalkableTriangles(
                context: *mut rcContext,
                walkable_slope_angle: f32,
                vertices: *const f32,
                n_vertices: i32,
                indices: *const i32,
                n_triangles: i32,
                areas: *mut u8,
            );

            #[rust_name = "rasterize_triangles_with_indices"]
            pub unsafe fn rcRasterizeTriangles(
                context: *mut rcContext,
                vertices: *const f32,
                n_vertices: i32,
                indices: *const i32,
                areas: *const u8,
                n_triangles: i32,
                heightfield: Pin<&mut rcHeightfield>,
                flag_merge_threshold: i32,
            ) -> bool;

            #[rust_name = "filter_low_hanging_walkable_obstacles"]
            pub unsafe fn rcFilterLowHangingWalkableObstacles(
                context: *mut rcContext,
                walkable_climb: i32,
                heightfield: Pin<&mut rcHeightfield>,
            );

            #[rust_name = "filter_ledge_spans"]
            pub unsafe fn rcFilterLedgeSpans(
                context: *mut rcContext,
                walkable_height: i32,
                walkable_climb: i32,
                heightfield: Pin<&mut rcHeightfield>,
            );

            #[rust_name = "filter_walkable_low_height_spans"]
            pub unsafe fn rcFilterWalkableLowHeightSpans(
                context: *mut rcContext,
                walkable_height: i32,
                heightfield: Pin<&mut rcHeightfield>,
            );

            #[rust_name = "build_compact_heightfield"]
            pub unsafe fn rcBuildCompactHeightfield(
                context: *mut rcContext,
                walkable_height: i32,
                walkable_climb: i32,
                heightfield: Pin<&mut rcHeightfield>,
                compact: Pin<&mut rcCompactHeightfield>,
            ) -> bool;

            #[rust_name = "erode_walkable_area"]
            pub unsafe fn rcErodeWalkableArea(
                context: *mut rcContext,
                radius: i32,
                heightfield: Pin<&mut rcCompactHeightfield>,
            ) -> bool;

            #[rust_name = "build_distance_field"]
            pub unsafe fn rcBuildDistanceField(
                context: *mut rcContext,
                heightfield: Pin<&mut rcCompactHeightfield>,
            ) -> bool;

            #[rust_name = "build_regions"]
            pub unsafe fn rcBuildRegions(
                context: *mut rcContext,
                heightfield: Pin<&mut rcCompactHeightfield>,
                border_size: i32,
                min_region_area: i32,
                merge_region_area: i32,
            ) -> bool;

            #[rust_name = "build_contours"]
            pub unsafe fn rcBuildContours(
                context: *mut rcContext,
                heightfield: Pin<&mut rcCompactHeightfield>,
                max_error: f32,
                max_edge_len: i32,
                contour_set: Pin<&mut rcContourSet>,
                build_flags: i32,
            ) -> bool;

            #[rust_name = "build_poly_mesh"]
            pub unsafe fn rcBuildPolyMesh(
                context: *mut rcContext,
                contour_set: Pin<&mut rcContourSet>,
                max_vertices_per_poly: i32,
                poly_mesh: Pin<&mut rcPolyMesh>,
            ) -> bool;

            #[rust_name = "build_poly_mesh_detail"]
            pub unsafe fn rcBuildPolyMeshDetail(
                context: *mut rcContext,
                poly_mesh: &rcPolyMesh,
                heightfield: &rcCompactHeightfield,
                sample_distance: f32,
                sample_max_error: f32,
                detail: Pin<&mut rcPolyMeshDetail>,
            ) -> bool;

            #[rust_name = "poly_mesh_get_vertices"]
            pub fn polyMeshGetVerts(poly_mesh: &rcPolyMesh) -> *const u16;

            #[rust_name = "poly_mesh_get_polys"]
            pub fn polyMeshGetPolys(poly_mesh: &rcPolyMesh) -> *const u16;

            #[rust_name = "poly_mesh_get_regions"]
            pub fn polyMeshGetRegions(poly_mesh: &rcPolyMesh) -> *const u16;

            #[rust_name = "poly_mesh_get_areas"]
            pub fn polyMeshGetAreas(poly_mesh: &rcPolyMesh) -> *const u8;

            #[rust_name = "poly_mesh_get_poly_count"]
            pub fn polyMeshGetPolyCount(poly_mesh: &rcPolyMesh) -> i32;

            #[rust_name = "poly_mesh_get_vertex_count"]
            pub fn polyMeshGetVertexCount(poly_mesh: &rcPolyMesh) -> i32;

            #[rust_name = "poly_mesh_max_vertex_count_per_poly"]
            pub fn polyMeshGetMaxVertexCountPerPoly(poly_mesh: &rcPolyMesh) -> i32;

            #[rust_name = "poly_mesh_detail_num_meshes"]
            pub fn polyMeshDetailGetNumMeshes(detail: &rcPolyMeshDetail) -> i32;

            #[rust_name = "poly_mesh_detail_num_vertices"]
            pub fn polyMeshDetailGetNumVerts(detail: &rcPolyMeshDetail) -> i32;

            #[rust_name = "poly_mesh_detail_num_triangles"]
            pub fn polyMeshDetailGetNumTris(detail: &rcPolyMeshDetail) -> i32;

            #[rust_name = "poly_mesh_detail_meshes"]
            pub fn polyMeshDetailGetMeshes(detail: &rcPolyMeshDetail) -> *const u32;

            #[rust_name = "poly_mesh_detail_vertices"]
            pub fn polyMeshDetailGetVerts(detail: &rcPolyMeshDetail) -> *const f32;

            #[rust_name = "poly_mesh_detail_triangles"]
            pub fn polyMeshDetailGetTris(detail: &rcPolyMeshDetail) -> *const u8;
        }
    }

    #[cxx::bridge]
    #[cfg(feature = "recast")]
    pub mod detour {
    }
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "recast")]
    mod recast {
        use crate::ffi::recast;

        #[test]
        fn test_new_heightfield_not_null() {
            let heightfield = recast::new_heightfield();
            assert!(!heightfield.is_null());
        }

        #[test]
        fn test_new_compact_heightfield_not_null() {
            let heightfield = recast::new_compact_heightfield();
            assert!(!heightfield.is_null());
        }

        #[test]
        fn test_new_contour_set_not_null() {
            let contour_set = recast::new_contour_set();
            assert!(!contour_set.is_null());
        }

        #[test]
        fn test_new_poly_mesh_not_null() {
            let poly_mesh = recast::new_poly_mesh();
            assert!(!poly_mesh.is_null());
        }

        #[test]
        fn test_new_poly_mesh_details_not_null() {
            let detail = recast::new_poly_mesh_detail();
            assert!(!detail.is_null());
        }

        #[test]
        fn test_new_context_not_null() {
            let ctx = recast::new_context(false);
            assert!(!ctx.is_null());
        }
    }
}
