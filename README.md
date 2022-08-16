# Recast-rs - Rust bindings to the Recast/Detour libraries

`recast-rs` offers bindings to the [Recast](https://github.com/recastnavigation/recastnavigation) libraries. The main
goal of this crate is to be used as a foundation for a plugin integrating Recast into Bevy, but it may be suitable for
other non-Bevy applications.

## Status

`recast-rs` is currently in very early development. There are currently no crates.io releases. The first release is
likely to happen when this crate becomes usable enough to build a minimal Bevy plugin.

## Contents

`recast-rs` contains two crates:

  * `recast-sys`: low level FFI bindings to the Recast library.
  * `recast`: higher level bindings.
