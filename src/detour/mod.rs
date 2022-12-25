use std::sync::{Arc, Mutex, Weak};

use crate::uptr_wrapper;

use recast_sys::ffi::detour::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DetourStatus {
    code: u32,
}

impl From<u32> for DetourStatus {
    fn from(code: u32) -> Self {
        DetourStatus { code }
    }
}

#[cfg(feature = "detour")]
impl DetourStatus {
    const DT_FAILURE: u32 = 1 << 31;
    const DT_SUCCESS: u32 = 1 << 30;
    const DT_IN_PROGRESS: u32 = 1 << 29;
    const DT_STATUS_DETAIL_MASK: u32 = 0x00FFFFFF;
    const DT_WRONG_MAGIC: u32 = 1 << 0;
    const DT_WRONG_VERSION: u32 = 1 << 1;
    const DT_OUT_OF_MEMORY: u32 = 1 << 2;
    const DT_INVALID_PARAM: u32 = 1 << 3;
    const DT_BUFFER_TOO_SMALL: u32 = 1 << 4;
    const DT_OUT_OF_NODES: u32 = 1 << 5;
    const DT_PARTIAL_RESULT: u32 = 1 << 6;
    const DT_ALREADY_OCCUPIED: u32 = 1 << 7;

    pub fn is_failure(&self) -> bool {
        (self.code & Self::DT_FAILURE) != 0
    }

    pub fn is_success(&self) -> bool {
        (self.code & Self::DT_SUCCESS) != 0
    }

    pub fn is_in_progress(&self) -> bool {
        (self.code & Self::DT_IN_PROGRESS) != 0
    }

    pub fn has_detail(&self) -> bool {
        (self.code & Self::DT_STATUS_DETAIL_MASK) != 0
    }

    pub fn is_wrong_magic(&self) -> bool {
        (self.code & Self::DT_WRONG_MAGIC) != 0
    }

    pub fn is_wrong_version(&self) -> bool {
        (self.code & Self::DT_WRONG_VERSION) != 0
    }

    pub fn is_oom(&self) -> bool {
        (self.code & Self::DT_OUT_OF_MEMORY) != 0
    }

    pub fn is_invalid_param(&self) -> bool {
        (self.code & Self::DT_INVALID_PARAM) != 0
    }

    pub fn is_buffer_too_small(&self) -> bool {
        (self.code & Self::DT_BUFFER_TOO_SMALL) != 0
    }

    pub fn is_out_of_nodes(&self) -> bool {
        (self.code & Self::DT_OUT_OF_NODES) != 0
    }

    pub fn is_partial_result(&self) -> bool {
        (self.code & Self::DT_PARTIAL_RESULT) != 0
    }

