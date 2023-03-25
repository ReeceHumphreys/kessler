# Kessler.rs

Kessler is a Rust package that allows users to simulate fragmentation events
in low earth orbit according to the NASA standard breakup model.

## Installation

To use Kessler, you'll need to have Rust and Cargo installed on your system.
You can install Rust and Cargo by following the instructions on the official
Rust website:

<https://www.rust-lang.org/tools/install>

Once you have Rust and Cargo installed, you can add Kessler to your project by
adding the following line to your `Cargo.toml` file:

```toml
[dependencies]
kessler = "0.1.0"
```

## Usage

To use Kessler, first import the library in your Rust code:

```rust
use kessler::{run, ExplosionEvent, SatKind, Satellite};

fn main() {

    // Define the initial position, velocity, mass, characteristic length, and kind of a satellite.
    let position = [6.702e6, 0.0, 0.0]; // position vector [m] relative to Earth's center
    let velocity = [0.0, 7.666e3, 0.0]; // velocity vector [m/s]
    let mass = 4.98e3; // kg
    let characteristic_length = 0.1; // m
    let sat_kind = SatKind::Rb; // kind of satellite (Rb, Debris, or Satellite)

    // Create a new satellite with the given parameters
    let satellite = Satellite::new(position, velocity, mass, characteristic_length, sat_kind);

    // Create a new explosion event with the satellite
    let event = ExplosionEvent::new(satellite);

    // Run the simulation with the explosion event
    let result = run(&event);

    // Print the final state of the satellite after the simulation has completed
    println!("{:#?}", result.dim());
}
```

## Contributing

If you find a bug or have a feature request, please open an issue on the Kessler
GitHub repository:

<https://github.com/reeceHumphreys/kessler/issues>

If you'd like to contribute to the project, feel free to fork the repository and
submit a pull request.

## License

Kessler is distributed under the terms of the MIT license. See the LICENSE file
for details.
