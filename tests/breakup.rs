use kessler::{collision, explosion, Fragments, InputError, SatKind};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

fn assert_consistent(f: &Fragments) {
    let n = f.characteristic_length.len();
    assert!(n > 0);
    assert!([f.am_ratio.len(), f.area.len(), f.mass.len(), f.delta_v.len()]
        .iter()
        .all(|&len| len == n));
    for i in 0..n {
        assert!((f.mass[i] - f.area[i] / f.am_ratio[i]).abs() <= f.mass[i].abs() * 1e-6);
    }
}

#[test]
fn same_seed_gives_same_fragments() {
    let run = |seed| explosion(839.0, SatKind::RocketBody, 0.05, &mut StdRng::seed_from_u64(seed)).unwrap();
    assert_eq!(run(42), run(42));
    assert_ne!(run(42), run(7));
}

#[test]
fn collision_ignores_argument_order() {
    let f = collision(
        500.0,
        SatKind::Spacecraft,
        5.0,
        SatKind::Spacecraft,
        9_500.0,
        0.1,
        &mut rng(),
    )
    .unwrap();
    assert_eq!(
        f,
        collision(
            5.0,
            SatKind::Spacecraft,
            500.0,
            SatKind::Spacecraft,
            9_500.0,
            0.1,
            &mut rng()
        )
        .unwrap()
    );
    assert_consistent(&f);
}

fn total_mass(f: &Fragments) -> f64 {
    f.mass.iter().map(|&m| f64::from(m)).sum()
}

fn largest(v: &[f32]) -> f32 {
    v.iter().copied().fold(0.0, f32::max)
}

#[test]
fn fragments_carry_the_input_mass() {
    for seed in 0..10 {
        let mut rng = StdRng::seed_from_u64(seed);
        let cases = [
            (explosion(839.0, SatKind::RocketBody, 0.1, &mut rng).unwrap(), 839.0),
            (
                collision(
                    500.0,
                    SatKind::Spacecraft,
                    5.0,
                    SatKind::Spacecraft,
                    9_500.0,
                    0.1,
                    &mut rng,
                )
                .unwrap(),
                505.0,
            ),
            (
                collision(
                    500.0,
                    SatKind::Spacecraft,
                    5.0,
                    SatKind::Spacecraft,
                    1_800.0,
                    0.1,
                    &mut rng,
                )
                .unwrap(),
                505.0,
            ),
        ];
        for (f, input) in cases {
            assert_consistent(&f);
            let total = total_mass(&f);
            assert!(
                (total - input).abs() < 1e-3 * input,
                "seed {seed}: {total} kg of {input} kg"
            );
        }
    }
}

#[test]
fn collision_threshold_is_40_joules_per_gram() {
    // 5 kg at 1.8 km/s into 500 kg: 8.1e6 J / 5e5 g = 16.2 J/g. Only the projectile's M_p·v² = 16.2 kg worth of
    // small fragments, and the target survives as one cratered fragment.
    let non_catastrophic = collision(
        500.0,
        SatKind::Spacecraft,
        5.0,
        SatKind::Spacecraft,
        1_800.0,
        0.1,
        &mut rng(),
    )
    .unwrap();
    assert!(non_catastrophic.mass.len() < 100);
    assert!(largest(&non_catastrophic.mass) >= 450.0);
    // At 9.5 km/s: 451 J/g, so both fragment completely: far more fragments, none of them the intact target.
    let catastrophic = collision(
        500.0,
        SatKind::Spacecraft,
        5.0,
        SatKind::Spacecraft,
        9_500.0,
        0.1,
        &mut rng(),
    )
    .unwrap();
    assert!(catastrophic.mass.len() > 500);
    assert!(largest(&catastrophic.characteristic_length) <= 1.0);
}

#[test]
fn explosion_puts_leftover_mass_in_2_to_8_large_fragments() {
    for seed in 0..20 {
        let f = explosion(1000.0, SatKind::RocketBody, 0.1, &mut StdRng::seed_from_u64(seed)).unwrap();
        let large = above(&f, 1.0);
        assert!((1..=8).contains(&large), "seed {seed}: {large} fragments over 1 m");
    }
}

