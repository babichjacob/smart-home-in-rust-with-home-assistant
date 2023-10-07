use std::time::Duration;

use pyo3::prelude::*;
use tokio::time::interval;

async fn real_main() -> ! {
    let duration = Duration::from_millis(400);
    let mut interval = interval(duration);

    loop {
        let instant = interval.tick().await;

        println!("it is now {instant:?}");
    }
}

#[pyfunction]
fn main(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async { Ok(real_main().await) })
}

/// A Python module implemented in Rust.
#[pymodule]
fn smart_home_in_rust_with_home_assistant(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(main, m)?)?;
    Ok(())
}
