use kessler::{collision, explosion, Fragments, SatKind};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn rng() -> StdRng {
    StdRng::seed_from_u64(42)
}

fn assert_consistent(f: &Fragments) {
    let n = f.characteristic_length.len();
    assert!(n > 0);
    assert!([f.am_ratio.len(), f.area.len(), f.mass.len(), f.delta_v.len()].iter().all(|&len| len == n));
    for i in 0..n {
        assert!((f.mass[i] - f.area[i] / f.am_ratio[i]).abs() <= f.mass[i].abs() * 1e-6);
    }
}

#[test]
fn explosion_fragment_count_follows_power_law() {
    let f = explosion(839.0, SatKind::RocketBody, 0.1, &mut rng());
    assert_eq!(f.characteristic_length.len(), (6.0 * 0.1f32.powf(-1.6)) as usize);
    assert_consistent(&f);
}

#[test]
fn same_seed_gives_same_fragments() {
    let run = |seed| explosion(839.0, SatKind::RocketBody, 0.05, &mut StdRng::seed_from_u64(seed));
    assert_eq!(run(42), run(42));
    assert_ne!(run(42), run(7));
}

#[test]
fn collision_ignores_argument_order() {
    let f = collision(500.0, SatKind::Spacecraft, 5.0, SatKind::Spacecraft, 9_500.0, 0.1, &mut rng());
    assert_eq!(f, collision(5.0, SatKind::Spacecraft, 500.0, SatKind::Spacecraft, 9_500.0, 0.1, &mut rng()));
    assert_consistent(&f);
}

#[test]
fn collision_threshold_is_40_joules_per_gram() {
    // 5 kg at 1.8 km/s into 500 kg: 8.1e6 J / 5e5 g = 16.2 J/g, non-catastrophic.
    // At 9.5 km/s: 451 J/g, catastrophic, so far more fragments.
    let count = |v| collision(500.0, SatKind::Spacecraft, 5.0, SatKind::Spacecraft, v, 0.1, &mut rng()).mass.len();
    assert_eq!(count(1_800.0), (0.1 * (5.0f32 * 1.8).powf(0.75) * 0.1f32.powf(-1.71)) as usize);
    assert_eq!(count(9_500.0), (0.1 * 505.0f32.powf(0.75) * 0.1f32.powf(-1.71)) as usize);
}

#[test]
fn fragments_are_physical_across_seeds() {
    for seed in 0..30 {
        let mut rng = StdRng::seed_from_u64(seed);
        let f = collision(500.0, SatKind::Spacecraft, 5.0, SatKind::Spacecraft, 9_500.0, 0.1, &mut rng);
        for i in 0..f.mass.len() {
            assert!(f.characteristic_length[i] >= 0.1, "seed {seed}: fragment {i} below the minimum length");
            assert!(f.am_ratio[i].is_finite() && f.am_ratio[i] > 0.0, "seed {seed}: A/M {}", f.am_ratio[i]);
            assert!(f.mass[i] > 0.0, "seed {seed}: mass {}", f.mass[i]);
        }
    }
}
