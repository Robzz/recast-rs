use cxx::UniquePtr;
use recast_sys::{ffi::recast::*, RecastConfig};

#[cfg(feature = "detour")]
use recast_sys::ffi::detour::*;

#[cfg(feature = "detour")]
use crate::detour::NavMesh;

use crate::{
    recast::{
        CompactHeightField, ContourSet, HeightField, MarkedMesh, Mesh, PolyMesh, PolyMeshDetail,
        RecastError, RecastNavMeshData,
    },
    Error,
};

pub struct RecastContext {
    ptr: UniquePtr<rcContext>,
    config: RecastConfig,
    grid_width: i32,
    grid_height: i32,
}

impl RecastContext {
    pub fn new(config: RecastConfig) -> Result<RecastContext, String> {
        let ptr = recast_sys::ffi::recast::new_context(true);
        if ptr.is_null() {
            return Err("Could not create recast context (out of memory ?)".to_owned());
        }

        let mut grid_width = 0;
        let mut grid_height = 0;
        unsafe {
            recast_sys::ffi::recast::calc_grid_size(
                config.bmin.as_ptr(),
                config.bmax.as_ptr(),
                config.cs,
                &mut grid_width as *mut _,
                &mut grid_height as *mut _,
            )
        };
        Ok(RecastContext {
            ptr,
            config,
            grid_width,
            grid_height,
        })
    }

    pub fn config(&self) -> &RecastConfig {
        &self.config
    }
    pub fn config_mut(&mut self) -> &mut RecastConfig {
        &mut self.config
    }

    pub fn mark_walkable_triangles<'ctx, 'data>(
        &'ctx mut self,
        mesh: &Mesh<'data>,
    ) -> MarkedMesh<'data> {
        let n_vertices = mesh.vertices.len() / 3;
        let indices = mesh.indices.map(|s| s.to_owned()).unwrap_or_else(|| {
            mesh.vertices
                .chunks_exact(3)
                .enumerate()
                .map(|(i, _v)| i as i32)
                .collect::<Vec<_>>()
        });
        let n_triangles = &indices.len() / 3;

