fn main() {
    let mut bridge = cxx_build::bridge("src/lib.rs");
    bridge.flag_if_supported("-std=c++14")
        .shared_flag(true)
        // Yeah, I know. The official cmake build system doesn't enable additional warnings either.
        // Enabling those flags does produce warnings, and I'd rather not patch Recast myself.
        .warnings(false);

    #[cfg(feature = "detour")]
    {
        bridge
            .include("recastnavigation/Detour/Include")
            // Detour source files
            .file("recastnavigation/Detour/Source/DetourAlloc.cpp")
            .file("recastnavigation/Detour/Source/DetourAssert.cpp")
            .file("recastnavigation/Detour/Source/DetourCommon.cpp")
            .file("recastnavigation/Detour/Source/DetourNavMesh.cpp")
            .file("recastnavigation/Detour/Source/DetourNavMeshBuilder.cpp")
            .file("recastnavigation/Detour/Source/DetourNavMeshQuery.cpp")
            .file("recastnavigation/Detour/Source/DetourNode.cpp")
            // Our additional functions
            .file("src/detour.cpp");

        #[cfg(feature = "detour_crowd")]
        {
            bridge
                .include("recastnavigation/DetourCrowd/Include")
                // Detour source files
                .file("recastnavigation/DetourCrowd/Source/DetourCrowd.cpp")
                .file("recastnavigation/DetourCrowd/Source/DetourLocalBoundary.cpp")
                .file("recastnavigation/DetourCrowd/Source/DetourObstacleAvoidance.cpp")
                .file("recastnavigation/DetourCrowd/Source/DetourPathCorridor.cpp")
                .file("recastnavigation/DetourCrowd/Source/DetourPathQueue.cpp")
                .file("recastnavigation/DetourCrowd/Source/DetourProximityGrid.cpp")
                // Our additional functions
                //.file("src/detour_crowd.cpp")
                ;
        }
    }
    #[cfg(feature = "recast")]
    {
        bridge
            .include("recastnavigation/Recast/Include/")
            // Recast source files
            .file("recastnavigation/Recast/Source/Recast.cpp")
            .file("recastnavigation/Recast/Source/RecastAlloc.cpp")
            .file("recastnavigation/Recast/Source/RecastArea.cpp")
            .file("recastnavigation/Recast/Source/RecastAssert.cpp")
            .file("recastnavigation/Recast/Source/RecastContour.cpp")
            .file("recastnavigation/Recast/Source/RecastFilter.cpp")
            .file("recastnavigation/Recast/Source/RecastLayers.cpp")
            .file("recastnavigation/Recast/Source/RecastMesh.cpp")
            .file("recastnavigation/Recast/Source/RecastMeshDetail.cpp")
            .file("recastnavigation/Recast/Source/RecastRasterization.cpp")
            .file("recastnavigation/Recast/Source/RecastRegion.cpp")
            // Our additional functions
            .file("src/recast.cpp");
    }

    bridge.compile("recast-sys");
}