#[test]
fn collision_delta_v_is_capped_at_1_3_impact_speed() {
    let f = collision(
        500.0,
        SatKind::Spacecraft,
        5.0,
        SatKind::Spacecraft,
        1_000.0,
        0.01,
        &mut rng(),
    )
    .unwrap();
    let fastest = f
        .delta_v
        .iter()
        .map(|v| v.iter().map(|c| c * c).sum::<f32>().sqrt())
        .fold(0.0, f32::max);
    assert!(fastest <= 1_300.0 * 1.0001, "fastest {fastest} m/s");
}

/// Number of fragments with characteristic length above `lc`.
fn above(f: &Fragments, lc: f32) -> usize {
    f.characteristic_length.iter().filter(|&&l| l > lc).count()
}

#[test]
fn agrees_with_space_agency_implementations() {
    // Inter-agency comparison (Rossi et al., 2006, via Letizia 2016 tables A.1-A.2): each count must land within the
    // spread of ASI, ESA, NASA and Letizia's CiELO. CNSA and DLR are outliers by an order of magnitude, so they're left
    // out.
    let f = explosion(1000.0, SatKind::RocketBody, 0.001, &mut rng()).unwrap();
    assert!(
        (324_886..=434_928).contains(&above(&f, 0.001)),
        "> 1 mm: {}",
        above(&f, 0.001)
    );
    assert!(
        (8_159..=10_731).contains(&above(&f, 0.01)),
        "> 1 cm: {}",
        above(&f, 0.01)
    );
    assert!((206..=253).contains(&above(&f, 0.1)), "> 10 cm: {}", above(&f, 0.1));
    assert!((1..=8).contains(&above(&f, 1.0)), "> 1 m: {}", above(&f, 1.0));

    let f = collision(
        1000.0,
        SatKind::RocketBody,
        10.0,
        SatKind::Spacecraft,
        10_000.0,
        0.001,
        &mut rng(),
    )
    .unwrap();
    assert!(
        (2_416_783..=4_723_391).contains(&above(&f, 0.001)),
        "> 1 mm: {}",
        above(&f, 0.001)
    );
    assert!((850..=1_539).contains(&above(&f, 0.1)), "> 10 cm: {}", above(&f, 0.1));
}

#[test]
fn fragments_are_physical_across_seeds() {
    for seed in 0..30 {
        let mut rng = StdRng::seed_from_u64(seed);
        let f = collision(
            500.0,
            SatKind::Spacecraft,
            5.0,
            SatKind::Spacecraft,
            9_500.0,
            0.1,
            &mut rng,
        )
        .unwrap();
        for i in 0..f.mass.len() {
            assert!(
                f.characteristic_length[i] >= 0.1,
                "seed {seed}: fragment {i} below the minimum length"
            );
            assert!(
                f.am_ratio[i].is_finite() && f.am_ratio[i] > 0.0,
                "seed {seed}: A/M {}",
                f.am_ratio[i]
            );
            assert!(f.mass[i] > 0.0, "seed {seed}: mass {}", f.mass[i]);
        }
    }
}

#[test]
fn rejects_input_outside_the_model() {
    let kind = SatKind::Spacecraft;
    for mass in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(explosion(mass, kind, 0.1, &mut rng()), Err(InputError::Mass));
        assert_eq!(
            collision(500.0, kind, mass, kind, 1_000.0, 0.1, &mut rng()),
            Err(InputError::Mass)
        );
    }
    for lc in [0.0, 0.0009, 1.0, f32::NAN] {
        assert_eq!(
            explosion(100.0, kind, lc, &mut rng()),
            Err(InputError::MinCharacteristicLength)
        );
    }
    for speed in [0.0, -1.0, f32::NAN] {
        assert_eq!(
            collision(500.0, kind, 5.0, kind, speed, 0.1, &mut rng()),
            Err(InputError::ImpactSpeed)
        );
    }
}
