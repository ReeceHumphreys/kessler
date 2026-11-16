import numpy as np
import pytest
from kesspy import SatKind, collision, explosion


def test_seed_reproduces_fragments():
    a = explosion(839.0, SatKind.RocketBody, 0.1, seed=1)
    b = explosion(839.0, SatKind.RocketBody, 0.1, seed=1)
    assert np.array_equal(a.mass, b.mass)
    assert np.array_equal(a.delta_v, b.delta_v)


def test_fragments_carry_the_input_mass():
    debris = collision(500.0, SatKind.Spacecraft, 5.0, SatKind.Spacecraft, 9_500.0, 0.1, seed=2)
    assert debris.mass.sum() == pytest.approx(505.0, rel=1e-3)
    assert debris.delta_v.shape == (len(debris.mass), 3)
    assert debris.mass.dtype == np.float32


def test_invalid_input_raises_value_error():
    with pytest.raises(ValueError, match="min_characteristic_length"):
        explosion(1000.0, SatKind.RocketBody, 0.0)
