use crate::error::Error;
use crate::record::build_record;

pub fn append_manifest(warc_data: &[u8], manifest_bytes: &[u8], record_id: &str) -> Result<Vec<u8>, Error> {
    if warc_data.is_empty() {
        return Err(Error::InvalidRecord("empty WARC data".into()));
    }
    let record = build_record("resource", "application/c2pa", record_id, manifest_bytes);
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
        let r1 = build_record("response", "text/html", "urn:uuid:aaa", b"<html></html>");
        let manifest = b"\xCA\xFE\xBA\xBE";

        let warc = append_manifest(&r1, manifest, "urn:uuid:manifest-1").unwrap();
        let extracted = read_manifest(&warc).unwrap();
        assert_eq!(extracted, manifest);
    }

    #[test]
    fn append_replaces_active() {
        let r1 = build_record("response", "text/html", "urn:uuid:aaa", b"<html></html>");
        let old_manifest = b"old";
        let new_manifest = b"new";

        let warc = append_manifest(&r1, old_manifest, "urn:uuid:m1").unwrap();
        let warc = append_manifest(&warc, new_manifest, "urn:uuid:m2").unwrap();
        let extracted = read_manifest(&warc).unwrap();
        assert_eq!(extracted, b"new");
    }
}
