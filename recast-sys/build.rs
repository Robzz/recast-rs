fn main() {
    let mut bridge = cxx_build::bridge("src/lib.rs");

    #[cfg(feature = "detour")]
    {
    }
    #[cfg(feature = "recast")]
    {
        bridge
            .include("recastnavigation/Recast/Include/")
            .flag_if_supported("-Wno-comment")
            .file("recastnavigation/Recast/Source/Recast.cpp")
            .file("recastnavigation/Recast/Source/RecastAlloc.cpp")
            .file("recastnavigation/Recast/Source/RecastArea.cpp")
            .file("recastnavigation/Recast/Source/RecastContour.cpp")
            .file("recastnavigation/Recast/Source/RecastFilter.cpp")
            .file("recastnavigation/Recast/Source/RecastLayers.cpp")
            .file("recastnavigation/Recast/Source/RecastMesh.cpp")
            .file("recastnavigation/Recast/Source/RecastMeshDetail.cpp")
            .file("recastnavigation/Recast/Source/RecastRasterization.cpp")
            .file("recastnavigation/Recast/Source/RecastRegion.cpp");
    }

    //#[cfg(not(feature = "bundled"))]
    //{
        //println!("cargo:rustc-link-lib=Recast");
    //}

    bridge.file("src/recast.cpp")
        .flag_if_supported("-std=c++14")
        //.flag_if_supported("Wl,--no-allow-shlib-undefined")
        .shared_flag(true)
        .compile("recast-sys");
}
