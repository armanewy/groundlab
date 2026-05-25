use serde::{Deserialize, Serialize};

use crate::{CellCoord, CoverClass};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentObject {
    pub id: String,
    pub label: String,
    pub kind: EnvironmentObjectKind,
    pub cell: CellCoord,
    pub footprint: (u32, u32),
    pub blocks_sight: bool,
    pub cover: CoverClass,
    pub movement_cost_delta: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EnvironmentObjectKind {
    Tree(TreeState),
    Log(LogState),
    Rock(RockState),
    Wall(WallState),
    Wire(ObstacleState),
    Stakes(ObstacleState),
    FightingPosition(PositionState),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TreeState {
    Standing,
    PartiallyCut { progress: u8 },
    Falling { direction: Direction },
    FallenTrunk { direction: Direction },
    CutLogs,
    StakesBundle,
    Stump,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogState {
    Loose {
        direction: Direction,
    },
    DragPrepared {
        direction: Direction,
    },
    Positioned {
        direction: Direction,
    },
    Braced {
        direction: Direction,
    },
    PreparedRoll {
        direction: Direction,
        release_cell: CellCoord,
        predicted_path: Vec<CellCoord>,
    },
    Released {
        direction: Direction,
    },
    Rolling {
        direction: Direction,
    },
    Spent {
        direction: Direction,
    },
    Piled,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RockState {
    Stable,
    Cracked,
    Rubble,
    RollingStone { direction: Direction },
    BlockedRubblePile,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WallState {
    Intact,
    Damaged,
    Breached,
    CollapsedRubble,
    ClearedRubble,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ObstacleState {
    Placed,
    Damaged,
    Cleared,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PositionState {
    DugIn,
    Reinforced,
    Collapsed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::North => (0, -1),
            Direction::East => (1, 0),
            Direction::South => (0, 1),
            Direction::West => (-1, 0),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Direction::North => "north",
            Direction::East => "east",
            Direction::South => "south",
            Direction::West => "west",
        }
    }
}
