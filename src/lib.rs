//! Simulates fragmentation events in orbit with the NASA Standard Breakup Model.
//!
//! The whole interface is [`explosion`] and [`collision`]: give the masses and kinds of the objects involved, pick
//! the smallest fragment size to model, and get back the [`Fragments`] produced. All inputs and outputs are SI (m,
//! kg, m/s). Randomness comes from the `rng` you pass in, so a seeded generator gives reproducible results.
//!
//! Sources: Johnson et al. (2001) for the model itself, Krisko (2011) for its corrections, Letizia (2016, PhD thesis,
//! appendix A.1) for the collision Δv cap. Krisko (2011) also gives the mass conservation method: the power law covers
//! fragments up to 1 m, and the mass left over goes into a few large fragments.

mod area_mass_ratio;

// Runs the README's examples as doctests.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

use std::ops::RangeInclusive;

use area_mass_ratio::am_ratio;
use rand::{Rng, RngExt};
use rand_distr::{Distribution, Normal, UnitSphere};

/// The kind of object breaking up, which picks its area-to-mass distribution. Use `Spacecraft` for anything that
/// isn't a rocket body, including debris.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatKind {
    RocketBody,
    Spacecraft,
}

/// The fragments produced by one event. Every `Vec` has one entry per fragment, in the same order. All values are SI.
#[derive(Debug, Clone, Default, PartialEq)]
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

/// Why [`explosion`] or [`collision`] rejected its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputError {
    /// A mass wasn't a positive, finite number.
    Mass,
    /// The minimum characteristic length was outside the 1 mm to 1 m range the model is valid for (Krisko, 2011).
    MinCharacteristicLength,
    /// The impact speed wasn't a positive, finite number.
    ImpactSpeed,
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            InputError::Mass => "mass must be a positive, finite number of kg",
            InputError::MinCharacteristicLength => {
                "min_characteristic_length must be at least 0.001 m and below 1 m, the range the model is valid for"
            }
            InputError::ImpactSpeed => "impact_speed must be a positive, finite number of m/s",
        })
    }
}

impl std::error::Error for InputError {}

fn check(valid: bool, error: InputError) -> Result<(), InputError> {
    if valid { Ok(()) } else { Err(error) }
}

fn positive(x: f32) -> bool {
    x > 0.0 && x.is_finite()
}

fn check_min_characteristic_length(lc: f32) -> Result<(), InputError> {
    check(
        (0.001..POWER_LAW_MAX).contains(&lc),
        InputError::MinCharacteristicLength,
    )
}

/// Breaks up a single object of `mass` [kg]. Only fragments at least `min_characteristic_length` [m] across are
/// modelled; together they carry all of `mass`, most of it in 2–8 fragments over 1 m.
///
/// # Errors
///
/// If `mass` isn't positive and finite, or `min_characteristic_length` isn't in [0.001, 1) m.
pub fn explosion(
    mass: f32,
    kind: SatKind,
    min_characteristic_length: f32,
    rng: &mut impl Rng,
) -> Result<Fragments, InputError> {
    check(positive(mass), InputError::Mass)?;
    check_min_characteristic_length(min_characteristic_length)?;
    let max_characteristic_length = characteristic_length(mass).min(POWER_LAW_MAX);
    let n = |lc: f32| 6.0 * lc.powf(-1.6);
    let params = Params {
        count: (n(min_characteristic_length) - n(max_characteristic_length)) as usize,
        min_characteristic_length,
        max_characteristic_length,
        power_law_exponent: -2.6,
        kind,
        delta_v_offset: [0.2, 1.85],
        cloud_mass: mass,
        input_mass: mass,
        // Krisko (2011): the bulk of the mass is in two to eight large pieces (tanks, nozzles, ...) over 1 m.
        remainder_pieces: 2..=8,
        remainder_characteristic_length: None,
        max_delta_v: None,
    };
    Ok(breakup(&params, rng))
}

