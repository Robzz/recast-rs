/// Create a new type wrapping a `UniquePtr` to a recast/detour resource.
/// The type will expose a constructor ensuring the returned pointer is valid, a `pin_mut` method
/// mutably pinning the value, as well as implement the `AsRef<T>` trait where `T` is a
/// Recast/Detour type exposed by `recast-sys`.
///
/// An invocation of the macro looks like :
///
/// ```
/// # #[macro_use]
/// # extern crate recast_rs;
/// # use recast_rs::uptr_wrapper;
/// # fn main() {}
/// uptr_wrapper!(PolyMeshWrapper, recast_sys::ffi::recast::rcPolyMesh, recast_sys::ffi::recast::new_poly_mesh);
/// ```
#[macro_export]
macro_rules! uptr_wrapper {
    ($visi:vis $my_t:ident, $rc_t:ty, $rc_ctor:path) => {
        $visi struct $my_t {
            ptr: ::cxx::UniquePtr<$rc_t>
        }

        #[allow(dead_code)]
        impl $my_t {
            pub fn new() -> ::core::result::Result<$my_t, $crate::Error> {
                let ptr = $rc_ctor();
                Ok($my_t { ptr: $crate::check_uptr_alloc(ptr)? })
            }

            pub fn pin_mut(&mut self) -> ::core::pin::Pin<&mut $rc_t> {
                self.ptr.pin_mut()
            }
        }

        impl Default for $my_t {
            fn default() -> $my_t {
                $my_t::new().expect("Memory allocation failure")
            }
        }

        impl ::core::convert::AsRef<$rc_t> for $my_t {
            fn as_ref(&self) -> &$rc_t {
                self.ptr.as_ref().unwrap()
            }
        }
    }
}
