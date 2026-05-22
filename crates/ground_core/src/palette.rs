use serde::{Deserialize, Serialize};

use crate::color::{clamp01, Rgba8};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ramp {
    pub name: String,
    pub colors: Vec<Rgba8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Palette {
    pub id: String,
    pub ramps: Vec<Ramp>,
}

impl Palette {
    pub fn ramp(&self, name: &str) -> Option<&Ramp> {
        self.ramps.iter().find(|r| r.name == name)
    }

    pub fn sample(&self, ramp_name: &str, t: f32) -> Rgba8 {
        let Some(ramp) = self.ramp(ramp_name) else {
            return Rgba8::opaque(255, 0, 255);
        };
        sample_ramp(&ramp.colors, t)
    }

    pub fn color_names(&self) -> Vec<String> {
        self.ramps.iter().map(|r| r.name.clone()).collect()
    }
}

pub fn sample_ramp(colors: &[Rgba8], t: f32) -> Rgba8 {
    if colors.is_empty() {
        return Rgba8::opaque(255, 0, 255);
    }
    if colors.len() == 1 {
        return colors[0];
    }

    let t = clamp01(t);
    let scaled = t * (colors.len() - 1) as f32;
    let i0 = scaled.floor() as usize;
    let i1 = (i0 + 1).min(colors.len() - 1);
    let frac = scaled - i0 as f32;
    colors[i0].blend(colors[i1], frac)
}

pub fn muted_field_32() -> Palette {
    Palette {
        id: "muted_field_32".to_string(),
        ramps: vec![
            ramp("grass", &[0x1f3125, 0x314d2e, 0x4f6d3a, 0x75844d, 0xa6a66b]),
            ramp(
                "dry_grass",
                &[0x2e2b1f, 0x56502f, 0x80784b, 0xa89b63, 0xc3b67b],
            ),
            ramp("dirt", &[0x241b16, 0x443026, 0x6c4d37, 0x987149, 0xbc935f]),
            ramp("mud", &[0x171717, 0x2b2722, 0x473a2e, 0x65513a, 0x806848]),
            ramp("rock", &[0x20242a, 0x393f46, 0x59616a, 0x7d8790, 0xa2a9ad]),
            ramp(
                "trench_floor",
                &[0x14100d, 0x241913, 0x3a271b, 0x563921, 0x775331],
            ),
            ramp(
                "trench_wall",
                &[0x1c130f, 0x332217, 0x54351f, 0x7c5130, 0xa46f42],
            ),
            ramp(
                "berm_top",
                &[0x2a1d14, 0x4d3321, 0x765238, 0xa17451, 0xc3966a],
            ),
            ramp(
                "berm_face",
                &[0x18110c, 0x2b1d14, 0x4b311e, 0x704728, 0x986333],
            ),
            ramp("overlay_blue", &[0x0f2232, 0x1d4b65, 0x398ba8, 0x80c8d8]),
            ramp("overlay_red", &[0x2b1014, 0x68212c, 0xa63e45, 0xe0785f]),
            ramp("overlay_yellow", &[0x2e2410, 0x6e5721, 0xa8903e, 0xe2d66a]),
        ],
    }
}

fn ramp(name: &str, hexes: &[u32]) -> Ramp {
    Ramp {
        name: name.to_string(),
        colors: hexes.iter().copied().map(Rgba8::from_rgb_hex).collect(),
    }
}
