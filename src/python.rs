//! Python bindings, built with [maturin]/[PyO3] behind the `python` feature and
//! published to PyPI as `c2pa-warc`.
//!
//! WARC archives are binary, so they map to and from Python `bytes`. An archive
//! carrying no manifest returns `None` from
//! [`read_manifest`](fn.read_manifest.html) rather than raising, because absence
//! of provenance is not an error.
//!
//! [maturin]: https://www.maturin.rs/
//! [PyO3]: https://pyo3.rs/

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList};

fn map_err(e: crate::Error) -> PyErr {
    match e.code() {
        Some(code) => PyValueError::new_err(format!("{e} [{code}]")),
        None => PyValueError::new_err(e.to_string()),
    }
}

/// Append a C2PA Manifest Store to a WARC archive as a new record.
#[pyfunction]
fn append_manifest<'py>(
    py: Python<'py>,
    warc: &[u8],
    manifest: &[u8],
    record_id: &str,
) -> PyResult<Bound<'py, PyBytes>> {
    let out = crate::append_manifest(warc, manifest, record_id).map_err(map_err)?;
    Ok(PyBytes::new(py, &out))
}

/// Read the embedded C2PA Manifest Store, or `None` when the archive carries
/// no provenance.
#[pyfunction]
fn read_manifest<'py>(py: Python<'py>, warc: &[u8]) -> PyResult<Option<Bound<'py, PyBytes>>> {
    match crate::read_manifest(warc) {
        Ok(store) => Ok(Some(PyBytes::new(py, &store))),
        Err(crate::Error::NotFound) => Ok(None),
        Err(e) => Err(map_err(e)),
    }
}

/// Every record in the archive, as a list of dicts with `headers`, `body`,
/// `offset`, and `length`.
#[pyfunction]
fn read_records<'py>(py: Python<'py>, warc: &[u8]) -> PyResult<Bound<'py, PyList>> {
    let records = crate::read_records(warc).map_err(map_err)?;
    let out = PyList::empty(py);
    for r in records {
        let d = PyDict::new(py);
        let headers = PyDict::new(py);
        for (k, v) in &r.headers {
            headers.set_item(k.as_str(), v.as_str())?;
        }
        d.set_item("version", r.version.as_str())?;
        d.set_item("headers", headers)?;
        d.set_item("body", PyBytes::new(py, &r.body))?;
        d.set_item("offset", r.raw_offset)?;
        d.set_item("length", r.raw_length)?;
        out.append(d)?;
    }
    Ok(out)
}

/// Build a single WARC record.
#[pyfunction]
#[pyo3(signature = (warc_type, content_type, record_id, body, target_uri = None))]
fn build_record<'py>(
    py: Python<'py>,
    warc_type: &str,
    content_type: &str,
    record_id: &str,
    body: &[u8],
    target_uri: Option<&str>,
) -> PyResult<Bound<'py, PyBytes>> {
    let out = crate::build_record(warc_type, content_type, record_id, target_uri, body);
    Ok(PyBytes::new(py, &out))
}

#[pymodule]
fn c2pa_warc(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(append_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(read_manifest, m)?)?;
    m.add_function(wrap_pyfunction!(read_records, m)?)?;
    m.add_function(wrap_pyfunction!(build_record, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