/// Breaks up two objects colliding at `impact_speed` [m/s] relative to each other. Masses are in [kg]; only
/// fragments at least `min_characteristic_length` [m] across are modelled. The order of `a` and `b` doesn't matter.
///
/// Above 40 J/g of target mass the collision is catastrophic and both objects fragment completely, with most of the
/// mass in a few fragments of about 1 m. Otherwise the projectile is destroyed and the target survives as one large
/// cratered fragment. Either way the fragments carry all of `mass_a + mass_b`.
///
/// # Errors
///
/// If a mass or `impact_speed` isn't positive and finite, or `min_characteristic_length` isn't in [0.001, 1) m.
pub fn collision(
    mass_a: f32,
    kind_a: SatKind,
    mass_b: f32,
    kind_b: SatKind,
    impact_speed: f32,
    min_characteristic_length: f32,
    rng: &mut impl Rng,
) -> Result<Fragments, InputError> {
    check(positive(mass_a) && positive(mass_b), InputError::Mass)?;
    check(positive(impact_speed), InputError::ImpactSpeed)?;
    check_min_characteristic_length(min_characteristic_length)?;
    let (m_targ, m_proj) = if mass_b > mass_a {
        (mass_b, mass_a)
    } else {
        (mass_a, mass_b)
    };
    let catastrophic = is_catastrophic(m_proj, m_targ, impact_speed);
    let m = if catastrophic {
        m_proj + m_targ
    } else {
        // Krisko (2011): M_p·v² with v in km/s. Johnson et al. (2001) printed M_p·v; Krisko confirmed the square
        // (Letizia, 2016, A.1).
        m_proj * (impact_speed / 1e3).powi(2)
    };
    let kind = if kind_a == SatKind::RocketBody || kind_b == SatKind::RocketBody {
        SatKind::RocketBody
    } else {
        SatKind::Spacecraft
    };

    let max_characteristic_length = characteristic_length(m_targ).min(POWER_LAW_MAX);
    let n = |lc: f32| 0.1 * m.powf(0.75) * lc.powf(-1.71);
    let params = Params {
        count: (n(min_characteristic_length) - n(max_characteristic_length)) as usize,
        min_characteristic_length,
        max_characteristic_length,
        power_law_exponent: -2.71,
        kind,
        delta_v_offset: [0.9, 2.9],
        cloud_mass: m,
        input_mass: m_proj + m_targ,
        // Krisko (2011): catastrophic leftovers go into "several large fragments" in the last bin below 1 m (we use
        // the explosion's 2–8); a non-catastrophic one is a single fragment, the cratered target.
        remainder_pieces: if catastrophic { 2..=8 } else { 1..=1 },
        remainder_characteristic_length: catastrophic.then_some(max_characteristic_length),
        // Without a cap a few fragments get unphysical speeds, around 60 km/s (Letizia, 2016, A.1).
        max_delta_v: Some(1.3 * impact_speed),
    };
    Ok(breakup(&params, rng))
}

/// The power law's upper limit [m]; larger fragments are placed by mass conservation instead (Krisko, 2011).
const POWER_LAW_MAX: f32 = 1.0;

/// Everything that differs between an explosion and a collision; the rest of the model is shared.
struct Params {
    count: usize,
    min_characteristic_length: f32,
    max_characteristic_length: f32,
    power_law_exponent: f32,
    kind: SatKind,
    delta_v_offset: [f32; 2],
    /// Mass [kg] the power-law fragments may add up to at most.
    cloud_mass: f32,
    /// Total mass [kg] all fragments add up to.
    input_mass: f32,
    /// How many large fragments share the mass the power law leaves over.
    remainder_pieces: RangeInclusive<usize>,
    /// Their characteristic length [m]; `None` derives it from each one's mass.
    remainder_characteristic_length: Option<f32>,
    /// Collisions: Δv [m/s] samples at or above this are redrawn.
    max_delta_v: Option<f32>,
}

