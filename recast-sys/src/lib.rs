/// Contains the configuration values for the full Recast + Detour navmesh generation pipeline.
#[derive(Debug, Clone, PartialEq)]
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
    pub filter_low_hanging_walkable_obstacles: bool,
    pub filter_ledge_spans: bool,
    pub filter_walkable_low_height_spans: bool,
    /// The maximum number of vertices per polygon in the output data.
    ///
    /// *NOTE*: Detour only supports navmeshes of up to 6 vertices per polygon. This is the default
    /// value.
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
            filter_low_hanging_walkable_obstacles: true,
            filter_ledge_spans: true,
            filter_walkable_low_height_spans: true,
            max_verts_per_poly: 6,
            details_sample_dist: 6.,
            details_sample_max_error: 1.,
        }
    }
}

/// The module containing the Recast libraries FFI definitions.
///
/// The functions exposed are intended to be as close as possible to a 1:1 correspondence to the
/// Recast API, but there might be some minor differences in places. This also means that using
/// this API from Rust safely will be fairly unergonomic, it is therefore highly recommended to use
/// the wrapper `recast-rs` crate to access this functionality.
///
/// This crate does not expose the `dt/rcAlloc*` functions and exposes its own constructors
/// returning a `UniquePtr` instead, as these are safer and more convenient to use.
///
/// Many of these functions deal with raw pointers and are therefore unsafe by nature.
/// In general, using them safely boils down to passing in properly initialized data along with
/// correct size information.
///
/// When in doubt, check the official Recast documentation for more information on what a specific
/// function does.
// TODO: safety preconditions will need to be documented before release
#[allow(clippy::too_many_arguments, clippy::missing_safety_doc)]
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
            ///
            /// This type does not automatically free the memory it owns, so you should either
            /// remember to call [`rcFreePolyMeshDetail`] yourself, or use
            /// [`rcPolyMeshDetailOwned`] instead which will do it for you.
            pub type rcPolyMeshDetail;

            /// Recast polygon mesh height detail smart pointer.
            pub type rcPolyMeshDetailOwned;

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

            #[rust_name = "new_poly_mesh_detail_owned"]
            /// This type and method don't exist in the upstream Recast API, this type is
            /// equivalent to `rcPolyMeshDetail`, except it has a destructor invoking
            /// `rcFreePolyMesh` so you don't have to do it yourself.
            pub fn newRcPolyMeshDetailOwned() -> UniquePtr<rcPolyMeshDetailOwned>;

            #[rust_name = "free_heightfield"]
            /// Free a heightfield allocated by Recast.
            ///
            /// # Safety
            ///
            /// The pointer passed to this method must have been obtained by the `new_heightfield`
            /// function and must not have been already free'd.
            pub unsafe fn rcFreeHeightField(heightfield: *mut rcHeightfield);

            #[rust_name = "free_poly_mesh_detail"]
            /// Free a rcPolyMeshDetail allocated by Recast.
            ///
            /// # Safety
            ///
            /// The pointer passed to this method must have been obtained by the
            /// `new_poly_mesh_detail` function and must not have been already free'd.
            pub unsafe fn rcFreePolyMeshDetail(detail: *mut rcPolyMeshDetail);

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

            #[rust_name = "rasterize_triangles"]
            pub unsafe fn rcRasterizeTriangles(
                context: *mut rcContext,
                vertices: *const f32,
                areas: *const u8,
                n_vertices: i32,
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

            #[rust_name = "poly_mesh_detail_owned_get_inner"]
            pub fn getInner(self: &rcPolyMeshDetailOwned) -> &rcPolyMeshDetail;

            #[rust_name = "poly_mesh_detail_owned_get_inner_mut"]
            pub fn getInner(self: Pin<&mut rcPolyMeshDetailOwned>) -> Pin<&mut rcPolyMeshDetail>;

            #[rust_name = "poly_mesh_get_vertices"]
            pub fn polyMeshGetVerts(poly_mesh: &rcPolyMesh) -> *const u16;

            #[rust_name = "poly_mesh_get_vertices_mut"]
            pub fn polyMeshGetVertsMut(poly_mesh: Pin<&mut rcPolyMesh>) -> *mut u16;

            #[rust_name = "poly_mesh_get_polys"]
            pub fn polyMeshGetPolys(poly_mesh: &rcPolyMesh) -> *const u16;

            #[rust_name = "poly_mesh_get_polys_mut"]
            pub fn polyMeshGetPolysMut(poly_mesh: Pin<&mut rcPolyMesh>) -> *mut u16;

            #[rust_name = "poly_mesh_get_regions"]
            pub fn polyMeshGetRegions(poly_mesh: &rcPolyMesh) -> *const u16;

            #[rust_name = "poly_mesh_get_regions_mut"]
            pub fn polyMeshGetRegionsMut(poly_mesh: Pin<&mut rcPolyMesh>) -> *mut u16;

            #[rust_name = "poly_mesh_get_flags"]
            pub fn polyMeshGetFlags(poly_mesh: &rcPolyMesh) -> *const u16;

            #[rust_name = "poly_mesh_get_flags_mut"]
            pub fn polyMeshGetFlagsMut(poly_mesh: Pin<&mut rcPolyMesh>) -> *mut u16;

            #[rust_name = "poly_mesh_get_areas"]
            pub fn polyMeshGetAreas(poly_mesh: &rcPolyMesh) -> *const u8;

            #[rust_name = "poly_mesh_get_areas_mut"]
            pub fn polyMeshGetAreasMut(poly_mesh: Pin<&mut rcPolyMesh>) -> *mut u8;

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
    #[cfg(feature = "detour")]
    pub mod detour {
        struct NavMeshCreateParams {
            // PolyMesh
            vertices: *const u16,
            num_vertices: i32,
            polygons: *const u16,
            polygon_flags: *const u16,
            polygon_areas: *const u8,
            num_polys: i32,
            max_vertices_per_poly: i32,
            // PolyMeshDetail
            detail_meshes: *const u32,
            detail_vertices: *const f32,
            num_detail_vertices: i32,
            detail_triangles: *const u8,
            num_detail_triangles: i32,
            // Off-mesh connections (optional)
            off_mesh_conn_vertices: *const f32,
            off_mesh_conn_radii: *const f32,
            off_mesh_conn_flags: *const u16,
            off_mesh_conn_areas: *const u8,
            off_mesh_conn_dir: *const u8,
            off_mesh_conn_ids: *const u32,
            off_mesh_conn_count: i32,
            // Tile attributes
            user_id: u32,
            tile_x: i32,
            tile_y: i32,
            tile_layer: i32,
            b_min: [f32; 3],
            b_max: [f32; 3],
            // General configuration
            walkable_height: f32,
            walkable_radius: f32,
            walkable_climb: f32,
            cs: f32,
            ch: f32,
            build_bv_tree: bool,
        }

        #[repr(i32)]
        enum dtTileFlags {
            #[rust_name = "FreeData"]
            DT_TILE_FREE_DATA = 1,
        }

        #[repr(i32)]
        enum dtStraightPathOptions {
            #[rust_name = "AreaCrossings"]
            DT_STRAIGHTPATH_AREA_CROSSINGS = 1,
            #[rust_name = "AllCrossings"]
            DT_STRAIGHTPATH_ALL_CROSSINGS = 2,
        }

        unsafe extern "C++" {
            include!("recast-sys/include/detour.h");

            #[cfg(feature = "detour_crowd")]
            include!("recast-sys/include/detour_crowd.h");

            type dtNavMesh;
            type dtNavMeshQuery;
            type dtTileFlags;
            type dtStraightPathOptions;
            type dtQueryFilter;

            #[cfg(feature = "detour_crowd")]
            type dtPathCorridor;

            #[rust_name = "new_navmesh"]
            pub fn newDtNavMesh() -> UniquePtr<dtNavMesh>;

            #[rust_name = "new_navmesh_query"]
            pub fn newDtNavMeshQuery() -> UniquePtr<dtNavMeshQuery>;

            #[rust_name = "new_query_filter"]
            pub fn newDtQueryFilter() -> UniquePtr<dtQueryFilter>;

            #[rust_name = "new_path_corridor"]
            #[cfg(feature = "detour_crowd")]
            pub fn newDtPathCorridor() -> UniquePtr<dtPathCorridor>;

            #[rust_name = "create_navmesh_data"]
            pub unsafe fn createNavMeshData(
                params: *mut NavMeshCreateParams,
                out_data: *mut *mut u8,
                out_data_size: *mut i32,
            ) -> bool;

            #[rust_name = "init"]
            pub unsafe fn init(
                self: Pin<&mut dtNavMesh>,
                data: *mut u8,
                data_len: i32,
                flags: i32,
            ) -> u32;

            #[rust_name = "init"]
            pub unsafe fn init(
                self: Pin<&mut dtNavMeshQuery>,
                navmesh: *const dtNavMesh,
                max_nodes: i32,
            ) -> u32;

            #[rust_name = "closest_point_on_poly"]
            pub unsafe fn closestPointOnPoly(
                self: &dtNavMeshQuery,
                ref_: u32,
                pos: *const f32,
                closest: *mut f32,
                pos_over_poly: *mut bool,
            ) -> u32;

            #[rust_name = "find_nearest_poly"]
            pub unsafe fn findNearestPoly(
                self: &dtNavMeshQuery,
                center: *const f32,
                half_extents: *const f32,
                filter: *const dtQueryFilter,
                nearest_ref: *mut u32,
                nearest_point: *mut f32,
            ) -> u32;

            #[rust_name = "find_path"]
            pub unsafe fn findPath(
                self: &dtNavMeshQuery,
                start_poly: u32,
                end_poly: u32,
                origin: *const f32,
                destination: *const f32,
                filter: *const dtQueryFilter,
                path: *mut u32,
                path_len: *mut i32,
                max_len: i32,
            ) -> u32;

            #[rust_name = "find_straight_path"]
            pub unsafe fn findStraightPath(
                self: &dtNavMeshQuery,
                start_pos: *const f32,
                end_pos: *const f32,
                path: *const u32,
                path_len: i32,
                straight_path: *mut f32,
                straight_path_flags: *mut u8,
                straight_path_polys: *mut u32,
                straight_path_len: *mut i32,
                max_len: i32,
                options: i32,
            ) -> u32;

            #[rust_name = "get_poly_height"]
            pub unsafe fn getPolyHeight(
                self: &dtNavMeshQuery,
                poly: u32,
                pos: *const f32,
                height: *mut f32,
            ) -> u32;

            #[rust_name = "init"]
            #[cfg(feature = "detour_crowd")]
            pub unsafe fn init(self: Pin<&mut dtPathCorridor>, max_len: i32) -> bool;

            #[rust_name = "len"]
            #[cfg(feature = "detour_crowd")]
            #[allow(clippy::len_without_is_empty)]
            pub fn getPathCount(self: &dtPathCorridor) -> i32;

            #[rust_name = "get_pos"]
            #[cfg(feature = "detour_crowd")]
            pub fn getPos(self: &dtPathCorridor) -> *const f32;

            #[rust_name = "reset"]
            #[cfg(feature = "detour_crowd")]
            pub unsafe fn reset(self: Pin<&mut dtPathCorridor>, poly: u32, pos: *const f32);

            #[rust_name = "set_corridor"]
            #[cfg(feature = "detour_crowd")]
            pub unsafe fn setCorridor(
                self: Pin<&mut dtPathCorridor>,
                target: *const f32,
                path: *const u32,
                path_len: i32,
            );

            #[rust_name = "find_corners"]
            #[cfg(feature = "detour_crowd")]
            pub unsafe fn findCorners(
                self: Pin<&mut dtPathCorridor>,
                corner_vertices: *mut f32,
                corner_flags: *mut u8,
                corner_polys: *mut u32,
                max_len: i32,
                query: *mut dtNavMeshQuery,
                filter: *const dtQueryFilter,
            ) -> i32;

            #[rust_name = "move_position"]
            #[cfg(feature = "detour_crowd")]
            pub unsafe fn movePosition(
                self: Pin<&mut dtPathCorridor>,
                new_pos: *const f32,
                query: *mut dtNavMeshQuery,
                filter: *const dtQueryFilter,
            ) -> bool;
        }
    }
}

