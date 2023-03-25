use ndarray::prelude::*;

#[derive(Debug, Clone)]
pub struct Satellite {
    pub position: Array1<f32>,
    pub velocity: Array1<f32>,
    pub mass: f32,
    pub characteristic_length: f32,
    pub sat_kind: SatKind,
}

impl Satellite {
    pub fn new(
        position: impl Into<[f32; 3]>,
        velocity: impl Into<[f32; 3]>,
        mass: f32,
        characteristic_length: f32,
        sat_kind: SatKind,
    ) -> Self {
        Self {
            position: Array1::from(position.into().to_vec()),
            velocity: Array1::from(velocity.into().to_vec()),
            mass,
            characteristic_length,
            sat_kind,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