    pub fn is_already_occupied(&self) -> bool {
        (self.code & Self::DT_ALREADY_OCCUPIED) != 0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error code returned by a Detour method
    #[error("A Detour operation failed")]
    Detour(Option<DetourStatus>),
    /// Error conditions not directly tied to a Detour error code
    #[error("{0}")]
    Other(#[from] OtherError),
}

impl From<DetourStatus> for Error {
    fn from(status: DetourStatus) -> Self {
        Error::Detour(Some(status))
    }
}

impl Default for Error {
    fn default() -> Self {
        Error::Other(OtherError::default())
    }
}

#[derive(Debug, Default, thiserror::Error)]
pub enum OtherError {
    #[error("No path was found")]
    NoPathFound,
    #[error("No polygon was found in the specified search extents")]
    NoPolyFound,
    #[error("Memory allocation failure")]
    MemAllocFailed,
    #[error("Navmesh creation failure")]
    NavMeshCreationFailed,
    #[error("A resource was invalidated")]
    DeallocatedResource,
    #[error("An unspecified error occurred")]
    #[default]
    Other,
}

uptr_wrapper!(pub(crate) OwnedNavMesh, dtNavMesh, new_navmesh);
uptr_wrapper!(QueryFilter, dtQueryFilter, new_query_filter);
uptr_wrapper!(pub(crate) NavMeshQueryPriv, dtNavMeshQuery, new_navmesh_query);
uptr_wrapper!(PathCorridorPriv, dtPathCorridor, new_path_corridor);

impl NavMeshQueryPriv {
    pub fn init(&mut self, navmesh: &OwnedNavMesh, max_nodes: u32) -> Result<(), Error> {
        let res: DetourStatus = unsafe {
            self.pin_mut()
                .init(navmesh.as_ref() as *const _, max_nodes as i32)
                .into()
        };
        if res.is_failure() {
            return Err(res.into());
        }
        Ok(())
    }
}

impl PathCorridorPriv {
    pub fn init(&mut self, max_len: u32) -> Result<(), Error> {
        let res = unsafe { self.pin_mut().init(max_len as i32) };
        if !res {
            return Err(Error::Other(OtherError::MemAllocFailed));
        }
        Ok(())
    }
}

pub struct NavMesh {
    ptr: Arc<Mutex<OwnedNavMesh>>,
}

impl NavMesh {
    /// Create a new single-tile `NavMesh`.
    pub fn single_tile(mut data: NavMeshCreateParams) -> crate::Result<NavMesh> {
        let mut data_ptr: *mut u8 = std::ptr::null_mut();
        let mut data_len: i32 = 0;
        let res = unsafe {
            create_navmesh_data(
                &mut data as *mut NavMeshCreateParams,
                &mut data_ptr as *mut *mut u8,
                &mut data_len as *mut i32,
            )
        };
        if !res {
            return Err(Error::Other(OtherError::NavMeshCreationFailed))?;
        }

        let mut navmesh = OwnedNavMesh::new()?;
        let res: DetourStatus = unsafe {
            navmesh
                .pin_mut()
                // Let Detour handle freeing the tile data when the navmesh is free'd
                .init(data_ptr, data_len, dtTileFlags::FreeData.repr)
                .into()
        };
        if res.is_failure() {
            return Err(Error::from(res))?;
        }

        Ok(NavMesh {
            ptr: Arc::new(Mutex::new(navmesh)),
        })
    }

    pub fn new_query(&self, max_nodes: u32) -> Result<NavMeshQuery, crate::Error> {
        let mut query = NavMeshQueryPriv::new()?;
        let lock = self.ptr.lock();
        query.init(lock.as_ref().unwrap(), max_nodes)?;
        Ok(NavMeshQuery {
            wptr: Arc::downgrade(&self.ptr),
            dt_query: query,
        })
    }
}

/// Interface to the navmesh query functions. Internally holds an `Arc` making sure the navmesh remains allocated as long as this object is valid.
pub struct NavMeshQueryGuard<'q> {
    ptr: Arc<Mutex<OwnedNavMesh>>,
    query: &'q mut NavMeshQueryPriv,
}

impl<'q> NavMeshQueryGuard<'q> {
    /// Find a path going from the origin point to the destination, going through a maximum of
    /// `max_len` navmesh polygons.
    ///
    /// The origin and destination points must reside on the surface of the navmesh. You can find
    /// the closest navmesh points by using the [`find_nearest_polygon`](Self::find_nearest_polygon) method. See the
    /// [`find_path_search_polys`](Self::find_path_search_polys) method for an alternative that performs this search
    /// for you.
    ///
    /// If a path is found, returns a non-empty [`Vec`] of polygon identifiers.
    pub fn find_path(
        &mut self,
        orig: [f32; 3],
        orig_poly: u32,
        dest: [f32; 3],
        dest_poly: u32,
        max_len: u32,
    ) -> crate::Result<Vec<u32>> {
        let mut path_vec = Vec::with_capacity(max_len as usize);
        let mut path_len = 0;
        let filter = QueryFilter::default();
        let status: DetourStatus = unsafe {
            let status = self.query.as_ref().find_path(
                orig_poly,
                dest_poly,
                orig.as_ptr(),
                dest.as_ptr(),
                filter.as_ref() as *const _,
                path_vec.as_mut_ptr(),
                &mut path_len as *mut i32,
                max_len as i32,
            );
            path_vec.set_len(path_len as usize);
            status.into()
        };
        if status.is_failure() {
            return Err(Error::from(status))?;
        }
        path_vec.shrink_to_fit();
        Ok(path_vec)
    }

