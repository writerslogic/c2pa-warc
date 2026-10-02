use crate::error::Error;
use crate::record::{build_record_with_version, parse_records};

/// Append a C2PA Manifest Store to a WARC archive as a new record.
///
/// This is append-only: no existing record, manifest or otherwise, is removed
/// or rewritten. WARC's archival/forensic invariant is that previously
/// written bytes are never deleted or shifted -- doing so retroactively
/// invalidates any external CDX/index built against prior byte offsets, and
/// conflicts with the established C2PA pattern (used for e.g. PDF incremental
/// updates) of appending new content and letting "the last one wins" govern
/// discovery, rather than discarding earlier content. If the archive already
/// carries one or more manifest records, they remain; [`crate::read_manifest`]
/// resolves which one is active by taking the last, positionally, exactly as
/// this function places the newest one.
pub fn append_manifest(
    warc_data: &[u8],
    manifest_bytes: &[u8],
    record_id: &str,
) -> Result<Vec<u8>, Error> {
    if warc_data.is_empty() {
        return Err(Error::InvalidRecord("empty WARC data".into()));
    }
    // Validates the archive parses before trusting its length as a starting
    // offset; the records themselves are never rewritten. The appended
    // manifest record declares the same WARC version as the archive's first
    // record (WARC/1.0 or WARC/1.1), so a legal-deposit archive captured
    // under the older version doesn't end up with a mismatched version line
    // on its newest record.
    let records = parse_records(warc_data)?;
    let version = records
        .first()
        .map(|r| r.version.as_str())
        .unwrap_or(crate::record::VERSION);
    let mut out = Vec::with_capacity(warc_data.len() + manifest_bytes.len());
    out.extend_from_slice(warc_data);
    let record = build_record_with_version(
        version,
        "c2pa-provenance",
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

    /// Appending to an archive captured under WARC/1.0 (ISO 28500:2009, the
    /// version still used by many legal-deposit and national-library
    /// collections) must not silently upgrade the file to WARC/1.1 on its
    /// newest record.
    #[test]
    fn appending_to_a_warc_1_0_archive_matches_its_version() {
        let r1 = crate::record::build_record_with_version(
            "WARC/1.0",
            "response",
            "text/html",
            "urn:uuid:aaa",
            Some("https://example.com/"),
            b"<html></html>",
        );
        let warc = append_manifest(&r1, b"\xCA\xFE", "urn:uuid:m1").unwrap();
        let records = crate::record::parse_records(&warc).unwrap();
        assert_eq!(records[0].version, "WARC/1.0");
        assert_eq!(
            records.last().unwrap().version,
            "WARC/1.0",
            "the appended manifest record must match the archive's own version"
        );
    }

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
    fn appending_a_new_manifest_keeps_the_old_one_and_activates_the_new_one() {
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

        // Append-only: the original record and the superseded manifest
        // record both survive, byte-for-byte, at their original offsets.
        let records = crate::record::parse_records(&warc).unwrap();
        let manifests: Vec<_> = records.iter().filter(|r| r.is_c2pa_manifest()).collect();
        assert_eq!(manifests.len(), 2, "no record was deleted");
        assert_eq!(manifests[0].body, old_manifest);
        assert_eq!(manifests[1].body, new_manifest);
        assert_eq!(records[0].raw_offset, 0, "r1's offset did not shift");
    }
}
