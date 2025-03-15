use pyo3::IntoPyObject;
use snafu::Snafu;

#[derive(Debug, Clone, derive_more::Into, IntoPyObject)]
pub struct FiniteF64(f64);

#[derive(Debug, Snafu)]
#[snafu(display("{value:?} is not finite"))]
pub struct NotFinite {
    value: f64,
}

impl TryFrom<f64> for FiniteF64 {
    type Error = NotFinite;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(NotFinite { value })
        }
    }
}
