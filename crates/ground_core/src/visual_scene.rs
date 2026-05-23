use serde::{Deserialize, Serialize};

use crate::feature::{feature_visual_material, TerrainFeatureKind, TerrainFeatureMap};
use crate::recipe::GroundMaterial;
use crate::terrain::TerrainMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisualTerrainFormKind {
    FloorRegion,
    RaisedPlatform,
    RoadPatch,
    MudBasin,
    RockOutcrop,
    CliffFace,
    TrenchRun,
    BermRun,
    ShadowPatch,
    Dressing,
}

impl VisualTerrainFormKind {
    pub fn label(self) -> &'static str {
        match self {
            VisualTerrainFormKind::FloorRegion => "floor_region",
            VisualTerrainFormKind::RaisedPlatform => "raised_platform",
            VisualTerrainFormKind::RoadPatch => "road_patch",
            VisualTerrainFormKind::MudBasin => "mud_basin",
            VisualTerrainFormKind::RockOutcrop => "rock_outcrop",
            VisualTerrainFormKind::CliffFace => "cliff_face",
            VisualTerrainFormKind::TrenchRun => "trench_run",
            VisualTerrainFormKind::BermRun => "berm_run",
            VisualTerrainFormKind::ShadowPatch => "shadow_patch",
            VisualTerrainFormKind::Dressing => "dressing",
        }
    }

    pub fn is_floor_like(self) -> bool {
        matches!(
            self,
            VisualTerrainFormKind::FloorRegion
                | VisualTerrainFormKind::RaisedPlatform
                | VisualTerrainFormKind::RoadPatch
                | VisualTerrainFormKind::MudBasin
                | VisualTerrainFormKind::RockOutcrop
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl VisualRect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width: width.max(1),
            height: height.max(1),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualTerrainForm {
    pub id: String,
    pub kind: VisualTerrainFormKind,
    pub material: GroundMaterial,
    pub rect: VisualRect,
    pub base_height: f32,
    pub height_delta: f32,
    pub priority: i32,
    pub note: String,
}

impl VisualTerrainForm {
    fn same_merge_key(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.material == other.material
            && (self.base_height - other.base_height).abs() < 0.05
            && (self.height_delta - other.height_delta).abs() < 0.05
            && self.rect.x == other.rect.x
            && self.rect.width == other.rect.width
            && self.rect.y + self.rect.height == other.rect.y
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualScene {
    pub width: u32,
    pub height: u32,
    pub forms: Vec<VisualTerrainForm>,
}

impl VisualScene {
    pub fn from_terrain(map: &TerrainMap) -> Self {
        let features = TerrainFeatureMap::from_terrain(map);
        let mut forms = Vec::new();
        collect_floor_forms(map, &features, &mut forms);
        collect_structural_forms(map, &features, &mut forms);
        collect_dressing_forms(map, &mut forms);
        for (idx, form) in forms.iter_mut().enumerate() {
            if form.id.is_empty() {
                form.id = format!("{}_{idx:03}", form.kind.label());
            }
        }
        forms.sort_by_key(|form| (form.priority, form.rect.y, form.rect.x));
        Self {
            width: map.width,
            height: map.height,
            forms,
        }
    }

    pub fn form_count_by_kind(&self, kind: VisualTerrainFormKind) -> usize {
        self.forms.iter().filter(|form| form.kind == kind).count()
    }

    pub fn summary_line(&self) -> String {
        format!(
            "visual forms: {} floor, {} cliff, {} trench, {} berm, {} dressing",
            self.forms
                .iter()
                .filter(|form| form.kind.is_floor_like())
                .count(),
            self.form_count_by_kind(VisualTerrainFormKind::CliffFace),
            self.form_count_by_kind(VisualTerrainFormKind::TrenchRun),
            self.form_count_by_kind(VisualTerrainFormKind::BermRun),
            self.form_count_by_kind(VisualTerrainFormKind::Dressing),
        )
    }
}

fn collect_floor_forms(
    map: &TerrainMap,
    features: &TerrainFeatureMap,
    forms: &mut Vec<VisualTerrainForm>,
) {
    for y in 0..map.height {
        let mut x = 0;
        while x < map.width {
            let Some(cell) = map.cell(x, y) else {
                x += 1;
                continue;
            };
            let Some(feature) = features.cell(x, y) else {
                x += 1;
                continue;
            };
            let material = feature_visual_material(cell.ground);
            let kind = floor_form_kind(feature.kind, material, cell.effective_height());
            let base_height = cell.effective_height().round();
            let start_x = x;
            x += 1;
            while x < map.width {
                let Some(next_cell) = map.cell(x, y) else {
                    break;
                };
                let Some(next_feature) = features.cell(x, y) else {
                    break;
                };
                let next_material = feature_visual_material(next_cell.ground);
                let next_kind = floor_form_kind(
                    next_feature.kind,
                    next_material,
                    next_cell.effective_height(),
                );
                let next_height = next_cell.effective_height().round();
                if next_material != material
                    || next_kind != kind
                    || (next_height - base_height).abs() > 0.01
                {
                    break;
                }
                x += 1;
            }
            push_or_extend(
                forms,
                VisualTerrainForm {
                    id: String::new(),
                    kind,
                    material,
                    rect: VisualRect::new(start_x, y, x - start_x, 1),
                    base_height,
                    height_delta: 0.0,
                    priority: 10,
                    note: "merged top-surface region".to_string(),
                },
            );
        }
    }
}

fn floor_form_kind(
    feature_kind: TerrainFeatureKind,
    material: GroundMaterial,
    effective_height: f32,
) -> VisualTerrainFormKind {
    match feature_kind {
        TerrainFeatureKind::Trench | TerrainFeatureKind::Ditch => VisualTerrainFormKind::TrenchRun,
        TerrainFeatureKind::Berm => VisualTerrainFormKind::BermRun,
        TerrainFeatureKind::Ledge if effective_height >= 4.0 => {
            VisualTerrainFormKind::RaisedPlatform
        }
        _ => match material {
            GroundMaterial::Dirt => VisualTerrainFormKind::RoadPatch,
            GroundMaterial::Mud => VisualTerrainFormKind::MudBasin,
            GroundMaterial::Rock => VisualTerrainFormKind::RockOutcrop,
            GroundMaterial::BermTop => VisualTerrainFormKind::BermRun,
            GroundMaterial::TrenchFloor => VisualTerrainFormKind::TrenchRun,
            _ if effective_height >= 4.0 => VisualTerrainFormKind::RaisedPlatform,
            _ => VisualTerrainFormKind::FloorRegion,
        },
    }
}

fn collect_structural_forms(
    map: &TerrainMap,
    _features: &TerrainFeatureMap,
    forms: &mut Vec<VisualTerrainForm>,
) {
    for y in 0..map.height {
        let mut x = 0;
        while x < map.width {
            let Some(cell) = map.cell(x, y) else {
                x += 1;
                continue;
            };
            let current = cell.effective_height();
            let neighbor = if y + 1 < map.height {
                map.cell(x, y + 1)
                    .map(|n| n.effective_height())
                    .unwrap_or(0.0)
            } else {
                0.0
            };
            let delta = current - neighbor;
            if delta <= 0.15 {
                x += 1;
                continue;
            }
            let material = face_material_for_visual(cell.ground);
            let start_x = x;
            let height_delta = (delta * 4.0).round() / 4.0;
            x += 1;
            while x < map.width {
                let Some(next_cell) = map.cell(x, y) else {
                    break;
                };
                let next_current = next_cell.effective_height();
                let next_neighbor = if y + 1 < map.height {
                    map.cell(x, y + 1)
                        .map(|n| n.effective_height())
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                let next_delta = (next_current - next_neighbor).max(0.0);
                if next_delta <= 0.15
                    || face_material_for_visual(next_cell.ground) != material
                    || (((next_delta * 4.0).round() / 4.0) - height_delta).abs() > 0.05
                    || (next_current.round() - current.round()).abs() > 0.01
                {
                    break;
                }
                x += 1;
            }
            forms.push(VisualTerrainForm {
                id: String::new(),
                kind: VisualTerrainFormKind::CliffFace,
                material,
                rect: VisualRect::new(start_x, y, x - start_x, 1),
                base_height: current.round(),
                height_delta,
                priority: 30,
                note: "continuous visible front face".to_string(),
            });
        }
    }
}

fn collect_dressing_forms(map: &TerrainMap, forms: &mut Vec<VisualTerrainForm>) {
    let spawn = map.spawn;
    let objective = map.objective;
    forms.push(VisualTerrainForm {
        id: "spawn_marker_pad".to_string(),
        kind: VisualTerrainFormKind::Dressing,
        material: GroundMaterial::Dirt,
        rect: VisualRect::new(spawn.0.saturating_sub(1), spawn.1.saturating_sub(1), 3, 2),
        base_height: map
            .cell(spawn.0, spawn.1)
            .map(|cell| cell.effective_height())
            .unwrap_or(0.0),
        height_delta: 0.0,
        priority: 70,
        note: "small prepared entry pad / scene dressing".to_string(),
    });
    forms.push(VisualTerrainForm {
        id: "objective_engineering_pad".to_string(),
        kind: VisualTerrainFormKind::Dressing,
        material: GroundMaterial::BermTop,
        rect: VisualRect::new(
            objective.0.saturating_sub(2),
            objective.1.saturating_sub(1),
            4,
            3,
        ),
        base_height: map
            .cell(objective.0, objective.1)
            .map(|cell| cell.effective_height())
            .unwrap_or(0.0),
        height_delta: 0.0,
        priority: 75,
        note: "field-engineered defended objective pad".to_string(),
    });
}

fn push_or_extend(forms: &mut Vec<VisualTerrainForm>, form: VisualTerrainForm) {
    if let Some(existing) = forms
        .iter_mut()
        .rev()
        .find(|existing| existing.same_merge_key(&form))
    {
        existing.rect.height += form.rect.height;
    } else {
        forms.push(form);
    }
}

fn face_material_for_visual(material: GroundMaterial) -> GroundMaterial {
    match material {
        GroundMaterial::Grass | GroundMaterial::Dirt => GroundMaterial::Dirt,
        GroundMaterial::Mud => GroundMaterial::Mud,
        GroundMaterial::Rock => GroundMaterial::Rock,
        GroundMaterial::TrenchFloor | GroundMaterial::TrenchWall => GroundMaterial::TrenchWall,
        GroundMaterial::BermTop | GroundMaterial::BermFace => GroundMaterial::BermFace,
    }
}
