import numpy as np
from kesspy import SatKind, collision, explosion

# All inputs and outputs are SI: metres, kilograms, metres per second.
debris = explosion(839.0, SatKind.RocketBody, 0.1, seed=42)

print("Fragments:", len(debris.mass))
print("Average characteristic length [m]:", np.mean(debris.characteristic_length))
print("Average A/M [m^2/kg]:", np.mean(debris.am_ratio))
print("Average area [m^2]:", np.mean(debris.area))
print("Average mass [kg]:", np.mean(debris.mass))
print("Average delta-v [m/s]:", np.mean(np.linalg.norm(debris.delta_v, axis=1)))

# A 5 kg object hitting a 500 kg spacecraft at 9.5 km/s.
debris = collision(500.0, SatKind.Spacecraft, 5.0, SatKind.Spacecraft, 9_500.0, 0.1)
print("Collision fragments:", len(debris.mass), "delta_v shape:", debris.delta_v.shape)
