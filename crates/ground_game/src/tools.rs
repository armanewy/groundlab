use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolLoadout {
    pub tools: Vec<ToolKind>,
}

impl ToolLoadout {
    pub fn basic_field_kit() -> Self {
        Self {
            tools: vec![
                ToolKind::Shovel,
                ToolKind::Axe,
                ToolKind::Hammer,
                ToolKind::Rope,
            ],
        }
    }

    pub fn has(&self, tool: ToolKind) -> bool {
        self.tools.contains(&tool)
    }

    pub fn has_all(&self, tools: &[ToolKind]) -> bool {
        tools.iter().all(|tool| self.has(*tool))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolKind {
    Shovel,
    Axe,
    Hammer,
    Rope,
    SawKit,
    Mattock,
    Winch,
    BraceKit,
}

impl ToolKind {
    pub fn label(self) -> &'static str {
        match self {
            ToolKind::Shovel => "shovel",
            ToolKind::Axe => "axe",
            ToolKind::Hammer => "hammer",
            ToolKind::Rope => "rope",
            ToolKind::SawKit => "saw kit",
            ToolKind::Mattock => "mattock",
            ToolKind::Winch => "winch",
            ToolKind::BraceKit => "brace kit",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrewPool {
    pub crews: u32,
    pub labor_seconds_available: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionConstraints {
    pub max_work_orders: u32,
    pub allow_assault_preview: bool,
}