fn breakup(p: &Params, rng: &mut impl Rng) -> Fragments {
    // (characteristic length, A/M, area) of each power-law fragment
    let lc_dist = PowerLaw::new(
        p.min_characteristic_length,
        p.max_characteristic_length,
        p.power_law_exponent,
    );
    let mut cloud: Vec<(f32, f32, f32)> = (0..p.count)
        .map(|_| {
            let lc = lc_dist.sample(rng);
            (lc, am_ratio(p.kind, lc, rng), area(lc))
        })
        .collect();
    let mass_of = |&(_, am, area): &(f32, f32, f32)| f64::from(area / am);
    let mut cloud_mass: f64 = cloud.iter().map(mass_of).sum();
    if cloud_mass > f64::from(p.cloud_mass) {
        // Krisko (2011): fragments are deposited from the smallest up until the mass is reached.
        cloud.sort_by(|a, b| a.0.total_cmp(&b.0));
        while cloud_mass > f64::from(p.cloud_mass) {
            cloud_mass -= cloud.pop().as_ref().map_or(0.0, mass_of);
        }
    }

    let mut fragments = Fragments::default();
    for (lc, am, area) in cloud {
        fragments.push(lc, am, area);
    }
    for mass in split(f64::from(p.input_mass) - cloud_mass, p.remainder_pieces.clone(), rng) {
        let lc = p
            .remainder_characteristic_length
            .unwrap_or_else(|| characteristic_length(mass));
        let area = area(lc);
        fragments.push(lc, area / mass, area);
    }
    fragments.delta_v = fragments.am_ratio.iter().map(|&am| delta_v(p, am, rng)).collect();
    fragments
}

impl Fragments {
    fn push(&mut self, characteristic_length: f32, am_ratio: f32, area: f32) {
        self.characteristic_length.push(characteristic_length);
        self.am_ratio.push(am_ratio);
        self.area.push(area);
        self.mass.push(area / am_ratio);
    }
}

/// Splits `mass` [kg] at uniformly random points into a random number of pieces from `pieces`.
fn split(mass: f64, pieces: RangeInclusive<usize>, rng: &mut impl Rng) -> Vec<f32> {
    if mass <= 0.0 {
        return Vec::new();
    }
    let mut cuts: Vec<f64> = (1..rng.random_range(pieces))
        .map(|_| rng.random_range(0.0..mass))
        .collect();
    cuts.extend([0.0, mass]);
    cuts.sort_by(f64::total_cmp);
    cuts.windows(2)
        .map(|w| (w[1] - w[0]) as f32)
        .filter(|&m| m > 0.0)
        .collect()
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

fn delta_v(p: &Params, am_ratio: f32, rng: &mut impl Rng) -> [f32; 3] {
    let log_normal = Normal::new(p.delta_v_offset[0] * am_ratio.log10() + p.delta_v_offset[1], 0.4).unwrap();
    let mut magnitude = exp10(log_normal.sample(rng));
    if let Some(max) = p.max_delta_v {
        // shortcut: stops redrawing after 1000 tries and uses the cap, which only happens at impact speeds of a few
        // m/s; switch to inverse-CDF truncated sampling if those ever matter.
        for _ in 0..1000 {
            if magnitude < max {
                break;
            }
            magnitude = exp10(log_normal.sample(rng));
        }
        magnitude = magnitude.min(max);
    }
    let direction: [f32; 3] = UnitSphere.sample(rng);
    direction.map(|c| magnitude * c)
}

/// The power law between `x0` and `x1` with exponent `n`, sampled by inverting its CDF.
struct PowerLaw {
    x0_pow: f32,
    span: f32,
    inv_exponent: f32,
}

impl PowerLaw {
    fn new(x0: f32, x1: f32, n: f32) -> Self {
        let x0_pow = x0.powf(n + 1.);
        PowerLaw {
            x0_pow,
            span: x1.powf(n + 1.) - x0_pow,
            inv_exponent: 1. / (n + 1.),
        }
    }

    fn sample(&self, rng: &mut impl Rng) -> f32 {
        let y: f32 = rng.random_range(0.0..=1.0);
        (self.span * y + self.x0_pow).powf(self.inv_exponent)
    }
}

/// 10^x, which `exp` computes faster than `powf`.
pub(crate) fn exp10(x: f32) -> f32 {
    (x * std::f32::consts::LN_10).exp()
}
