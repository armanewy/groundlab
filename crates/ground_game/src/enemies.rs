use serde::{Deserialize, Serialize};

use crate::CellCoord;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnemyGroupSpec {
    pub label: String,
    pub count: u32,
    pub doctrine: EnemyDoctrine,
    pub spawn: CellCoord,
    pub objective: CellCoord,
    pub movement_profile: MovementProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EnemyDoctrine {
    RushShortest,
    PreferCover,
    FlankViaConcealment,
    AvoidObstacles,
    PushThroughLightObstacles,
    ClearObstacles,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MovementProfile {
    pub base_speed: f32,
    pub obstacle_tolerance: f32,
    pub cover_preference: f32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DoctrineWeights {
    pub(crate) trench_cost: f32,
    pub(crate) berm_cost: f32,
    pub(crate) obstacle_cost: f32,
    pub(crate) cover_discount: f32,
    pub(crate) concealment_discount: f32,
    pub(crate) road_bias: f32,
    pub(crate) height_cost: f32,
}

impl EnemyDoctrine {
    pub(crate) fn weights(self) -> DoctrineWeights {
        match self {
            EnemyDoctrine::RushShortest => DoctrineWeights {
                trench_cost: 1.3,
                berm_cost: 1.1,
                obstacle_cost: 1.4,
                cover_discount: 0.0,
                concealment_discount: 0.0,
                road_bias: -0.18,
                height_cost: 0.28,
            },
            EnemyDoctrine::PreferCover => DoctrineWeights {
                trench_cost: 0.9,
                berm_cost: 0.6,
                obstacle_cost: 1.1,
                cover_discount: 0.32,
                concealment_discount: 0.18,
                road_bias: 0.02,
                height_cost: 0.24,
            },
            EnemyDoctrine::FlankViaConcealment => DoctrineWeights {
                trench_cost: 0.8,
                berm_cost: 0.7,
                obstacle_cost: 1.0,
                cover_discount: 0.18,
                concealment_discount: 0.42,
                road_bias: 0.24,
                height_cost: 0.20,
            },
            EnemyDoctrine::AvoidObstacles => DoctrineWeights {
                trench_cost: 2.2,
                berm_cost: 2.0,
                obstacle_cost: 2.7,
                cover_discount: 0.0,
                concealment_discount: 0.0,
                road_bias: -0.05,
                height_cost: 0.42,
            },
            EnemyDoctrine::PushThroughLightObstacles => DoctrineWeights {
                trench_cost: 1.5,
                berm_cost: 1.2,
                obstacle_cost: 0.75,
                cover_discount: 0.0,
                concealment_discount: 0.0,
                road_bias: -0.12,
                height_cost: 0.35,
            },
            EnemyDoctrine::ClearObstacles => DoctrineWeights {
                trench_cost: 1.4,
                berm_cost: 1.1,
                obstacle_cost: 0.45,
                cover_discount: 0.04,
                concealment_discount: 0.0,
                road_bias: -0.10,
                height_cost: 0.32,
            },
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EnemyDoctrine::RushShortest => "rush shortest",
            EnemyDoctrine::PreferCover => "prefer cover",
            EnemyDoctrine::FlankViaConcealment => "flank via concealment",
            EnemyDoctrine::AvoidObstacles => "avoid obstacles",
            EnemyDoctrine::PushThroughLightObstacles => "push through light obstacles",
            EnemyDoctrine::ClearObstacles => "clear obstacles",
        }
    }
}
