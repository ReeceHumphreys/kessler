use ndarray::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::Float32Array;

#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct Satellite {
    pub mass: f32,
    pub characteristic_length: f32,
    pub sat_kind: SatKind,
    position: Array1<f32>,
    velocity: Array1<f32>,
}

#[wasm_bindgen]
impl Satellite {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(position: Vec<f32>, velocity: Vec<f32>, mass: f32, sat_kind: SatKind) -> Self {
        Self {
            position: Array1::from_vec(position),
            velocity: Array1::from_vec(velocity),
            mass,
            characteristic_length: calculate_characteristic_length_from_mass(mass),
            sat_kind,
        }
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen(constructor)]
    pub fn new(position: JsValue, velocity: JsValue, mass: f32, sat_kind: SatKind) -> Self {
        let position_array: Float32Array = position.into();
        let velocity_array: Float32Array = velocity.into();

        assert_eq!(position_array.length(), 3, "Position array must have 3 elements.");
        assert_eq!(velocity_array.length(), 3, "Velocity array must have 3 elements.");

        Self {
            position: Array1::from_vec(position_array.to_vec()),
            velocity: Array1::from_vec(velocity_array.to_vec()),
            mass,
            characteristic_length: calculate_characteristic_length_from_mass(mass),
            sat_kind,
        }
    }

    pub fn set_position(&mut self, position: &[f32]) {
        self.position = Array1::from_vec(position.to_vec());
    }

    pub fn set_velocity(&mut self, velocity: &[f32]) {
        self.velocity = Array1::from_vec(velocity.to_vec());
    }

    pub fn get_position(&self) -> Vec<f32> {
        self.position.to_vec()
    }

    pub fn get_velocity(&self) -> Vec<f32> {
        self.velocity.to_vec()
    }
}

#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SatKind {
    Rb = 0,
    Soc = 1,
    Sc = 2,
}

impl From<i32> for SatKind {
    fn from(i: i32) -> Self {
        match i {
            0 => SatKind::Rb,
            1 => SatKind::Soc,
            2 => SatKind::Sc,
            _ => panic!("Invalid SatKind"),
        }
    }
}

const PI: f32 = std::f32::consts::PI;

fn calculate_characteristic_length_from_mass(mass: f32) -> f32 {
    const MUL_92_937_PI: f32 = 92.937 * PI;
    const INV_2_26: f32 = 1.0 / 2.26;

    ((6.0 * mass) / MUL_92_937_PI).powf(INV_2_26)
}
