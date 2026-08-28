use crate::error::Error;
use crate::record::{parse_records, WarcRecord};

/// Read the archive's single embedded C2PA Manifest Store.
///
/// An archive containing more than one manifest record is treated as if no
/// manifest were located, as required by the WARC embedding annex.
pub fn read_manifest(data: &[u8]) -> Result<Vec<u8>, Error> {
    let records = parse_records(data)?;
    let mut manifests = records.iter().filter(|r| r.is_c2pa_manifest());
    let manifest = manifests.next().ok_or(Error::NotFound)?;
    if manifests.next().is_some() {
        return Err(Error::NotFound);
    }
    Ok(manifest.body.clone())
}

/// Parse every record in the archive, in file order.
pub fn read_records(data: &[u8]) -> Result<Vec<WarcRecord>, Error> {
    parse_records(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::build_record;

    #[test]
    fn read_manifest_from_warc() {
        let r1 = build_record(
            "response",
            "text/html",
            "urn:uuid:aaa",
            Some("https://example.com/"),
            b"<html></html>",
        );
        let manifest_bytes = b"\x00\x01\x02\x03";
        let r2 = build_record(
            "c2pa-provenance",
            "application/c2pa",
            "urn:uuid:bbb",
            None,
            manifest_bytes,
        );
        let mut warc = Vec::new();
        warc.extend_from_slice(&r1);
        warc.extend_from_slice(&r2);

        let result = read_manifest(&warc).unwrap();
        assert_eq!(result, manifest_bytes);
    }

    #[test]
    fn multiple_manifests_are_treated_as_not_located() {
        let r1 = build_record(
            "c2pa-provenance",
            "application/c2pa",
            "urn:uuid:old",
            None,
            b"old",
        );
        let r2 = build_record(
            "response",
            "text/html",
            "urn:uuid:mid",
            Some("https://example.com/"),
            b"<html></html>",
        );
        let r3 = build_record(
            "c2pa-provenance",
            "application/c2pa",
            "urn:uuid:new",
            None,
            b"new",
        );
        let mut warc = Vec::new();
        warc.extend_from_slice(&r1);
        warc.extend_from_slice(&r2);
        warc.extend_from_slice(&r3);

        assert!(matches!(read_manifest(&warc), Err(Error::NotFound)));
    }

    #[test]
    fn no_manifest() {
        let r1 = build_record(
            "response",
            "text/html",
            "urn:uuid:aaa",
            Some("https://example.com/"),
            b"<html></html>",
        );
        assert!(matches!(read_manifest(&r1), Err(Error::NotFound)));
    }
}
