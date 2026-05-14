use crate::error::Error;
use crate::record::{parse_records, WarcRecord};

pub fn read_manifest(data: &[u8]) -> Result<Vec<u8>, Error> {
    let records = parse_records(data)?;
    let manifest = records
        .iter()
        .rev()
        .find(|r| r.is_c2pa_manifest())
        .ok_or(Error::NotFound)?;
    Ok(manifest.body.clone())
}

pub fn read_records(data: &[u8]) -> Result<Vec<WarcRecord>, Error> {
    parse_records(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::build_record;

    #[test]
    fn read_manifest_from_warc() {
        let r1 = build_record("response", "text/html", "urn:uuid:aaa", b"<html></html>");
        let manifest_bytes = b"\x00\x01\x02\x03";
        let r2 = build_record("resource", "application/c2pa", "urn:uuid:bbb", manifest_bytes);
        let mut warc = Vec::new();
        warc.extend_from_slice(&r1);
        warc.extend_from_slice(&r2);

        let result = read_manifest(&warc).unwrap();
        assert_eq!(result, manifest_bytes);
    }

    #[test]
    fn uses_last_manifest() {
        let r1 = build_record("resource", "application/c2pa", "urn:uuid:old", b"old");
        let r2 = build_record("response", "text/html", "urn:uuid:mid", b"<html></html>");
        let r3 = build_record("resource", "application/c2pa", "urn:uuid:new", b"new");
        let mut warc = Vec::new();
        warc.extend_from_slice(&r1);
        warc.extend_from_slice(&r2);
        warc.extend_from_slice(&r3);

        let result = read_manifest(&warc).unwrap();
        assert_eq!(result, b"new");
    }

    #[test]
    fn no_manifest() {
        let r1 = build_record("response", "text/html", "urn:uuid:aaa", b"<html></html>");
        assert!(matches!(read_manifest(&r1), Err(Error::NotFound)));
    }
}
