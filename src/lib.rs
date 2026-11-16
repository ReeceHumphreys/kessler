//! Simulates fragmentation events in orbit with the NASA Standard Breakup Model.
//!
//! The whole interface is [`explosion`] and [`collision`]: give the masses and kinds of the objects involved, pick
//! the smallest fragment size to model, and get back the [`Fragments`] produced. All inputs and outputs are SI (m,
//! kg, m/s). Randomness comes from the `rng` you pass in, so a seeded generator gives reproducible results.

mod area_mass_ratio;

use area_mass_ratio::am_ratio;
use rand::Rng;
use rand_distr::{Distribution, Normal, UnitSphere};

/// The kind of object breaking up. Rocket bodies use their own area-to-mass distribution; spacecraft and debris
/// share the spacecraft one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatKind {
    RocketBody,
    Spacecraft,
    Debris,
}

/// The fragments produced by one event. Every `Vec` has one entry per fragment, in the same order. All values are SI.
#[derive(Debug, Clone, PartialEq)]
pub struct Fragments {
    /// [m]
    pub characteristic_length: Vec<f32>,
    /// Area-to-mass ratio [m²/kg]
    pub am_ratio: Vec<f32>,
    /// [m²]
    pub area: Vec<f32>,
    /// [kg]
    pub mass: Vec<f32>,
    /// Velocity change [m/s] imparted by the breakup, in a random direction.
    pub delta_v: Vec<[f32; 3]>,
}

/// Breaks up a single object of `mass` [kg]. Only fragments at least `min_characteristic_length` [m] across are
/// modelled.
pub fn explosion(mass: f32, kind: SatKind, min_characteristic_length: f32, rng: &mut impl Rng) -> Fragments {
    let params = Params {
        count: (6.0 * min_characteristic_length.powf(-1.6)) as usize,
        min_characteristic_length,
        max_characteristic_length: characteristic_length(mass),
        power_law_exponent: -2.6,
        kind,
        delta_v_offset: [0.2, 1.85],
    };
    breakup(&params, rng)
}

/// Breaks up two objects colliding at `impact_speed` [m/s] relative to each other. Masses are in [kg]; only
/// fragments at least `min_characteristic_length` [m] across are modelled. The order of `a` and `b` doesn't matter.
pub fn collision(
    mass_a: f32,
    kind_a: SatKind,
    mass_b: f32,
    kind_b: SatKind,
    impact_speed: f32,
    min_characteristic_length: f32,
    rng: &mut impl Rng,
) -> Fragments {
    let (m_targ, m_proj) = if mass_b > mass_a { (mass_b, mass_a) } else { (mass_a, mass_b) };
    let m = if is_catastrophic(m_proj, m_targ, impact_speed) {
        m_proj + m_targ
    } else {
        // TODO: Have seen conflicting definitions of this. Need to find most correct one.
        m_proj * impact_speed / 1e3 // [km/s], as the model's fit expects
    };
    let kind = if kind_a == SatKind::RocketBody || kind_b == SatKind::RocketBody {
        SatKind::RocketBody
    } else {
        SatKind::Debris
    };

    let params = Params {
        count: (0.1 * m.powf(0.75) * min_characteristic_length.powf(-1.71)) as usize,
        min_characteristic_length,
        max_characteristic_length: characteristic_length(m_targ),
        power_law_exponent: -2.71,
        kind,
        delta_v_offset: [0.9, 2.9],
    };
    breakup(&params, rng)
}

/// Everything that differs between an explosion and a collision; the rest of the model is shared.
struct Params {
    count: usize,
    min_characteristic_length: f32,
    max_characteristic_length: f32,
    power_law_exponent: f32,
    kind: SatKind,
    delta_v_offset: [f32; 2],
}

fn breakup(p: &Params, rng: &mut impl Rng) -> Fragments {
    let characteristic_length: Vec<f32> = (0..p.count)
        .map(|_| {
            let y = rng.gen_range(0.0..=1.0);
            power_law(p.min_characteristic_length, p.max_characteristic_length, p.power_law_exponent, y)
        })
        .collect();
    let am_ratio: Vec<f32> = characteristic_length.iter().map(|&lc| am_ratio(p.kind, lc, rng)).collect();
    let area: Vec<f32> = characteristic_length.iter().map(|&lc| area(lc)).collect();
    // TODO: Implement mass conservation
    let mass = area.iter().zip(&am_ratio).map(|(a, am)| a / am).collect();
    let delta_v = am_ratio.iter().map(|&am| delta_v(p.delta_v_offset, am, rng)).collect();

    Fragments { characteristic_length, am_ratio, area, mass, delta_v }
}

/// Characteristic length [m] of a satellite, from its mass [kg].
fn characteristic_length(mass: f32) -> f32 {
    const MUL_92_937_PI: f32 = 92.937 * std::f32::consts::PI;
    const INV_2_26: f32 = 1.0 / 2.26;
    ((6.0 * mass) / MUL_92_937_PI).powf(INV_2_26)
}

/// True if the relative kinetic energy of the collision exceeds 40 J/g of target mass.
/// Masses in [kg], `v_impact` in [m/s].
fn is_catastrophic(m_proj: f32, m_targ: f32, v_impact: f32) -> bool {
    let ke = 0.5 * m_proj * v_impact.powi(2); // [J]
    ke / (m_targ * 1e3) > 40.0
}

fn area(characteristic_length: f32) -> f32 {
    if characteristic_length < 0.00167 {
        0.540424 * characteristic_length.powi(2)
    } else {
        0.556945 * characteristic_length.powf(2.0047077)
    }
}

fn delta_v(offset: [f32; 2], am_ratio: f32, rng: &mut impl Rng) -> [f32; 3] {
    let mean = offset[0] * am_ratio.log10() + offset[1];
    let magnitude = 10.0f32.powf(Normal::new(mean, 0.4).unwrap().sample(rng));
    // TODO: Need to handle NaNs from powf
    let magnitude = if magnitude.is_nan() { 0.0 } else { magnitude };
    let direction: [f32; 3] = UnitSphere.sample(rng);
    direction.map(|c| magnitude * c)
}

/// Samples the power law between `x0` and `x1` with exponent `n`, given a uniform sample `y` in [0, 1].
fn power_law(x0: f32, x1: f32, n: f32, y: f32) -> f32 {
    let step = x1.powf(n + 1.) - x0.powf(n + 1.) * y + x0.powf(n + 1.);
    step.powf(1. / (n + 1.))
}
