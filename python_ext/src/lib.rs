use pyo3::prelude::*;

mod py;
mod llinterpreter;

// name must match the `lib.name` setting in the `Cargo.toml`
#[pymodule]
fn _lib(_py: Python<'_>, m: &Bound<PyModule>) -> PyResult<()> {
    py::init(m)?;
    llinterpreter::init(m)?;
    Ok(())
}
