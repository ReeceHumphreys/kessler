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
kessler = "0.1.4"
```

## Usage

The whole interface is two functions, `explosion` and `collision`. All inputs and outputs are SI (metres, kilograms,
metres per second). Randomness comes from the generator you pass in, so a seeded one gives reproducible results.

```rust
use kessler::{collision, explosion, SatKind};
use rand::rngs::StdRng;
use rand::SeedableRng;

fn main() {
    let mut rng = StdRng::seed_from_u64(42);

    // An 839 kg rocket body exploding, modelling fragments down to 10 cm.
    let debris = explosion(839.0, SatKind::RocketBody, 0.1, &mut rng);
    println!("{} fragments, first has mass {} kg", debris.mass.len(), debris.mass[0]);

    // A 5 kg object hitting a 500 kg spacecraft at 9.5 km/s.
    let debris = collision(500.0, SatKind::Spacecraft, 5.0, SatKind::Debris, 9_500.0, 0.1, &mut rng);
    println!("{} fragments", debris.mass.len());
}
```

`Fragments` holds one entry per fragment in each of `characteristic_length` [m], `am_ratio` [m²/kg], `area` [m²],
`mass` [kg] and `delta_v` [m/s, 3D]. Fragments start where the event happened; add `delta_v` to the parent's velocity
to get each fragment's velocity.

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

- [ ] Add tests
- [ ] Add documentation
- [ ] Validate results
