#pragma once

#include "rust/cxx.h"
#include "recast-sys/recastnavigation/Recast/Include/Recast.h"
#include <memory>

struct rcPolyMeshDetailOwned {
    public:
    rcPolyMeshDetailOwned(rcPolyMeshDetail* detail);
    ~rcPolyMeshDetailOwned();

    rcPolyMeshDetail const& getInner() const;
    rcPolyMeshDetail& getInner();

    private:
    rcPolyMeshDetail* m_inner;
};

std::unique_ptr<rcContext> newRcContext(bool diagnostics);
std::unique_ptr<rcHeightfield> newRcHeightfield();
std::unique_ptr<rcCompactHeightfield> newRcCompactHeightfield();
std::unique_ptr<rcContourSet> newRcContourSet();
std::unique_ptr<rcPolyMesh> newRcPolyMesh();
std::unique_ptr<rcPolyMeshDetail> newRcPolyMeshDetail();
std::unique_ptr<rcPolyMeshDetailOwned> newRcPolyMeshDetailOwned();

const std::uint16_t* polyMeshGetVerts(rcPolyMesh const& poly_mesh);
std::uint16_t* polyMeshGetVertsMut(rcPolyMesh & poly_mesh);
const std::uint16_t* polyMeshGetPolys(rcPolyMesh const& poly_mesh);
std::uint16_t* polyMeshGetPolysMut(rcPolyMesh& poly_mesh);
const std::uint16_t* polyMeshGetRegions(rcPolyMesh const& poly_mesh);
std::uint16_t* polyMeshGetRegionsMut(rcPolyMesh& poly_mesh);
const std::uint16_t* polyMeshGetFlags(rcPolyMesh const& poly_mesh);
std::uint16_t* polyMeshGetFlagsMut(rcPolyMesh& poly_mesh);
const std::uint8_t* polyMeshGetAreas(rcPolyMesh const& poly_mesh);
std::uint8_t* polyMeshGetAreasMut(rcPolyMesh& poly_mesh);


std::int32_t polyMeshGetPolyCount(rcPolyMesh const& poly_mesh);
std::int32_t polyMeshGetVertexCount(rcPolyMesh const& poly_mesh);
std::int32_t polyMeshGetMaxVertexCountPerPoly(rcPolyMesh const& poly_mesh);

std::int32_t polyMeshDetailGetNumMeshes(rcPolyMeshDetail const& detail);
std::int32_t polyMeshDetailGetNumVerts(rcPolyMeshDetail const& detail);
std::int32_t polyMeshDetailGetNumTris(rcPolyMeshDetail const& detail);
const std::uint32_t* polyMeshDetailGetMeshes(rcPolyMeshDetail const& detail);
const float* polyMeshDetailGetVerts(rcPolyMeshDetail const& detail);
const std::uint8_t* polyMeshDetailGetTris(rcPolyMeshDetail const& detail);
