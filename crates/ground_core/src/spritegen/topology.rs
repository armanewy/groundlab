#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopologyEdge {
    North,
    East,
    South,
    West,
}

impl TopologyEdge {
    pub const ALL: [Self; 4] = [Self::North, Self::East, Self::South, Self::West];

    pub fn bit(self) -> u8 {
        match self {
            Self::North => 1,
            Self::East => 2,
            Self::South => 4,
            Self::West => 8,
        }
    }

    pub fn opposite(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::North => "north",
            Self::East => "east",
            Self::South => "south",
            Self::West => "west",
        }
    }

    pub fn from_bit(bit: u8) -> Option<Self> {
        match bit {
            1 => Some(Self::North),
            2 => Some(Self::East),
            4 => Some(Self::South),
            8 => Some(Self::West),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopologyMaskKind {
    Isolated,
    DeadEnd,
    Straight,
    Corner,
    TJunction,
    Cross,
}

impl TopologyMaskKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Isolated => "isolated",
            Self::DeadEnd => "dead_end",
            Self::Straight => "straight",
            Self::Corner => "corner",
            Self::TJunction => "t_junction",
            Self::Cross => "cross",
        }
    }
}

#[derive(Clone, Debug)]
pub struct TopologyMask {
    pub mask: u8,
    pub degree: u32,
    pub kind: TopologyMaskKind,
    pub connected_edges: Vec<TopologyEdge>,
    pub exposed_edges: Vec<TopologyEdge>,
}

impl TopologyMask {
    pub fn has_edge(&self, edge: TopologyEdge) -> bool {
        self.mask & edge.bit() != 0
    }
}

pub fn topology_for_mask(mask: u8) -> TopologyMask {
    let mask = mask & 0x0f;
    let degree = mask.count_ones();
    let connected_edges = TopologyEdge::ALL
        .into_iter()
        .filter(|edge| mask & edge.bit() != 0)
        .collect::<Vec<_>>();
    let exposed_edges = TopologyEdge::ALL
        .into_iter()
        .filter(|edge| mask & edge.bit() == 0)
        .collect::<Vec<_>>();
    let kind = match degree {
        0 => TopologyMaskKind::Isolated,
        1 => TopologyMaskKind::DeadEnd,
        2 if (mask == 0b0101) || (mask == 0b1010) => TopologyMaskKind::Straight,
        2 => TopologyMaskKind::Corner,
        3 => TopologyMaskKind::TJunction,
        _ => TopologyMaskKind::Cross,
    };
    TopologyMask {
        mask,
        degree,
        kind,
        connected_edges,
        exposed_edges,
    }
}

pub fn compatible_neighbor_masks(edge: TopologyEdge) -> impl Iterator<Item = u8> {
    let required = edge.opposite().bit();
    (0_u8..16).filter(move |mask| mask & required != 0)
}

pub fn topology_opening_span(length: u32, fraction: f32) -> (u32, u32) {
    let length = length.max(1);
    let center = length / 2;
    let opening = (length as f32 * fraction).round().max(1.0) as u32;
    let start = center.saturating_sub(opening / 2);
    let end = (center + opening / 2).min(length - 1);
    (start, end)
}
