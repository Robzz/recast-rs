use cxx::UniquePtr;
use recast_sys::{ffi::recast::*, RecastConfig};

use crate::{
    CompactHeightField, ContourSet, HeightField, MarkedMesh, Mesh, PolyMesh, PolyMeshDetail,
    RecastError, RecastNavMeshData,
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
        let n_triangles = mesh.indices.unwrap().len() / 3;

        let mut walkable_areas = vec![0; n_triangles];
        unsafe {
            recast_sys::ffi::recast::mark_walkable_triangles(
                self.context_ptr(),
                self.config.walkable_slope_angle,
                mesh.vertices.as_ptr(),
                n_vertices as i32,
                mesh.indices.unwrap().as_ptr(),
                n_triangles as i32,
                walkable_areas.as_mut_ptr(),
            )
        };
        MarkedMesh {
            vertices: mesh.vertices,
            indices: mesh.indices.unwrap(),
            areas: walkable_areas,
        }
    }

    pub fn init_heightfield(
        &mut self,
        heightfield: &mut HeightField,
        width: i32,
        height: i32,
    ) {
        unsafe {
            recast_sys::ffi::recast::create_heightfield(
                self.context_ptr(),
                heightfield.pin_mut(),
                width as i32,
                height as i32,
                self.config.bmin.as_ptr(),
                self.config.bmax.as_ptr(),
                self.config.cs,
                self.config.ch,
            );
        }
    }

    /// Applies a sensible sequence of operations to generate a NavMesh from a triangle mesh. This
    /// method is a good default to start building navmeshes, but for more advanced or performance
    /// critical scenarios, you may want to use the other methods exposed by this class directly
    /// and cache intermediate results.
    pub fn default_pipeline<'a, I>(
        &mut self,
        input_geo: I,
    ) -> Result<RecastNavMeshData, RecastError>
    where
        I: IntoIterator<Item = &'a Mesh<'a>>,
    {
        let mut heightfield = HeightField::new().unwrap();
        self.init_heightfield(&mut heightfield, self.grid_width, self.grid_height);

        for mesh in input_geo {
            if let Some(_indices) = mesh.indices {
                let marked_mesh = self.mark_walkable_triangles(mesh);
                self.rasterize_mesh(&mut heightfield, &marked_mesh);
            } else {
                todo!("TODO: process non indexed mesh");
            }
        }

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

        let mut detail = PolyMeshDetail::new().unwrap();
        self.build_poly_mesh_detail(&poly_mesh, &compact_heightfield, &mut detail);

        Ok(RecastNavMeshData { poly_mesh, detail })
    }

    pub fn rasterize_mesh(&mut self, heightfield: &mut HeightField, mesh: &MarkedMesh) -> bool {
        let n_vertices = mesh.vertices.len() / 3;
        let n_triangles = mesh.indices.len() / 3;

        println!("Rasterizing mesh with {} vertices and {} triangles", n_vertices, n_triangles);

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