    /// Find the closest navmesh points to the specified origin and destination within the specified search extents,
    /// and compute a path between them, going through a maximum of `max_len` navmesh polygons.
    ///
    /// If a path is found, returns a non-empty [`Vec`] of polygon identifiers.
    pub fn find_path_search_polys(
        &mut self,
        orig: [f32; 3],
        dest: [f32; 3],
        half_extents: [f32; 3],
        max_len: u32,
    ) -> crate::Result<Vec<u32>> {
        let (orig_poly, orig) = self.find_nearest_polygon(orig, half_extents)?;
        let (dest_poly, dest) = self.find_nearest_polygon(dest, half_extents)?;
        self.find_path(orig, orig_poly, dest, dest_poly, max_len)
    }

    /// Return a path straightened by 'string pulling'
    pub fn find_straight_path(
        &self,
        start: [f32; 3],
        dest: [f32; 3],
        path: &[u32],
        max_len: u32,
        crossings: StraightPathCrossings,
    ) -> crate::Result<StraightPath> {
        let mut straight_path = Vec::with_capacity(max_len as usize * 3);
        let mut path_flags = Vec::with_capacity(max_len as usize);
        let mut path_polys = Vec::with_capacity(max_len as usize);
        let mut path_len = 0;
        let opts: dtStraightPathOptions = crossings.into();
        let status: DetourStatus = unsafe {
            let status = self.query.as_ref().find_straight_path(
                start.as_ptr(),
                dest.as_ptr(),
                path.as_ptr(),
                path.len() as i32,
                straight_path.as_mut_ptr(),
                path_flags.as_mut_ptr(),
                path_polys.as_mut_ptr(),
                &mut path_len as *mut i32,
                max_len as i32,
                opts.repr,
            );
            straight_path.set_len(path_len as usize * 3);
            path_flags.set_len(path_len as usize);
            path_polys.set_len(path_len as usize);
            status.into()
        };

        if status.is_failure() {
            return Err(Error::from(status))?;
        }

        let straight_path = straight_path
            .chunks_exact(3)
            .map(|c| c.try_into().unwrap())
            .collect();
        path_flags.shrink_to_fit();
        path_polys.shrink_to_fit();
        Ok(StraightPath {
            points: straight_path,
            flags: path_flags,
            polys: path_polys,
        })
    }

    pub fn get_poly_height(&self, poly: u32, pos: [f32; 3]) -> crate::Result<f32> {
        let mut height = 0.;
        let status: DetourStatus = unsafe {
            self.query
                .as_ref()
                .get_poly_height(
                    poly,
                    pos.as_ptr(),
                    &mut height as *mut f32,
                )
                .into()
        };
        if status.is_failure() {
            return Err(Error::from(status))?
        }
        Ok(height)
    }

    /// Construct a new path corridor from a path returned by one of the `find_path` methods.
    pub fn new_corridor(
        &self,
        start: [f32; 3],
        end: [f32; 3],
        path: &[u32],
    ) -> crate::Result<PathCorridor> {
        if path.is_empty() {
            // TODO: might be better to throw an error here ?
            return Ok(PathCorridor::new());
        }
        let mut dt_path = PathCorridorPriv::new()?;
        let path_len = path.len() as u32;
        dt_path.init(path_len + 1)?;

        unsafe {
            dt_path
                .pin_mut()
                .reset(*path.first().unwrap(), start.as_ptr());
        }

        let mut corridor = PathCorridor {
            wptr: Arc::downgrade(&self.ptr),
            path: dt_path,
        };

        unsafe {
            corridor
                .path
                .pin_mut()
                .set_corridor(end.as_ptr(), path.as_ptr(), path_len as i32);
        };

        Ok(corridor)
    }

