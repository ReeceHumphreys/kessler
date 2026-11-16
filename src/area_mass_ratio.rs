//! Area-to-mass ratio distributions of the NASA Standard Breakup Model (Johnson et al., 2001), with the 8–11 cm
//! bridge as in ESA's implementation (Schuhmacher et al., 2021).

use crate::{exp10, SatKind};

use rand::distributions::Distribution;
use rand::Rng;
use rand_distr::Normal;

/// Samples the area-to-mass ratio [m²/kg] of a fragment with the given characteristic length [m].
pub fn am_ratio(sat_kind: SatKind, characteristic_len: f32, rng: &mut impl Rng) -> f32 {
    let lambda_lc = characteristic_len.log10();
    // The size thresholds are on L_c itself [m]; lambda_lc only feeds the distribution parameters.
    if characteristic_len > 0.11 {
        exp10(large_chi(&sat_kind, lambda_lc, rng))
    } else if characteristic_len < 0.08 {
        exp10(small_chi(lambda_lc, rng))
    } else {
        // Linear bridge between the two regimes
        let y0 = exp10(small_chi(lambda_lc, rng));
        let y1 = exp10(large_chi(&sat_kind, lambda_lc, rng));
        y0 + (characteristic_len - 0.08) * (y1 - y0) / 0.03
    }
}

/// log10(A/M) for fragments over 11 cm: the bimodal density α·N(μ₁, σ₁) + (1 − α)·N(μ₂, σ₂), i.e. a mixture that
/// picks mode 1 with probability α. (Not α·n₁ + (1 − α)·n₂, which is a narrower, unimodal normal.)
fn large_chi(sat_kind: &SatKind, lambda_lc: f32, rng: &mut impl Rng) -> f32 {
    let (mu, sigma) = if rng.gen::<f32>() < alpha(sat_kind, lambda_lc) {
        (mean_1(sat_kind, lambda_lc), sigma_1(sat_kind, lambda_lc))
    } else {
        (mean_2(sat_kind, lambda_lc), sigma_2(sat_kind, lambda_lc))
    };
    Normal::new(mu, sigma).unwrap().sample(rng)
}

/// log10(A/M) for fragments under 8 cm.
fn small_chi(lambda_lc: f32, rng: &mut impl Rng) -> f32 {
    Normal::new(mean_soc(lambda_lc), sigma_soc(lambda_lc)).unwrap().sample(rng)
}

// ------------------ Helpers ------------------
fn distribution_constant<F>(
    lambda_lc: f32,
    lower: f32,
    upper: f32,
    lower_return: f32,
    upper_return: f32,
    mid_function: F,
) -> f32
where
    F: Fn(f32) -> f32,
{
    if lambda_lc <= lower {
        lower_return
    } else if lambda_lc >= upper {
        upper_return
    } else {
        mid_function(lambda_lc)
    }
}

// ------------------ Means and Std. Devs ------------------
fn alpha(sat_kind: &SatKind, lambda_lc: f32) -> f32 {
    match sat_kind {
        SatKind::RocketBody => distribution_constant(lambda_lc, -1.4, 0.0, 1.0, 0.5, |lambda_lc| {
            1.0 - 0.3571 * (lambda_lc + 1.4)
        }),
        _ => distribution_constant(lambda_lc, -1.95, 0.55, 0.0, 1.0, |lambda_lc| {
            0.3 + 0.4 * (lambda_lc + 1.2)
        }),
    }
}

// Handles RB and SC case
fn mean_1(sat_kind: &SatKind, lambda_lc: f32) -> f32 {
    match sat_kind {
        SatKind::RocketBody => distribution_constant(lambda_lc, -0.5, 0.0, -0.45, -0.9, |lambda_lc| {
            -0.45 - 0.9 * (lambda_lc + 0.5)
        }),
        _ => distribution_constant(lambda_lc, -1.1, 0.0, -0.6, -0.95, |lambda_lc| {
            -0.6 - std::f32::consts::FRAC_1_PI * (lambda_lc + 1.1)
        }),
    }
}

