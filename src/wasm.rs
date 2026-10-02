//! WebAssembly bindings, built only for the `wasm32` target and published to
//! npm as `c2pa-warc`.
//!
//! Archives map to and from `Uint8Array`. An archive carrying no manifest
//! returns `null` from [`readManifest`](fn.read_manifest.html) rather than
//! throwing, because absence of provenance is not an error.

use wasm_bindgen::prelude::*;

fn js_err(e: crate::Error) -> JsError {
    match e.code() {
        Some(code) => JsError::new(&format!("{e} [{code}]")),
        None => JsError::new(&e.to_string()),
    }
}

/// Append a C2PA Manifest Store to a WARC archive as a new record.
#[wasm_bindgen(js_name = appendManifest)]
pub fn append_manifest(warc: &[u8], manifest: &[u8], record_id: &str) -> Result<Vec<u8>, JsError> {
    crate::append_manifest(warc, manifest, record_id).map_err(js_err)
}

/// Read the embedded C2PA Manifest Store, or `null` when the archive carries no
/// provenance.
#[wasm_bindgen(js_name = readManifest)]
pub fn read_manifest(warc: &[u8]) -> Result<Option<Vec<u8>>, JsError> {
    match crate::read_manifest(warc) {
        Ok(store) => Ok(Some(store)),
        Err(crate::Error::NotFound) => Ok(None),
        Err(e) => Err(js_err(e)),
    }
}

/// Every record in the archive, as objects with `headers`, `body`, `offset`,
/// and `length`.
#[wasm_bindgen(js_name = readRecords)]
pub fn read_records(warc: &[u8]) -> Result<Vec<JsValue>, JsError> {
    let records = crate::read_records(warc).map_err(js_err)?;
    Ok(records
        .into_iter()
        .map(|r| {
            let out = js_sys::Object::new();
            let headers = js_sys::Object::new();
            for (k, v) in &r.headers {
                let _ = js_sys::Reflect::set(&headers, &k.as_str().into(), &v.as_str().into());
            }
            let _ = js_sys::Reflect::set(&out, &"version".into(), &r.version.as_str().into());
            let _ = js_sys::Reflect::set(&out, &"headers".into(), &headers);
            let _ = js_sys::Reflect::set(
                &out,
                &"body".into(),
                &js_sys::Uint8Array::from(&r.body[..]).into(),
            );
            let _ = js_sys::Reflect::set(&out, &"offset".into(), &(r.raw_offset as u32).into());
            let _ = js_sys::Reflect::set(&out, &"length".into(), &(r.raw_length as u32).into());
            out.into()
        })
        .collect())
}

/// Build a single WARC record.
#[wasm_bindgen(js_name = buildRecord)]
pub fn build_record(
    warc_type: &str,
    content_type: &str,
    record_id: &str,
    body: &[u8],
    target_uri: Option<String>,
) -> Vec<u8> {
    crate::build_record(
        warc_type,
        content_type,
        record_id,
        target_uri.as_deref(),
        body,
    )
}