unsafe impl Send for ffi::recast::rcPolyMesh {}
unsafe impl Sync for ffi::recast::rcPolyMesh {}
unsafe impl Send for ffi::recast::rcPolyMeshDetail {}
unsafe impl Sync for ffi::recast::rcPolyMeshDetail {}
unsafe impl Send for ffi::recast::rcPolyMeshDetailOwned {}
unsafe impl Sync for ffi::recast::rcPolyMeshDetailOwned {}
unsafe impl Send for ffi::detour::dtNavMesh {}
unsafe impl Send for ffi::detour::dtNavMeshQuery {}
unsafe impl Sync for ffi::detour::dtNavMeshQuery {}
unsafe impl Send for ffi::detour::dtPathCorridor {}
unsafe impl Sync for ffi::detour::dtPathCorridor {}

impl std::ops::Deref for ffi::recast::rcPolyMeshDetailOwned {
    type Target = ffi::recast::rcPolyMeshDetail;

    fn deref(&self) -> &Self::Target {
        self.poly_mesh_detail_owned_get_inner()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "recast")]
    mod recast {
        use crate::ffi::recast;

        #[test]
        fn test_calc_grid_size() {
            let bmin = [-10.; 3];
            let bmax = [10.; 3];
            let cs = 1.;
            let (mut w, mut h) = (0, 0);
            unsafe {
                recast::calc_grid_size(
                    bmin.as_ptr(),
                    bmax.as_ptr(),
                    cs,
                    &mut w as *mut _,
                    &mut h as *mut _,
                );
            }

            assert_eq!(w, 20);
            assert_eq!(h, 20);
        }

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
