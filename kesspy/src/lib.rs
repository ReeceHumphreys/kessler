use numpy::{PyArray1, PyArray2};
use pyo3::prelude::*;
use rand::rngs::StdRng;
use rand::SeedableRng;

#[pyclass(name = "SatKind")]
#[derive(Clone, Copy)]
pub enum PySatKind {
    RocketBody,
    Spacecraft,
    Debris,
}

impl From<PySatKind> for kessler::SatKind {
    fn from(kind: PySatKind) -> Self {
        match kind {
            PySatKind::RocketBody => kessler::SatKind::RocketBody,
            PySatKind::Spacecraft => kessler::SatKind::Spacecraft,
            PySatKind::Debris => kessler::SatKind::Debris,
        }
    }
}

/// The fragments from one event, as numpy arrays with one row per fragment. All values are SI.
#[pyclass(name = "Fragments")]
pub struct PyFragments {
    #[pyo3(get)]
    characteristic_length: Py<PyArray1<f32>>,
    #[pyo3(get)]
    am_ratio: Py<PyArray1<f32>>,
    #[pyo3(get)]
    area: Py<PyArray1<f32>>,
    #[pyo3(get)]
    mass: Py<PyArray1<f32>>,
    /// [m/s], shape (n, 3).
    #[pyo3(get)]
    delta_v: Py<PyArray2<f32>>,
}

impl PyFragments {
    fn new(py: Python, f: kessler::Fragments) -> PyResult<Self> {
        let n = f.delta_v.len();
        let delta_v = PyArray1::from_iter(py, f.delta_v.into_iter().flatten()).reshape([n, 3])?;
        Ok(PyFragments {
            characteristic_length: PyArray1::from_vec(py, f.characteristic_length).into(),
            am_ratio: PyArray1::from_vec(py, f.am_ratio).into(),
            area: PyArray1::from_vec(py, f.area).into(),
            mass: PyArray1::from_vec(py, f.mass).into(),
            delta_v: delta_v.into(),
        })
    }
}

fn rng(seed: Option<u64>) -> StdRng {
    seed.map_or_else(StdRng::from_entropy, StdRng::seed_from_u64)
}

/// Breaks up a single object. Inputs and outputs are SI (m, kg, m/s). Pass a `seed` for reproducible results.
#[pyfunction]
#[pyo3(signature = (mass, kind, min_characteristic_length, seed = None))]
fn explosion(
    py: Python,
    mass: f32,
    kind: PySatKind,
    min_characteristic_length: f32,
    seed: Option<u64>,
) -> PyResult<PyFragments> {
    PyFragments::new(py, kessler::explosion(mass, kind.into(), min_characteristic_length, &mut rng(seed)))
}

/// Breaks up two objects colliding at `impact_speed`. Inputs and outputs are SI (m, kg, m/s). Pass a `seed` for
/// reproducible results.
#[pyfunction]
#[pyo3(signature = (mass_a, kind_a, mass_b, kind_b, impact_speed, min_characteristic_length, seed = None))]
#[allow(clippy::too_many_arguments)]
fn collision(
    py: Python,
    mass_a: f32,
    kind_a: PySatKind,
    mass_b: f32,
    kind_b: PySatKind,
    impact_speed: f32,
    min_characteristic_length: f32,
    seed: Option<u64>,
) -> PyResult<PyFragments> {
    let f = kessler::collision(
        mass_a,
        kind_a.into(),
        mass_b,
        kind_b.into(),
        impact_speed,
        min_characteristic_length,
        &mut rng(seed),
    );
    PyFragments::new(py, f)
}

// The name of the module must be the same as the rust package name
#[pymodule]
fn kesspy(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PySatKind>()?;
    m.add_class::<PyFragments>()?;
    m.add_function(wrap_pyfunction!(explosion, m)?)?;
    m.add_function(wrap_pyfunction!(collision, m)?)?;
    Ok(())
}