// Handles RB and SC case
fn sigma_1(sat_kind: &SatKind, lambda_lc: f32) -> f32 {
    match sat_kind {
        SatKind::RocketBody => 0.55,
        _ => distribution_constant(lambda_lc, -1.3, -0.3, 0.1, 0.3, |lambda_lc| {
            0.1 + 0.2 * (lambda_lc + 1.3)
        }),
    }
}

// Handles RB and SC case
fn mean_2(sat_kind: &SatKind, lambda_lc: f32) -> f32 {
    match sat_kind {
        SatKind::RocketBody => -0.9,
        _ => distribution_constant(lambda_lc, -0.7, -0.1, -1.2, -2.0, |lambda_lc| {
            -1.2 - 1.333 * (lambda_lc + 0.7)
        }),
    }
}

// Handles RB and SC case
fn sigma_2(sat_kind: &SatKind, lambda_lc: f32) -> f32 {
    match sat_kind {
        // 0.28 - 0.1636(λ + 1) is the form continuous at both ends; ESA's code has -0.28 here, ODAP 0.29.
        SatKind::RocketBody => distribution_constant(lambda_lc, -1.0, 0.1, 0.28, 0.1, |lambda_lc| {
            0.28 - 0.1636 * (lambda_lc + 1.0)
        }),
        _ => distribution_constant(lambda_lc, -0.5, -0.3, 0.5, 0.3, |lambda_lc| 0.5 - (lambda_lc + 0.5)),
    }
}

fn mean_soc(lambda_lc: f32) -> f32 {
    distribution_constant(lambda_lc, -1.75, -1.25, -0.3, -1.0, |lambda_lc| {
        -0.3 - 1.4 * (lambda_lc + 1.75)
    })
}

fn sigma_soc(lambda_lc: f32) -> f32 {
    if lambda_lc <= -3.5 {
        0.2
    } else {
        0.2 + 0.1333 * (lambda_lc + 3.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn log10_am_stats(kind: SatKind, lc: f32) -> (f32, f32) {
        let mut rng = StdRng::seed_from_u64(1);
        let chi: Vec<f32> = (0..20_000).map(|_| am_ratio(kind, lc, &mut rng).log10()).collect();
        let mean = chi.iter().sum::<f32>() / chi.len() as f32;
        let var = chi.iter().map(|c| (c - mean).powi(2)).sum::<f32>() / chi.len() as f32;
        (mean, var.sqrt())
    }

    #[test]
    fn fragments_over_11cm_follow_the_bimodal_mixture() {
        // Spacecraft at L_c = 1 m: modes at -0.95 and -2.0 (σ 0.3 each), α = 0.78. The mixture's spread includes the
        // gap between the modes (σ ≈ 0.53); a weighted sum of two samples would give σ ≈ 0.24.
        let (kind, l) = (SatKind::Spacecraft, 0.0);
        let a = alpha(&kind, l);
        let (m1, m2, s1, s2) = (mean_1(&kind, l), mean_2(&kind, l), sigma_1(&kind, l), sigma_2(&kind, l));
        let mean = a * m1 + (1.0 - a) * m2;
        let sd = (a * s1 * s1 + (1.0 - a) * s2 * s2 + a * (1.0 - a) * (m1 - m2).powi(2)).sqrt();
        let (got_mean, got_sd) = log10_am_stats(kind, 1.0);
        assert!((got_mean - mean).abs() < 0.02, "mean {got_mean} vs {mean}");
        assert!((got_sd - sd).abs() < 0.02, "sd {got_sd} vs {sd}");
    }

    #[test]
    fn rocket_body_sigma_2_is_continuous() {
        let rb = &SatKind::RocketBody;
        assert_eq!(sigma_2(rb, -1.5), 0.28);
        assert!((sigma_2(rb, -0.5) - 0.1982).abs() < 1e-4);
        assert!((sigma_2(rb, 0.0999) - 0.1).abs() < 1e-3);
        assert_eq!(sigma_2(rb, 0.5), 0.1);
    }

    #[test]
    fn fragments_under_8cm_use_the_small_object_distribution() {
        let (mean, sd) = log10_am_stats(SatKind::RocketBody, 0.01);
        assert!((mean - mean_soc(-2.0)).abs() < 0.02);
        assert!((sd - sigma_soc(-2.0)).abs() < 0.02);
    }
}