        let mut walkable_areas = vec![0; n_triangles];
        unsafe {
            recast_sys::ffi::recast::mark_walkable_triangles(
                self.context_ptr(),
                self.config.walkable_slope_angle,
                mesh.vertices.as_ptr(),
                n_vertices as i32,
                indices.as_ptr(),
                n_triangles as i32,
                walkable_areas.as_mut_ptr(),
            )
        };
        MarkedMesh {
            vertices: mesh.vertices,
            indices,
            areas: walkable_areas,
        }
    }

    pub fn new_heightfield(&mut self, width: i32, height: i32) -> Result<HeightField, Error> {
        let mut heightfield = HeightField::new()?;
        let res = unsafe {
            recast_sys::ffi::recast::create_heightfield(
                self.context_ptr(),
                heightfield.pin_mut(),
                width as i32,
                height as i32,
                self.config.bmin.as_ptr(),
                self.config.bmax.as_ptr(),
                self.config.cs,
                self.config.ch,
            )
        };
        if res {
            Ok(heightfield)
        } else {
            // TODO: wrong error kind
            Err(RecastError::CompactHeightfieldError.into())
        }
    }

    /// Applies a sensible sequence of operations to generate a NavMesh from a triangle mesh. This
    /// method is a good default to start building navmeshes, but for more advanced or performance
    /// critical scenarios, you may want to use the other methods exposed by this class directly
    /// and cache intermediate results.
    pub fn default_pipeline<'a, I>(&mut self, input_geo: I) -> Result<RecastNavMeshData, Error>
    where
        I: IntoIterator<Item = &'a Mesh<'a>>,
    {
        let mut heightfield = self.new_heightfield(self.grid_width, self.grid_height)?;

        for mesh in input_geo {
            let marked_mesh = self.mark_walkable_triangles(mesh);
            self.rasterize_mesh(&mut heightfield, &marked_mesh);
        }

        // TODO: make configurable ?
        let filter_low_hanging_obstacles = true;
        let filter_ledge_spans = true;
        let filter_walkable_low_height_spans = true;
        if filter_low_hanging_obstacles {
            self.filter_low_hanging_walkable_obstacles(&mut heightfield);
        }
        if filter_ledge_spans {
            self.filter_ledge_spans(&mut heightfield);
        }
        if filter_walkable_low_height_spans {
            self.filter_walkable_low_height_spans(&mut heightfield);
        }

        let mut compact_heightfield = CompactHeightField::new().unwrap();
        self.build_compact_heightfield(&mut heightfield, &mut compact_heightfield);
        self.erode_walkable_area(&mut compact_heightfield);

        self.build_distance_field(&mut compact_heightfield);
        self.build_regions(&mut compact_heightfield);

        let mut contour_set = ContourSet::new().unwrap();
        self.build_contours(&mut compact_heightfield, &mut contour_set);

        let mut poly_mesh = PolyMesh::new().unwrap();
        self.build_poly_mesh(&mut contour_set, &mut poly_mesh);
        // TODO: this should not be arcane as it is. this is about the flags thingy for detour filter queries, btw
        for flag in poly_mesh.flags_mut() {
            *flag = 1;
        }

        let mut detail = PolyMeshDetail::new().unwrap();
        self.build_poly_mesh_detail(&poly_mesh, &compact_heightfield, &mut detail);

        Ok(RecastNavMeshData { poly_mesh, detail })
    }

    #[cfg(feature = "detour")]
    pub fn default_pipeline_detour<'a, I>(
        &mut self,
        input_geo: I,
    ) -> Result<(RecastNavMeshData, NavMesh), Error>
    where
        I: IntoIterator<Item = &'a Mesh<'a>>,
    {
        let navmesh_data = self.default_pipeline(input_geo)?;
        let mut create_mesh_data = NavMeshCreateParams::from(&navmesh_data);
        create_mesh_data.b_min = self.config.bmin;
        create_mesh_data.b_max = self.config.bmax;
        create_mesh_data.walkable_height = self.config.walkable_height as f32 * self.config.ch;
        create_mesh_data.walkable_radius = self.config.walkable_radius as f32 * self.config.cs;
        create_mesh_data.walkable_climb = self.config.walkable_climb as f32 * self.config.ch;
        create_mesh_data.cs = self.config.cs;
        create_mesh_data.ch = self.config.ch;
        create_mesh_data.build_bv_tree = true;

        let navmesh = NavMesh::single_tile(create_mesh_data)?;

        Ok((navmesh_data, navmesh))
    }

    pub fn rasterize_mesh(&mut self, heightfield: &mut HeightField, mesh: &MarkedMesh) -> bool {
        let n_vertices = mesh.vertices.len() / 3;
        let n_triangles = mesh.indices.len() / 3;

        println!(
            "Rasterizing mesh with {} vertices and {} triangles",
            n_vertices, n_triangles
        );

        return unsafe {
            recast_sys::ffi::recast::rasterize_triangles_with_indices(
                self.context_ptr(),
                mesh.vertices.as_ptr(),
                n_vertices as i32,
                mesh.indices.as_ptr(),
                mesh.areas.as_ptr(),
                n_triangles as i32,
                heightfield.pin_mut(),
                self.config.merge_region_area,
            )
        };
    }

    pub fn filter_low_hanging_walkable_obstacles(&mut self, heightfield: &mut HeightField) {
        return unsafe {
            recast_sys::ffi::recast::filter_low_hanging_walkable_obstacles(
                self.context_ptr(),
                self.config.walkable_climb,
                heightfield.pin_mut(),
            )
        };
    }

    pub fn filter_ledge_spans(&mut self, heightfield: &mut HeightField) {
        return unsafe {
            recast_sys::ffi::recast::filter_ledge_spans(
                self.context_ptr(),
                self.config.walkable_height,
                self.config.walkable_climb,
                heightfield.pin_mut(),
            )
        };
    }

    pub fn filter_walkable_low_height_spans(&mut self, heightfield: &mut HeightField) {
        return unsafe {
            recast_sys::ffi::recast::filter_walkable_low_height_spans(
                self.context_ptr(),
                self.config.walkable_height,
                heightfield.pin_mut(),
            )
        };
    }

    pub fn build_compact_heightfield(
        &mut self,
        heightfield: &mut HeightField,
        compact: &mut CompactHeightField,
    ) -> bool {
        return unsafe {
            recast_sys::ffi::recast::build_compact_heightfield(
                self.context_ptr(),
                self.config.walkable_height,
                self.config.walkable_climb,
                heightfield.pin_mut(),
                compact.pin_mut(),
            )
        };
    }

    pub fn erode_walkable_area(&mut self, heightfield: &mut CompactHeightField) -> bool {
        return unsafe {
            recast_sys::ffi::recast::erode_walkable_area(
                self.context_ptr(),
                self.config.walkable_radius,
                heightfield.pin_mut(),
            )
        };
    }

    pub fn build_distance_field(&mut self, heightfield: &mut CompactHeightField) -> bool {
        return unsafe {
            recast_sys::ffi::recast::build_distance_field(self.context_ptr(), heightfield.pin_mut())
        };
    }

    pub fn build_regions(&mut self, heightfield: &mut CompactHeightField) -> bool {
        return unsafe {
            recast_sys::ffi::recast::build_regions(
                self.context_ptr(),
                heightfield.pin_mut(),
                self.config.border_size,
                self.config.min_region_area,
                self.config.merge_region_area,
            )
        };
    }

    pub fn build_contours(
        &mut self,
        heightfield: &mut CompactHeightField,
        contours: &mut ContourSet,
    ) -> bool {
        return unsafe {
            recast_sys::ffi::recast::build_contours(
                self.context_ptr(),
                heightfield.pin_mut(),
                self.config.max_simplification_error,
                self.config.max_edge_len,
                contours.pin_mut(),
                0,
            )
        };
    }

    pub fn build_poly_mesh(&mut self, contours: &mut ContourSet, poly_mesh: &mut PolyMesh) -> bool {
        return unsafe {
            recast_sys::ffi::recast::build_poly_mesh(
                self.context_ptr(),
                contours.pin_mut(),
                self.config.max_verts_per_poly,
                poly_mesh.pin_mut(),
            )
        };
    }

    pub fn build_poly_mesh_detail(
        &mut self,
        poly_mesh: &PolyMesh,
        heightfield: &CompactHeightField,
        detail: &mut PolyMeshDetail,
    ) -> bool {
        return unsafe {
            recast_sys::ffi::recast::build_poly_mesh_detail(
                self.context_ptr(),
                poly_mesh.as_ref(),
                heightfield.as_ref(),
                self.config.details_sample_dist,
                self.config.details_sample_max_error,
                detail.pin_mut(),
            )
        };
    }

    unsafe fn context_ptr(&mut self) -> *mut rcContext {
        self.ptr.pin_mut().get_unchecked_mut()
    }
}

