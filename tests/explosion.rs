// Import the full local kessler crate
use kessler::{run, ExplosionEvent, SatKind, Satellite};

#[test]
fn explosion_nimbus() {
    // Define the initial position, velocity, mass, characteristic length, and kind of a satellite.
    let position = vec![0.0, 0.0, 0.0]; // position vector [m] relative to Earth's center
    let velocity = vec![0.0, 0.0, 0.0]; // velocity vector [m/s]
    let mass = 849.0; // kg
    let characteristic_length = 0.1; // m
    let sat_kind = SatKind::Rb; // kind of satellite (Rb, Debris, or Satellite)

    // Create a new satellite with the given parameters
    let satellite = Satellite::new(position, velocity, mass, sat_kind);

    // Create a new explosion event with the satellite
    let event = ExplosionEvent::new(satellite, characteristic_length);

    // Run the simulation with the explosion event
    let result = run(&event);

    // Print the final state of the satellite after the simulation has completed
    println!("{:#?}", result.dim());
}