    pub fn closest_point_on_poly(
        &self,
        poly_ref: u32,
        pos: [f32; 3],
    ) -> crate::Result<([f32; 3], bool)> {
        let mut pos_over_poly = false;
        let mut closest = [0.; 3];
        let status: DetourStatus = unsafe {
            self.query
                .as_ref()
                .closest_point_on_poly(
                    poly_ref,
                    pos.as_ptr(),
                    closest.as_mut_ptr(),
                    &mut pos_over_poly as *mut bool,
                )
                .into()
        };
        if status.is_failure() {
            return Err(Error::from(status))?;
        }
        Ok((closest, pos_over_poly))
    }

    /// Search for the nearest polygon in a box around `p`. Returns the detour polygon id of the found polygon if any, 0 if none.
    pub fn find_nearest_polygon(
        &self,
        p: [f32; 3],
        half_extents: [f32; 3],
    ) -> crate::Result<(u32, [f32; 3])> {
        let mut nearest_ref = 0;
        let mut nearest = [0.; 3];
        let filter = QueryFilter::default();
        let status: DetourStatus = unsafe {
            self.query
                .as_ref()
                .find_nearest_poly(
                    p.as_ptr(),
                    half_extents.as_ptr(),
                    filter.as_ref() as *const _,
                    &mut nearest_ref as *mut _,
                    nearest.as_mut_ptr(),
                )
                .into()
        };
        if status.is_failure() {
            return Err(Error::from(status))?;
        }

        if nearest_ref != 0 {
            Ok((nearest_ref, nearest))
        } else {
            Err(Error::from(OtherError::NoPolyFound))?
        }
    }
}

/// Provides the navigation mesh query functionality. Internally stores a weak reference to the
/// target navmesh. The actual querying API is provided by the `NavMeshQueryGuard` type,
/// constructed by upgrading the internal weak pointer.
#[derive(Default)]
pub struct NavMeshQuery {
    wptr: Weak<Mutex<OwnedNavMesh>>,
    pub(crate) dt_query: NavMeshQueryPriv,
}

impl NavMeshQuery {
    /// Construct a new empty `NavMeshQuery`, not bound to any `NavMesh`.
    ///
    /// This method is useful for default construction of queries, but these queries will be useless as they can't be used to query a navmesh.
    /// To construct a query object targeting a specific navmesh, see the `NavMesh::new_query` method.
    pub fn new() -> NavMeshQuery {
        Self::default()
    }

    /// Upgrade this query object into a query guard that exposes the query methods.
    ///
    /// Return `None` if the navmesh was deallocated.
    pub fn upgrade(&mut self) -> Option<NavMeshQueryGuard> {
        if let Some(arc) = self.wptr.upgrade() {
            Some(NavMeshQueryGuard {
                ptr: arc,
                query: &mut self.dt_query,
            })
        } else {
            None
        }
    }
}

/// The possible ways that the [`NavMeshQueryGuard::find_straight_path`] method can tesselate paths at crossings.
#[derive(Debug)]
pub enum StraightPathCrossings {
    /// Do not add extra points on the path at crossings
    None,
    /// Add extra points at area crossings
    Area,
    /// Add extra points at polygon crossings
    Poly,
}

impl From<StraightPathCrossings> for dtStraightPathOptions {
    fn from(value: StraightPathCrossings) -> Self {
        match value {
            StraightPathCrossings::None => dtStraightPathOptions { repr: 0 },
            StraightPathCrossings::Area => dtStraightPathOptions::AreaCrossings,
            StraightPathCrossings::Poly => dtStraightPathOptions::AllCrossings,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StraightPath {
    pub points: Vec<[f32; 3]>,
    pub flags: Vec<u8>,
    pub polys: Vec<u32>,
}

#[derive(Debug)]
pub struct Corners {
    vertices: Vec<f32>,
    flags: Vec<u8>,
    polys: Vec<u32>,
}

impl Corners {
    pub fn into_parts(self) -> (Vec<f32>, Vec<u8>, Vec<u32>) {
        (self.vertices, self.flags, self.polys)
    }

    pub fn vertices(&self) -> impl ExactSizeIterator<Item = [f32; 3]> + '_ {
        self.vertices.chunks_exact(3).map(|s| s.try_into().unwrap())
    }

    pub fn flags(&self) -> impl ExactSizeIterator<Item = u8> + '_ {
        self.flags.iter().cloned()
    }

    pub fn polys(&self) -> impl ExactSizeIterator<Item = u32> + '_ {
        self.polys.iter().cloned()
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = ([f32; 3], u8, u32)> + '_ {
        self.vertices()
            .zip(self.flags())
            .zip(self.polys())
            .map(|((v, f), p)| (v, f, p))
    }
}

/// Interface to the path corridor functions. Upgraded version of the weak pointer based
/// [`PathCorridor`] which guarantees the navmesh is valid as long as the object exists.
pub struct PathCorridorGuard<'q> {
    _ptr: Arc<Mutex<OwnedNavMesh>>,
    path: &'q mut PathCorridorPriv,
}

