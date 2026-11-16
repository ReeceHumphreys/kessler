# Kessler.rs

Kessler is a Rust package that allows users to simulate fragmentation events
in low earth orbit according to the NASA standard breakup model.

**IMPORTANT**: Until this package reaches version 1.0, the API is subject to change and the accuracy of results cannot be guaranteed.

- [Kessler.rs](#kesslerrs)
  - [Installation](#installation)
  - [Usage](#usage)
  - [Contributing](#contributing)
  - [License](#license)
  - [TODO](#todo)

## Installation

To use Kessler, you'll need to have Rust and Cargo installed on your system.
You can install Rust and Cargo by following the instructions on the official
Rust website:

<https://www.rust-lang.org/tools/install>

Once you have Rust and Cargo installed, you can add Kessler to your project by
adding the following line to your `Cargo.toml` file:

```toml
[dependencies]
kessler = "0.2.0"
```

## Usage

The whole interface is two functions, `explosion` and `collision`. All inputs and outputs are SI (metres, kilograms,
metres per second). Randomness comes from the generator you pass in, so a seeded one gives reproducible results.

```rust
use kessler::{collision, explosion, SatKind};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() -> Result<(), kessler::InputError> {
    let mut rng = StdRng::seed_from_u64(42);

    // An 839 kg rocket body exploding, modelling fragments down to 10 cm.
    let debris = explosion(839.0, SatKind::RocketBody, 0.1, &mut rng)?;
    println!("{} fragments, first has mass {} kg", debris.mass.len(), debris.mass[0]);

    // A 5 kg object hitting a 500 kg spacecraft at 9.5 km/s.
    let debris = collision(500.0, SatKind::Spacecraft, 5.0, SatKind::Spacecraft, 9_500.0, 0.1, &mut rng)?;
    println!("{} fragments", debris.mass.len());
    Ok(())
}
```

Inputs outside the model's range (a non-positive mass or speed, or a minimum size outside 1 mm to 1 m) return an
`InputError`. Each event runs on one thread; to run many, give each thread its own generator:

```rust
# use kessler::{explosion, SatKind};
# use rand::{rngs::StdRng, SeedableRng};
let runs = std::thread::scope(|s| {
    let handles: Vec<_> = (0..8)
        .map(|seed| s.spawn(move || explosion(839.0, SatKind::RocketBody, 0.1, &mut StdRng::seed_from_u64(seed))))
        .collect();
    handles.into_iter().map(|h| h.join().unwrap()).collect::<Result<Vec<_>, _>>()
});
assert_eq!(runs.unwrap().len(), 8);
```

`Fragments` holds one entry per fragment in each of `characteristic_length` [m], `am_ratio` [m²/kg], `area` [m²],
`mass` [kg] and `delta_v` [m/s, 3D]. Fragments start where the event happened; add `delta_v` to the parent's velocity
to get each fragment's velocity.

Mass is conserved: the fragments always add up to the input mass. Following Krisko (2011), the power law covers
fragments up to 1 m, and the mass it leaves over goes into 2–8 large fragments for explosions, a few ~1 m fragments
for catastrophic collisions, or one cratered-target fragment for non-catastrophic ones.

### Comparison with other implementations

Counts of fragments above each size, mass, area and Δv, compared with the inter-agency comparison of the NASA breakup
model (Rossi et al., 2006) and Letizia's (2016) CiELO. Kessler's figures are the mean ± standard deviation over 10
seeded runs. To regenerate them, run
`cargo test --release --test benchmarks -- --ignored --nocapture` and paste the output here.

**Explosion** of a 1000 kg rocket body:

| | Kessler | ASI | CNSA | DLR | ESA | NASA | CiELO |
|---|---|---|---|---|---|---|---|
| L<sub>c</sub> > 1 mm | 378,572 ± 2 | 378,581 | 37,865 | 1,217,054 | 324,886 | 434,928 | 378,568 |
| L<sub>c</sub> > 1 cm | 9,527 ± 102 | 9,403 | 960 | 11,724 | 8,159 | 10,731 | 10,312 |
| L<sub>c</sub> > 10 cm | 234 ± 15 | 234 | 32 | 230 | 206 | 248 | 253 |
| L<sub>c</sub> > 1 m | 4 ± 1 | 7 | 9 | 0 | 6 | 8 | 1 |
| Mass > 1 g | 2,667 ± 47 | 2,472 | 254 | 25,844 | 2,093 | 2,525 | – |
| Area > 1 cm² | 5,882 ± 73 | 5,878 | – | 31,124 | 5,024 | 6,416 | – |
| Δv > 100 m/s | 114,164 ± 249 | 112,932 | 11,380 | 31,124 | 98,717 | 132,032 | – |

**Catastrophic collision** of a 1000 kg rocket body with a 10 kg projectile at 10 km/s:

| | Kessler | ASI | CNSA | ESA | NASA | CiELO |
|---|---|---|---|---|---|---|
| L<sub>c</sub> > 1 mm | 2,416,789 ± 2 | 2,416,795 | 2,416,790 | 4,723,391 | 2,957,159 | 2,416,783 |
| L<sub>c</sub> > 10 cm | 904 ± 30 | 850 | 935 | 1,539 | 862 | 1,017 |

### Sources

- N. L. Johnson et al., *NASA's new breakup model of EVOLVE 4.0*, Advances in Space Research 28(9), 2001: the model.
- P. H. Krisko, *Proper Implementation of the 1998 NASA Breakup Model*, Orbital Debris Quarterly News 15(4), 2011:
  mass conservation and the corrected non-catastrophic collision mass.
- F. Letizia, *Space debris cloud evolution in Low Earth Orbit*, PhD thesis, University of Southampton, 2016
  (appendix A.1): the 1.3 × impact speed cap on collision Δv, and the inter-agency comparison the tests check against.
- R. Humphreys, *Orbital Debris Cloud Evolution: An Analysis of Fragmentation Events in Low Earth Orbit*, Honors
  thesis, Florida Atlantic University, 2021 (table 1): the mass, area and Δv rows of that comparison.

The Python bindings live in [`kesspy/`](kesspy) and the WebAssembly bindings in [`kessler-wasm/`](kessler-wasm).

## Contributing

If you find a bug or have a feature request, please open an issue on the Kessler
GitHub repository:

<https://github.com/reeceHumphreys/kessler/issues>

If you'd like to contribute to the project, feel free to fork the repository and
submit a pull request.

## License

Kessler is distributed under the terms of the MIT license. See the LICENSE file
for details.

## TODO

- [x] Add tests
- [ ] Add documentation
- [x] Validate fragment counts against the inter-agency comparison (Rossi et al., 2006)
- [ ] Validate the A/M, area, mass and Δv distributions
