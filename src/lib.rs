pub mod compact_heightfield;
pub mod context;
pub mod contour_set;
pub mod heightfield;
pub mod mesh;
pub mod navmesh_data;
pub mod poly_mesh;
pub mod poly_mesh_detail;

pub use compact_heightfield::*;
pub use context::*;
pub use contour_set::*;
use cxx::{UniquePtr, private::UniquePtrTarget};
pub use heightfield::*;
pub use mesh::*;
pub use navmesh_data::*;
pub use poly_mesh::*;
pub use poly_mesh_detail::*;

pub use recast_sys::RecastConfig;

pub struct RecastCompactHeightfield;
pub struct RecastContourSet;
pub struct RecastPolyMeshDetail;

use thiserror::Error;

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

pub(crate) fn check_uptr_alloc<T>(ptr: UniquePtr<T>) -> Result<UniquePtr<T>, RecastError>
    where T: UniquePtrTarget
{
    if ptr.is_null() {
        return Err(RecastError::OutOfMemoryError);
    }
    Ok(ptr)
}
