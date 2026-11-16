//! WebAssembly bindings for [`kessler`].

use rand::rngs::StdRng;
use rand::SeedableRng;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum SatKind {
    RocketBody,
    Spacecraft,
    Debris,
}

impl From<SatKind> for kessler::SatKind {
    fn from(kind: SatKind) -> Self {
        match kind {
            SatKind::RocketBody => kessler::SatKind::RocketBody,
            SatKind::Spacecraft => kessler::SatKind::Spacecraft,
            SatKind::Debris => kessler::SatKind::Debris,
        }
    }
}

#[wasm_bindgen]
pub struct Fragments(kessler::Fragments);

#[wasm_bindgen]
impl Fragments {
    #[wasm_bindgen(getter, js_name = characteristicLength)]
    pub fn characteristic_length(&self) -> Vec<f32> {
        self.0.characteristic_length.clone()
    }

    #[wasm_bindgen(getter, js_name = amRatio)]
    pub fn am_ratio(&self) -> Vec<f32> {
        self.0.am_ratio.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn area(&self) -> Vec<f32> {
        self.0.area.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn mass(&self) -> Vec<f32> {
        self.0.mass.clone()
    }

    /// [m/s], flattened as `[x0, y0, z0, x1, y1, z1, ...]`.
    #[wasm_bindgen(getter, js_name = deltaV)]
    pub fn delta_v(&self) -> Vec<f32> {
        self.0.delta_v.iter().flatten().copied().collect()
    }
}

/// Breaks up a single object. Inputs and outputs are SI (m, kg, m/s). Pass a `seed` for reproducible results.
#[wasm_bindgen]
pub fn explosion(mass: f32, kind: SatKind, min_characteristic_length: f32, seed: Option<u32>) -> Fragments {
    Fragments(kessler::explosion(mass, kind.into(), min_characteristic_length, &mut rng(seed)))
}

/// Breaks up two colliding objects. Inputs and outputs are SI (m, kg, m/s). Pass a `seed` for reproducible results.
#[wasm_bindgen]
pub fn collision(
    mass_a: f32,
    kind_a: SatKind,
    mass_b: f32,
    kind_b: SatKind,
    impact_speed: f32,
    min_characteristic_length: f32,
    seed: Option<u32>,
) -> Fragments {
    let f = kessler::collision(
        mass_a,
        kind_a.into(),
        mass_b,
        kind_b.into(),
        impact_speed,
        min_characteristic_length,
        &mut rng(seed),
    );
    Fragments(f)
}

fn rng(seed: Option<u32>) -> StdRng {
    seed.map_or_else(StdRng::from_entropy, |s| StdRng::seed_from_u64(s.into()))
}
