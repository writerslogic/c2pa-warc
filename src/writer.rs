use crate::error::Error;
use crate::record::{build_record, parse_records};

// A WARC file carries at most one manifest record, always last. Updating removes
// any existing manifest record before appending the replacement, so the bytes of
// every other record are preserved and the file stays conformant.
pub fn append_manifest(
    warc_data: &[u8],
    manifest_bytes: &[u8],
    record_id: &str,
) -> Result<Vec<u8>, Error> {
    if warc_data.is_empty() {
        return Err(Error::InvalidRecord("empty WARC data".into()));
    }
    let mut out = Vec::with_capacity(warc_data.len() + manifest_bytes.len());
    for r in parse_records(warc_data)? {
        if r.is_c2pa_manifest() {
            continue;
        }
        out.extend_from_slice(&warc_data[r.raw_offset..r.raw_offset + r.raw_length]);
    }
    let record = build_record(
        "c2paprovenance",
        "application/c2pa",
        record_id,
        None,
        manifest_bytes,
    );
    out.extend_from_slice(&record);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reader::read_manifest;
    use crate::record::build_record;

    #[test]
    fn append_and_read_roundtrip() {
        let r1 = build_record(
            "response",
            "text/html",
            "urn:uuid:aaa",
            Some("https://example.com/"),
            b"<html></html>",
        );
        let manifest = b"\xCA\xFE\xBA\xBE";

        let warc = append_manifest(&r1, manifest, "urn:uuid:manifest-1").unwrap();
        let extracted = read_manifest(&warc).unwrap();
        assert_eq!(extracted, manifest);
    }

    #[test]
    fn manifest_record_headers() {
        let r1 = build_record(
            "response",
            "text/html",
            "urn:uuid:aaa",
            Some("https://example.com/"),
            b"<html></html>",
        );
        let warc = append_manifest(&r1, b"\x00\x01", "urn:uuid:m1").unwrap();
        let records = crate::record::parse_records(&warc).unwrap();
        let manifest = records.last().unwrap();
        assert!(manifest.is_c2pa_manifest());
        assert_eq!(manifest.headers.get("warc-target-uri"), None);
        assert_eq!(
            manifest.headers.get("warc-record-id").map(String::as_str),
            Some("<urn:uuid:m1>")
        );
    }

    #[test]
    fn append_replaces_active() {
        let r1 = build_record(
            "response",
            "text/html",
            "urn:uuid:aaa",
            Some("https://example.com/"),
            b"<html></html>",
        );
        let old_manifest = b"old";
        let new_manifest = b"new";

        let warc = append_manifest(&r1, old_manifest, "urn:uuid:m1").unwrap();
        let warc = append_manifest(&warc, new_manifest, "urn:uuid:m2").unwrap();
        let extracted = read_manifest(&warc).unwrap();
        assert_eq!(extracted, b"new");

        // Exactly one manifest record survives the update.
        let manifests = crate::record::parse_records(&warc)
            .unwrap()
            .into_iter()
            .filter(|r| r.is_c2pa_manifest())
            .count();
        assert_eq!(manifests, 1);
    }
}
