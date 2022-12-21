use std::sync::{Arc, Mutex, Weak};

use crate::uptr_wrapper;

use recast_sys::ffi::detour::*;

#[derive(Debug, Default, thiserror::Error)]
#[error("A Detour error occured")]
pub struct Error {
    code: Option<u32>
}

impl Error {
    pub(crate) fn from_code(code: u32) -> Error {
        Error { code: Some(code) }
    }
}

uptr_wrapper!(pub(crate) OwnedNavMesh, dtNavMesh, new_navmesh);
uptr_wrapper!(QueryFilter, dtQueryFilter, new_query_filter);
uptr_wrapper!(pub(crate) NavMeshQueryPriv, dtNavMeshQuery, new_navmesh_query);
uptr_wrapper!(PathCorridor, dtPathCorridor, new_path_corridor);

fn detour_check_status<T>(ok_data: T, status: u32) -> Result<T, Error> {
    if status_failed(status) {
        return Err(Error::from_code(status));
    }
    Ok(ok_data)
}

impl NavMeshQueryPriv {
    pub fn init(&mut self, navmesh: &OwnedNavMesh, max_nodes: u32) -> Result<(), Error> {
        let res = unsafe {
            self.pin_mut()
                .init(navmesh.as_ref() as *const _, max_nodes as i32)
        };
        detour_check_status((), res)
    }
}

impl PathCorridor {
    pub fn init(&mut self, max_len: u32) -> Result<(), Error> {
        let res = unsafe { self.pin_mut().init(max_len as i32) };
        if !res {
            return Err(Error::default());
        }
        Ok(())
    }
}

pub struct NavMesh {
    ptr: Arc<Mutex<OwnedNavMesh>>,
}

impl NavMesh {
    /// Create a new single-tile `NavMesh`.
    pub fn single_tile(mut data: NavMeshCreateParams) -> Result<NavMesh, crate::Error> {
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
            return Err(Error::default())?;
        }