impl<'q> PathCorridorGuard<'q> {
    pub fn reset(&mut self, poly: u32, position: [f32; 3]) {
        unsafe {
            self.path.pin_mut().reset(poly, position.as_ptr());
        }
    }

    pub fn find_corners(
        &mut self,
        max_corners: i32,
        query: &mut NavMeshQueryGuard,
    ) -> Option<Corners> {
        let mut vertices = Vec::with_capacity(3 * max_corners as usize);
        let mut flags = Vec::with_capacity(max_corners as usize);
        let mut polys = Vec::with_capacity(max_corners as usize);
        let filter = QueryFilter::new().unwrap();
        unsafe {
            let corners_len = self.path.pin_mut().find_corners(
                vertices.as_mut_ptr(),
                flags.as_mut_ptr(),
                polys.as_mut_ptr(),
                max_corners,
                (query.query.pin_mut().get_unchecked_mut()) as *mut dtNavMeshQuery,
                filter.as_ref() as *const _,
            );

            if corners_len != 0 {
                vertices.set_len(3 * corners_len as usize);
                flags.set_len(corners_len as usize);
                polys.set_len(corners_len as usize);
                vertices.shrink_to_fit();
                flags.shrink_to_fit();
                polys.shrink_to_fit();
                return Some(Corners {
                    vertices,
                    flags,
                    polys,
                });
            }
        };
        None
    }

    /// Attempt to move the corridor to the specified position. If the move was successful, returns
    /// the new position, constrained to the surface of the navmesh.
    pub fn move_position(
        &mut self,
        new_position: [f32; 3],
        query: &mut NavMeshQueryGuard,
    ) -> Option<[f32; 3]> {
        let filter = QueryFilter::new().unwrap();
        let res = unsafe {
            self.path.pin_mut().move_position(
                new_position.as_ptr(),
                query.query.pin_mut().get_unchecked_mut() as *mut _,
                filter.as_ref() as *const _,
            )
        };
        if res {
            Some(*self.position())
        } else {
            None
        }
    }

    /// Return the current position of the path corridor.
    pub fn position(&self) -> &[f32; 3] {
        unsafe {
            std::slice::from_raw_parts(self.path.ptr.get_pos(), 3)
                .try_into()
                .unwrap()
        }
    }
}

/// Assists navigation along a computed path on a [`NavMesh`].
///
/// Internally holds a weak pointer to the navmesh it is bound to, which means active path
/// corridors will be invalidated along with their navmesh.
#[derive(Default)]
pub struct PathCorridor {
    wptr: Weak<Mutex<OwnedNavMesh>>,
    path: PathCorridorPriv,
}

impl PathCorridor {
    /// Construct a new empty path corridor, not bound to any [`NavMesh`].
    ///
    /// This method is useful for default construction of paths corridors, but but such objects will
    /// not be able to query a navmesh until they have been bound to one.
    pub fn new() -> PathCorridor {
        Self::default()
    }

    pub fn len(&self) -> u32 {
        self.path.ptr.len() as u32
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Upgrade this path corridor into a [`PathCorridorGuard`] that exposes the navigation methods.
    ///
    /// Return `None` if the navmesh was deallocated or no navmesh is bound.
    pub fn upgrade(&mut self) -> Option<PathCorridorGuard> {
        if let Some(arc) = self.wptr.upgrade() {
            Some(PathCorridorGuard {
                _ptr: arc,
                path: &mut self.path,
            })
        } else {
            None
        }
    }
}