#[cfg(test)]
mod tests {
    use recast_sys::RecastConfig;

    use crate::recast::Mesh;

    use super::RecastContext;

    // TODO: test non indexed mesh
    const SAMPLE_TRI_MESH: &[[f32; 3]] = &[
        [-10., 0., 10.],
        [ 10., 0., 10.],
        [ 10., 0.,-10.],
        [-10., 0.,-10.],
    ];
    const SAMPLE_TRI_MESH_INDICES: &[i32] = &[
        0, 1, 2,
        2, 3, 0
    ];
    const SAMPLE_TRI_MESH_BMIN: [f32; 3] = [-15., -1., -15.];
    const SAMPLE_TRI_MESH_BMAX: [f32; 3] = [15., 1., 15.];

    #[test]
    fn test_new_context() {
        let context = RecastContext::new(RecastConfig::default());
        assert!(context.is_ok());
    }

    #[test]
    fn test_context_config_passthrough() {
        let context = RecastContext::new(RecastConfig::default()).unwrap();
        assert_eq!(context.config, RecastConfig::default());
    }

    #[test]
    fn test_new_heightfield() {
        let mut context = RecastContext::new(RecastConfig::default()).unwrap();
        assert!(context.new_heightfield(20, 20).is_ok());
    }

    #[test]
    fn mesh_from_sample_succeeds() {
        let buf = SAMPLE_TRI_MESH
            .into_iter()
            .flatten()
            .map(|f| *f)
            .collect::<Vec<_>>();
        let res = Mesh::from_buffers(buf.as_slice(), SAMPLE_TRI_MESH_INDICES);
        assert!(res.is_ok());
    }

    #[test]
    fn default_pipeline_sample_mesh_succeeds() {
        let mut context = RecastContext::new(RecastConfig {
            bmin: SAMPLE_TRI_MESH_BMIN,
            bmax: SAMPLE_TRI_MESH_BMAX,
            ..Default::default()
        })
        .unwrap();
        let buf = SAMPLE_TRI_MESH
            .into_iter()
            .flatten()
            .map(|f| *f)
            .collect::<Vec<_>>();
        let mesh = Mesh::from_vertex_buffer(buf.as_slice()).unwrap();
        let res = context.default_pipeline(&[mesh]);
        assert!(res.is_ok());
        dbg!(res.unwrap().poly_mesh.vertices());
    }

    #[test]
    #[cfg(feature = "detour")]
    fn default_detour_pipeline_sample_mesh_succeeds() {
        let mut context = RecastContext::new(RecastConfig {
            bmin: SAMPLE_TRI_MESH_BMIN,
            bmax: SAMPLE_TRI_MESH_BMAX,
            max_verts_per_poly: 6,
            ..Default::default()
        })
        .unwrap();
        let buf = SAMPLE_TRI_MESH
            .into_iter()
            .flatten()
            .map(|f| *f)
            .collect::<Vec<_>>();
        let mesh = Mesh::from_vertex_buffer(buf.as_slice()).unwrap();
        let res = context.default_pipeline_detour(&[mesh]);
        assert!(res.is_ok());
    }
}
