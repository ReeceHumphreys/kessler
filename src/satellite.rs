use ndarray::prelude::*;

#[derive(Debug, Clone)]
pub struct Satellite {
    pub position: Array1<f32>,
    pub velocity: Array1<f32>,
    pub mass: f32,
    pub characteristic_length: f32,
    pub sat_kind: SatKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SatKind {
    RB = 0,
    Soc = 1,
    SC = 2,
}

impl From<i32> for SatKind {
    fn from(i: i32) -> Self {
        match i {
            0 => SatKind::RB,
            1 => SatKind::Soc,
            2 => SatKind::SC,
            _ => panic!("Invalid SatKind"),
        }
    }
}
