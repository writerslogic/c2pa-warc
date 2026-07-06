use crate::error::Error;
use crate::record::build_record;

pub fn append_manifest(
    warc_data: &[u8],
    manifest_bytes: &[u8],
    record_id: &str,
) -> Result<Vec<u8>, Error> {
    if warc_data.is_empty() {
        return Err(Error::InvalidRecord("empty WARC data".into()));
    }
    let record = build_record(
        "resource",
        "application/c2pa",
        record_id,
        "urn:c2pa:manifest",
        manifest_bytes,
    );
    let mut out = warc_data.to_vec();
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
            "https://example.com/",
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
            "https://example.com/",
            b"<html></html>",
        );
        let warc = append_manifest(&r1, b"\x00\x01", "urn:uuid:m1").unwrap();
        let records = crate::record::parse_records(&warc).unwrap();
        let manifest = records.last().unwrap();
        assert!(manifest.is_c2pa_manifest());
        assert_eq!(
            manifest.headers.get("warc-target-uri").map(String::as_str),
            Some("urn:c2pa:manifest")
        );
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
            "https://example.com/",
            b"<html></html>",
        );
        let old_manifest = b"old";
        let new_manifest = b"new";

        let warc = append_manifest(&r1, old_manifest, "urn:uuid:m1").unwrap();
        let warc = append_manifest(&warc, new_manifest, "urn:uuid:m2").unwrap();
        let extracted = read_manifest(&warc).unwrap();
        assert_eq!(extracted, b"new");
    }
}
