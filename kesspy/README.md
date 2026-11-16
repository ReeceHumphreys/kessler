# KessPy

- [KessPy](#kesspy)
  - [Installation](#installation)
  - [Getting Started](#getting-started)
  - [Result Data Format](#result-data-format)
  - [Contributing](#contributing)
  - [License](#license)
  - [TODO](#todo)

KessPy is a Python library that uses the NASA Standard Breakup Model to simulate explosion and collision events in orbit, which is important for managing space debris and preventing collisions with other orbiting objects. It is a Python wrapper around Kessler.rs, a Rust implementation of the NASA Standard Breakup Model, which provides significant performance gains over purely Python-based implementations. More information about Kessler.rs can be found on its GitHub page at <https://github.com/reecehumphreys/kessler>.

**IMPORTANT**: Until this package reaches version 1.0, the API is subject to change and the accuracy of results cannot be guaranteed.

## Installation

KessPy needs Python 3.10 or later. Install it with pip, which also installs numpy:

```shell
pip install kesspy
```

## Getting Started

All inputs and outputs are SI (metres, kilograms, metres per second).

```python
import numpy as np
from kesspy import SatKind, collision, explosion

# An 839 kg rocket body exploding, modelling fragments down to 10 cm.
debris = explosion(839.0, SatKind.RocketBody, 0.1, seed=42)
print(f"{len(debris.mass)} fragments, mean mass {np.mean(debris.mass)} kg")

# A 5 kg object hitting a 500 kg spacecraft at 9.5 km/s.
debris = collision(500.0, SatKind.Spacecraft, 5.0, SatKind.Spacecraft, 9_500.0, 0.1)
```

Leave out `seed` for a different result each run. A given seed gives the same fragments on every platform, and the
same as the Rust and JavaScript packages. Input outside the model's range (a non-positive mass or speed, or a minimum
size outside 1 mm to 1 m) raises `ValueError`.

## Result Data Format

Both functions return a `Fragments` object whose fields are numpy `float32` arrays with one row per fragment:

| field                   | unit  | shape  |
| ----------------------- | ----- | ------ |
| `characteristic_length` | m     | (n,)   |
| `am_ratio`              | m²/kg | (n,)   |
| `area`                  | m²    | (n,)   |
| `mass`                  | kg    | (n,)   |
| `delta_v`               | m/s   | (n, 3) |

Fragments start where the event happened; add `delta_v` to the parent's velocity to get each fragment's velocity.

## Contributing

If you find a bug or have a feature request, please open an issue on the KessPy
GitHub repository:

<https://github.com/ReeceHumphreys/kessler/issues>

If you'd like to contribute to the project, feel free to fork the repository and
submit a pull request. To build and test it locally (needs [Rust](https://rustup.rs) and [uv](https://docs.astral.sh/uv/)):

```shell
cd kesspy
uv venv && uv pip install -e . pytest
.venv/bin/pytest
```

## License

Kessler is distributed under the terms of the MIT license. See the LICENSE file
for details.

## TODO

- [x] Update the documentation to reflect the new API
- [x] Add tests for the new API