        let mut navmesh = OwnedNavMesh::new()?;
        let res = unsafe {
            navmesh
                .pin_mut()
                // Let Detour handle freeing the tile data when the navmesh is free'd
                .init(data_ptr, data_len, dtTileFlags::FreeData.repr)
        };
        if status_failed(res) {
            return Err(Error::from_code(res))?;
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
    pub fn find_path(
        &mut self,
        origin: [f32; 3],
        destination: [f32; 3],
        max_len: u32,
    ) -> Option<NavMeshPath> {
        // TODO: remove
        const SEARCH_EXTENTS: [f32; 3] = [5., 5., 5.];

        if let Some((start_poly, start_point)) = self.find_nearest_polygon(origin, SEARCH_EXTENTS) {
            if let Some((end_poly, end_point)) =
                self.find_nearest_polygon(destination, SEARCH_EXTENTS)
            {
                let mut path_vec = Vec::with_capacity(max_len as usize);
                let mut path_len = 0;
                let filter = QueryFilter::default();
                unsafe {
                    self.query.as_ref().find_path(
                        start_poly,
                        end_poly,
                        start_point.as_ptr(),
                        end_point.as_ptr(),
                        filter.as_ref() as *const _,
                        path_vec.as_mut_ptr(),
                        &mut path_len as *mut i32,
                        max_len as i32,
                    );

                    if path_len != 0 {
                        path_vec.set_len(path_len as usize);
                        path_vec.shrink_to_fit();
                    } else {
                        return None;
                    }
                }

                let mut dt_path = PathCorridor::new().unwrap();
                if dt_path.init(max_len).is_err() {
                    return None;
                }

                unsafe {
                    dt_path.pin_mut().reset(start_poly, start_point.as_ptr());
                }

                let mut path = NavMeshPath {
                    wptr: Arc::downgrade(&self.ptr),
                    path: dt_path,
                };

                unsafe {
                    path.path.pin_mut().set_corridor(
                        destination.as_ptr(),
                        path_vec.as_ptr(),
                        path_len,
                    );
                }

                return Some(path);
            }
        }
        None
    }

    pub fn closest_point_on_poly(&self, poly_ref: u32, pos: &[f32; 3]) -> Result<([f32; 3], bool), Error> {
        let mut pos_over_poly = false;
        let mut closest = [0.; 3];
        let _res = unsafe {
            self.query.as_ref().closest_point_on_poly(poly_ref, pos.as_ptr(), closest.as_mut_ptr(), &mut pos_over_poly as *mut bool)
        };
        // TODO: error checking
        Ok((closest, pos_over_poly))
    }

    /// Search for the nearest polygon in a box around `p`. Returns the detour polygon id of the found polygon if any, 0 if none.
    pub fn find_nearest_polygon(&self, p: [f32; 3], half_extents: [f32; 3]) -> Option<(u32, [f32; 3])> {
        let mut nearest_ref = 0;
        let mut nearest = [0.; 3];
        let filter = QueryFilter::default();
        unsafe {
            self.query.as_ref().find_nearest_poly(
                p.as_ptr(),
                half_extents.as_ptr(),
                filter.as_ref() as *const _,
                &mut nearest_ref as *mut _,
                nearest.as_mut_ptr(),
            );
        }

        if nearest_ref != 0 {
            Some((nearest_ref, nearest))
        } else {
            None
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

/// Interface to the path navigation functions. Internally holds an `Arc` making sure the navmesh remains allocated as long as this object is valid.
pub struct NavMeshPathGuard<'q> {
    _ptr: Arc<Mutex<OwnedNavMesh>>,
    path: &'q mut PathCorridor,
}

impl<'q> NavMeshPathGuard<'q> {
    pub fn reset(&mut self, poly: u32, position: [f32; 3]) {
        unsafe {
            self.path.pin_mut().reset(poly, position.as_ptr());
        }
    }

    pub fn find_corners(&mut self, max_corners: i32, query: &mut NavMeshQuery) -> Option<Vec<f32>> {
        if let Some(query) = query.upgrade() {
            let mut corners = Vec::with_capacity(3 * max_corners as usize);
            let mut flags = Vec::with_capacity(max_corners as usize);
            let mut polys = Vec::with_capacity(max_corners as usize);
            let corners_len;
            let filter = QueryFilter::new().unwrap();
            unsafe {
                corners_len = self.path.pin_mut().find_corners(
                    corners.as_mut_ptr(),
                    flags.as_mut_ptr(),
                    polys.as_mut_ptr(),
                    max_corners,
                    (query.query.pin_mut().get_unchecked_mut()) as *mut dtNavMeshQuery,
                    filter.as_ref() as *const _,
                );

                if corners_len != 0 {
                    corners.set_len(3 * corners_len as usize);
                    flags.set_len(corners_len as usize);
                    polys.set_len(corners_len as usize);
                    corners.shrink_to_fit();
                    flags.shrink_to_fit();
                    polys.shrink_to_fit();
                } else {
                    return None;
                }
            }

            Some(corners)
        } else {
            None
        }
    }

    pub fn move_position(&mut self, new_position: [f32; 3], query: &mut NavMeshQuery) -> bool {
        if let Some(query) = query.upgrade() {
            let filter = QueryFilter::new().unwrap();
            unsafe {
                self.path.pin_mut().move_position(
                    new_position.as_ptr(),
                    query.query.pin_mut().get_unchecked_mut() as *mut _,
                    filter.as_ref() as *const _,
                )
            }
        } else {
            false
        }
    }
}

/// Assists navigation along a computed path on a `NavMesh`.
#[derive(Default)]
pub struct NavMeshPath {
    wptr: Weak<Mutex<OwnedNavMesh>>,
    path: PathCorridor,
}

impl NavMeshPath {
    /// Construct a new empty `NavMeshPath`, not bound to any `NavMesh`.
    ///
    /// This method is useful for default construction of paths, but these paths will be useless as they don't contain any actual path.
    /// To computer a path on a specific navmesh, see the `NavMeshQueryGuard::find_path` method.
    pub fn new() -> NavMeshPath {
        Self::default()
    }

    /// Upgrade this path object into a path guard that exposes the navigation methods.
    ///
    /// Return `None` if the navmesh was deallocated.
    pub fn upgrade(&mut self) -> Option<NavMeshPathGuard> {
        if let Some(arc) = self.wptr.upgrade() {
            Some(NavMeshPathGuard {
                _ptr: arc,
                path: &mut self.path,
            })
        } else {
            None
        }
    }
}
