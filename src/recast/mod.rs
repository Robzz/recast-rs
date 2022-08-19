use crate::uptr_wrapper;

use recast_sys::ffi::recast::*;
use thiserror::Error;

mod context;
mod mesh;
mod navmesh_data;
mod poly_mesh;
mod poly_mesh_detail;

pub use context::*;
pub use mesh::*;
pub use navmesh_data::*;
pub use poly_mesh::*;
pub use poly_mesh_detail::*;

pub use recast_sys::RecastConfig;

#[derive(Debug, Error)]
pub enum RecastError {
    #[error("Failed to allocate memory")]
    OutOfMemoryError,
    #[error("An error occured during mesh to heightfield rasterization")]
    RasterizeMeshError,
    #[error("An error occured during compact heightfield construction")]
    CompactHeightfieldError,
    #[error("An error occured during walkables areas erosion")]
    ErodeWalkableAreasError,
    #[error("An error occured during distance field construction")]
    DistanceFieldError,
    #[error("An error occured during regions construction")]
    RegionsError,
    #[error("An error occured during region contours construction")]
    ContoursError,
    #[error("An error occured during polygon mesh construction")]
    PolyMesh,
    #[error("An error occured during detailed polygon mesh construction")]
    PolyMeshDetailsError,
}

uptr_wrapper!(pub HeightField, rcHeightfield, new_heightfield);
uptr_wrapper!(
    pub CompactHeightField,
    rcCompactHeightfield,
    new_compact_heightfield
);
uptr_wrapper!(pub ContourSet, rcContourSet, new_contour_set);
uptr_wrapper!(pub PolyMesh, rcPolyMesh, new_poly_mesh);
uptr_wrapper!(pub PolyMeshDetail, rcPolyMeshDetail, new_poly_mesh_detail);
