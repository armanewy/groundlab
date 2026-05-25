use serde::{Deserialize, Serialize};

use crate::WorkOrderKind;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalMaterialStock {
    pub earth_spoil: i32,
    pub timber: i32,
    pub logs: i32,
    pub stakes: i32,
    pub loose_stone: i32,
    pub scrap: i32,
    pub rope_uses: i32,
}

impl LocalMaterialStock {
    pub fn get(&self, kind: LocalMaterialKind) -> i32 {
        match kind {
            LocalMaterialKind::EarthSpoil => self.earth_spoil,
            LocalMaterialKind::Timber => self.timber,
            LocalMaterialKind::Logs => self.logs,
            LocalMaterialKind::Stakes => self.stakes,
            LocalMaterialKind::LooseStone => self.loose_stone,
            LocalMaterialKind::Scrap => self.scrap,
            LocalMaterialKind::RopeUses => self.rope_uses,
        }
    }

    pub fn add(&mut self, kind: LocalMaterialKind, amount: i32) {
        match kind {
            LocalMaterialKind::EarthSpoil => self.earth_spoil += amount,
            LocalMaterialKind::Timber => self.timber += amount,
            LocalMaterialKind::Logs => self.logs += amount,
            LocalMaterialKind::Stakes => self.stakes += amount,
            LocalMaterialKind::LooseStone => self.loose_stone += amount,
            LocalMaterialKind::Scrap => self.scrap += amount,
            LocalMaterialKind::RopeUses => self.rope_uses += amount,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.earth_spoil == 0
            && self.timber == 0
            && self.logs == 0
            && self.stakes == 0
            && self.loose_stone == 0
            && self.scrap == 0
            && self.rope_uses == 0
    }

    pub fn net(outputs: &Self, inputs: &Self) -> Self {
        Self {
            earth_spoil: outputs.earth_spoil - inputs.earth_spoil,
            timber: outputs.timber - inputs.timber,
            logs: outputs.logs - inputs.logs,
            stakes: outputs.stakes - inputs.stakes,
            loose_stone: outputs.loose_stone - inputs.loose_stone,
            scrap: outputs.scrap - inputs.scrap,
            rope_uses: outputs.rope_uses - inputs.rope_uses,
        }
    }

    pub fn signed_summary(&self) -> Vec<String> {
        [
            (self.earth_spoil, "spoil"),
            (self.timber, "timber"),
            (self.logs, "logs"),
            (self.stakes, "stakes"),
            (self.loose_stone, "stone"),
            (self.scrap, "scrap"),
            (self.rope_uses, "rope"),
        ]
        .into_iter()
        .filter(|(value, _)| *value != 0)
        .map(|(value, label)| {
            if value > 0 {
                format!("{label} +{value}")
            } else {
                format!("{label} {value}")
            }
        })
        .collect()
    }

    pub fn positive_summary(&self) -> Vec<String> {
        [
            (LocalMaterialKind::EarthSpoil, "spoil"),
            (LocalMaterialKind::Timber, "timber"),
            (LocalMaterialKind::Logs, "logs"),
            (LocalMaterialKind::Stakes, "stakes"),
            (LocalMaterialKind::LooseStone, "stone"),
            (LocalMaterialKind::Scrap, "scrap"),
            (LocalMaterialKind::RopeUses, "rope"),
        ]
        .into_iter()
        .filter_map(|(kind, label)| {
            let value = self.get(kind);
            (value > 0).then(|| format!("{label}: {value}"))
        })
        .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocalMaterialKind {
    EarthSpoil,
    Timber,
    Logs,
    Stakes,
    LooseStone,
    Scrap,
    RopeUses,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MaterialLedgerEntry {
    pub order_id: u32,
    pub order_kind: WorkOrderKind,
    pub inputs: LocalMaterialStock,
    pub outputs: LocalMaterialStock,
    pub net: LocalMaterialStock,
    pub note: String,
}
