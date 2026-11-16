//! Prints the comparison tables for the README, to paste in after model changes:
//!
//! ```shell
//! cargo test --release --test benchmarks -- --ignored --nocapture
//! ```
//!
//! Reference counts are the inter-agency comparison of Rossi et al. (2006), as reproduced in Letizia (2016), tables
//! A.1-A.2 (which adds CiELO) and Humphreys (2021), table 1 (which adds the mass, area and Δv rows).

use kessler::{collision, explosion, Fragments, SatKind};
use rand::rngs::StdRng;
use rand::SeedableRng;

const RUNS: u64 = 10;

/// A table row: what is counted, how to count it, and each reference implementation's count (`None` if not reported).
type Row<const N: usize> = (&'static str, fn(&Fragments) -> usize, [Option<u32>; N]);

fn count(values: &[f32], threshold: f32) -> usize {
    values.iter().filter(|&&v| v > threshold).count()
}

fn lc(f: &Fragments, threshold: f32) -> usize {
    count(&f.characteristic_length, threshold)
}

fn speed(f: &Fragments) -> Vec<f32> {
    f.delta_v.iter().map(|v| v.iter().map(|c| c * c).sum::<f32>().sqrt()).collect()
}

/// Mean and standard deviation of each row's count over `RUNS` seeded runs.
fn stats<const N: usize>(rows: &[Row<N>], run: impl Fn(&mut StdRng) -> Fragments) -> Vec<(f64, f64)> {
    let counts: Vec<Vec<f64>> = (0..RUNS)
        .map(|seed| {
            let f = run(&mut StdRng::seed_from_u64(seed));
            rows.iter().map(|(_, count, _)| count(&f) as f64).collect()
        })
        .collect();
    (0..rows.len())
        .map(|i| {
            let n = RUNS as f64;
            let mean = counts.iter().map(|c| c[i]).sum::<f64>() / n;
            let var = counts.iter().map(|c| (c[i] - mean).powi(2)).sum::<f64>() / n;
            (mean, var.sqrt())
        })
        .collect()
}

/// Formats with thousands separators.
fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn table<const N: usize>(names: [&str; N], rows: &[Row<N>], stats: &[(f64, f64)]) -> String {
    let mut out = format!("| | Kessler | {} |\n|---|---|{}\n", names.join(" | "), "---|".repeat(N));
    for ((label, _, refs), (mean, sd)) in rows.iter().zip(stats) {
        let refs: Vec<String> = refs.iter().map(|r| r.map_or("–".into(), |r| thousands(r.into()))).collect();
        let kessler = format!("{} ± {}", thousands(mean.round() as u64), thousands(sd.round() as u64));
        out += &format!("| {label} | {kessler} | {} |\n", refs.join(" | "));
    }
    out
}

#[test]
#[ignore = "slow; prints the README tables"]
fn print_readme_tables() {
    let explosion_rows: [Row<6>; 7] = [
        ("L<sub>c</sub> > 1 mm", |f| lc(f, 0.001), [378_581, 37_865, 1_217_054, 324_886, 434_928, 378_568].map(Some)),
        ("L<sub>c</sub> > 1 cm", |f| lc(f, 0.01), [9_403, 960, 11_724, 8_159, 10_731, 10_312].map(Some)),
        ("L<sub>c</sub> > 10 cm", |f| lc(f, 0.1), [234, 32, 230, 206, 248, 253].map(Some)),
        ("L<sub>c</sub> > 1 m", |f| lc(f, 1.0), [7, 9, 0, 6, 8, 1].map(Some)),
        ("Mass > 1 g", |f| count(&f.mass, 1e-3), [Some(2_472), Some(254), Some(25_844), Some(2_093), Some(2_525), None]),
        ("Area > 1 cm²", |f| count(&f.area, 1e-4), [Some(5_878), None, Some(31_124), Some(5_024), Some(6_416), None]),
        ("Δv > 100 m/s", |f| count(&speed(f), 100.0), [Some(112_932), Some(11_380), Some(31_124), Some(98_717), Some(132_032), None]),
    ];
    let collision_rows: [Row<5>; 2] = [
        ("L<sub>c</sub> > 1 mm", |f| lc(f, 0.001), [2_416_795, 2_416_790, 4_723_391, 2_957_159, 2_416_783].map(Some)),
        ("L<sub>c</sub> > 10 cm", |f| lc(f, 0.1), [850, 935, 1_539, 862, 1_017].map(Some)),
    ];

    let explosion_stats = stats(&explosion_rows, |rng| explosion(1000.0, SatKind::RocketBody, 0.001, rng));
    let collision_stats = stats(&collision_rows, |rng| {
        collision(1000.0, SatKind::RocketBody, 10.0, SatKind::Debris, 10_000.0, 0.001, rng)
    });

    let tables = format!(
        "**Explosion** of a 1000 kg rocket body:\n\n{}\n\
         **Catastrophic collision** of a 1000 kg rocket body with a 10 kg projectile at 10 km/s:\n\n{}",
        table(["ASI", "CNSA", "DLR", "ESA", "NASA", "CiELO"], &explosion_rows, &explosion_stats),
        table(["ASI", "CNSA", "ESA", "NASA", "CiELO"], &collision_rows, &collision_stats),
    );
    println!("{tables}");
}
