#pragma once

#include "rust/cxx.h"
#include <memory>

#include "recast-sys/recastnavigation/Detour/Include/DetourNavMesh.h"
#include "recast-sys/recastnavigation/Detour/Include/DetourNavMeshBuilder.h"
#include "recast-sys/recastnavigation/Detour/Include/DetourNavMeshQuery.h"
#include "recast-sys/recastnavigation/Detour/Include/DetourStatus.h"

struct NavMeshCreateParams;

std::unique_ptr<dtNavMesh> newDtNavMesh();
std::unique_ptr<dtNavMeshQuery> newDtNavMeshQuery();
std::unique_ptr<dtQueryFilter> newDtQueryFilter();

bool createNavMeshData(NavMeshCreateParams* params, std::uint8_t **outData, std::int32_t *outDataSize);
