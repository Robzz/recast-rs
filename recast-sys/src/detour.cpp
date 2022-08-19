#include "recast-sys/include/detour.h"

#include "recast-sys/src/lib.rs.h"

std::unique_ptr<dtNavMesh> newDtNavMesh() {
    return std::make_unique<dtNavMesh>();
}

std::unique_ptr<dtNavMeshQuery> newDtNavMeshQuery() {
    return std::make_unique<dtNavMeshQuery>();
}

std::unique_ptr<dtQueryFilter> newDtQueryFilter() {
    return std::make_unique<dtQueryFilter>();
}

std::unique_ptr<dtPathCorridor> newDtPathCorridor() {
    return std::make_unique<dtPathCorridor>();
}

bool createNavMeshData(NavMeshCreateParams* params, std::uint8_t **outData, std::int32_t *outDataSize) {
    auto dtParams = dtNavMeshCreateParams();
    dtParams.verts = params->vertices;
    dtParams.vertCount = params->num_vertices;
    dtParams.polys = params->polygons;
    dtParams.polyFlags = params->polygon_flags;
    dtParams.polyAreas = params->polygon_areas;
    dtParams.polyCount = params->num_polys;
    dtParams.nvp = params->max_vertices_per_poly;
    dtParams.detailMeshes = params->detail_meshes;
    dtParams.detailVerts = params->detail_vertices;
    dtParams.detailVertsCount = params->num_detail_vertices;
    dtParams.detailTris = params->detail_triangles;
    dtParams.detailTriCount = params->num_detail_triangles;
    dtParams.offMeshConVerts = params->off_mesh_conn_vertices;
    dtParams.offMeshConRad = params->off_mesh_conn_radii;
    dtParams.offMeshConFlags = params->off_mesh_conn_flags;
    dtParams.offMeshConAreas = params->off_mesh_conn_areas;
    dtParams.offMeshConDir = params->off_mesh_conn_dir;
    dtParams.offMeshConUserID = params->off_mesh_conn_ids;
    dtParams.offMeshConCount = params->off_mesh_conn_count;
    dtParams.userId = params->user_id;
    dtParams.tileX = params->tile_x;
    dtParams.tileY = params->tile_y;
    dtParams.tileLayer = params->tile_layer;
    std::copy(params->b_min.begin(), params->b_min.end(), dtParams.bmin);
    std::copy(params->b_max.begin(), params->b_max.end(), dtParams.bmax);
    dtParams.walkableHeight = params->walkable_height;
    dtParams.walkableRadius = params->walkable_radius;
    dtParams.walkableClimb = params->walkable_climb;
    dtParams.cs = params->cs;
    dtParams.ch = params->ch;
    dtParams.buildBvTree = params->build_bv_tree;

    return dtCreateNavMeshData(&dtParams, outData, outDataSize);
}
