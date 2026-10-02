use crate::error::Error;
use crate::record::{parse_records, WarcRecord};

/// Read the archive's single embedded C2PA Manifest Store.
///
/// The C2PA Technical Specification defines no WARC embedding method (see the
/// crate README); this is this crate's own convention, not a cited
/// requirement. [`crate::append_manifest`] is append-only and never removes an
/// earlier manifest record, so an archive may legitimately carry more than
/// one: the one that is positionally *last* in the file is active, the same
/// "last one wins" rule the C2PA spec applies elsewhere (e.g. the last C2PA
/// Manifest superbox in a BMFF asset).
pub fn read_manifest(data: &[u8]) -> Result<Vec<u8>, Error> {
    let records = parse_records(data)?;
    records
        .iter()
        .rev()
        .find(|r| r.is_c2pa_manifest())
        .map(|r| r.body.clone())
        .ok_or(Error::NotFound)
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
    fn the_last_manifest_record_wins() {
        // append_manifest is append-only and never removes an earlier
        // manifest record, so more than one may legitimately be present; the
        // positionally last one is active, matching the C2PA "last one wins"
        // pattern used elsewhere in the spec.
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

        assert_eq!(read_manifest(&warc).unwrap(), b"new");
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
