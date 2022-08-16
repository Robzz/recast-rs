#include "recast-sys/src/lib.rs.h"
#include "recast-sys/recastnavigation/Recast/Include/Recast.h"
#include <memory>

std::unique_ptr<rcContext> newRcContext(bool diagnostics) {
    return std::make_unique<rcContext>(diagnostics);
}

std::unique_ptr<rcHeightfield> newRcHeightfield() {
    return std::make_unique<rcHeightfield>();
}

std::unique_ptr<rcCompactHeightfield> newRcCompactHeightfield() {
    return std::make_unique<rcCompactHeightfield>();
}

std::unique_ptr<rcContourSet> newRcContourSet() {
    return std::make_unique<rcContourSet>();
}

std::unique_ptr<rcPolyMesh> newRcPolyMesh() {
    return std::make_unique<rcPolyMesh>();
}

std::unique_ptr<rcPolyMeshDetail> newRcPolyMeshDetail() {
    return std::make_unique<rcPolyMeshDetail>();
}

const std::uint16_t* polyMeshGetVerts(rcPolyMesh const& poly_mesh) {
    return poly_mesh.verts;
}

const std::uint16_t* polyMeshGetPolys(rcPolyMesh const& poly_mesh) {
    return poly_mesh.polys;
}

const std::uint16_t* polyMeshGetRegions(rcPolyMesh const& poly_mesh) {
    return poly_mesh.regs;
}

const std::uint8_t* polyMeshGetAreas(rcPolyMesh const& poly_mesh) {
    return poly_mesh.areas;
}

std::int32_t polyMeshGetPolyCount(rcPolyMesh const& poly_mesh) {
    return poly_mesh.npolys;
}

std::int32_t polyMeshGetVertexCount(rcPolyMesh const& poly_mesh) {
    return poly_mesh.nverts;
}

std::int32_t polyMeshGetMaxVertexCountPerPoly(rcPolyMesh const& poly_mesh) {
    return poly_mesh.nverts;
}

std::int32_t polyMeshDetailGetNumMeshes(rcPolyMeshDetail const& detail) {
    return detail.nmeshes;
}

std::int32_t polyMeshDetailGetNumVerts(rcPolyMeshDetail const& detail) {
    return detail.nverts;
}

std::int32_t polyMeshDetailGetNumTris(rcPolyMeshDetail const& detail) {
    return detail.ntris;
}

const std::uint32_t* polyMeshDetailGetMeshes(rcPolyMeshDetail const& detail) {
    return detail.meshes;
}

const float* polyMeshDetailGetVerts(rcPolyMeshDetail const& detail) {
    return detail.verts;
}

const std::uint8_t* polyMeshDetailGetTris(rcPolyMeshDetail const& detail) {
    return detail.tris;
}
