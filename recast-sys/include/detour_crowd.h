#pragma once

#include "rust/cxx.h"
#include <memory>

#include "recast-sys/recastnavigation/DetourCrowd/Include/DetourPathCorridor.h"

struct NavMeshCreateParams;

std::unique_ptr<dtPathCorridor> newDtPathCorridor();