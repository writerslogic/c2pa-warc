use std::collections::HashMap;

use crate::error::Error;

const VERSION: &str = "WARC/1.1";
const C2PA_CONTENT_TYPE: &str = "application/c2pa";
const C2PA_WARC_TYPE: &str = "c2pa-provenance";

/// A parsed WARC record: its headers, its body, and where it sits in the file.
#[derive(Debug, Clone)]
pub struct WarcRecord {
    /// Record headers, with names lowercased for case-insensitive lookup.
    pub headers: HashMap<String, String>,
    /// The record body, exactly as stored.
    pub body: Vec<u8>,
    /// Byte offset of the record within the archive.
    pub raw_offset: usize,
    /// Byte length of the whole record, headers included.
    pub raw_length: usize,
}

impl WarcRecord {
    /// The `WARC-Type` header, if present.
    pub fn warc_type(&self) -> Option<&str> {
        self.headers.get("warc-type").map(|s| s.as_str())
    }

    /// The `Content-Type` header, if present.
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }

    /// The `WARC-Record-ID` header, if present.
    pub fn record_id(&self) -> Option<&str> {
        self.headers.get("warc-record-id").map(|s| s.as_str())
    }

    /// Whether this is the C2PA manifest record: the dedicated WARC type with
    /// the `application/c2pa` content type.
    pub fn is_c2pa_manifest(&self) -> bool {
        self.warc_type() == Some(C2PA_WARC_TYPE) && self.content_type() == Some(C2PA_CONTENT_TYPE)
    }
}

/// Parse every record in the archive, in file order.
pub fn parse_records(data: &[u8]) -> Result<Vec<WarcRecord>, Error> {
    let mut records = Vec::new();
    let mut pos = 0;

    while pos < data.len() {
        while pos < data.len() && (data[pos] == b'\r' || data[pos] == b'\n') {
            pos += 1;
        }
        if pos >= data.len() {
            break;
        }

        let record_start = pos;

        if !data[pos..].starts_with(VERSION.as_bytes()) {
            return Err(Error::InvalidRecord(format!(
                "expected WARC/1.1 at offset {pos}"
            )));
        }

        let header_end = find_double_crlf(&data[pos..])
            .ok_or_else(|| Error::InvalidRecord("unterminated header".into()))?;
        let header_block = &data[pos..pos + header_end];
        pos += header_end + 4; // skip \r\n\r\n

        let headers = parse_headers(header_block)?;

        let content_length: usize = headers
            .get("content-length")
            .ok_or_else(|| Error::InvalidRecord("missing Content-Length".into()))?
            .parse()
            .map_err(|_| Error::InvalidRecord("invalid Content-Length".into()))?;

        // Content-Length is attacker-controlled; a value like usize::MAX makes
        // `pos + content_length` overflow rather than legitimately exceed
        // data.len(), which panics on overflow-checked builds instead of
        // reaching the bounds error below.
        let end = pos
            .checked_add(content_length)
            .ok_or_else(|| Error::InvalidRecord("Content-Length overflows".into()))?;
        if end > data.len() {
            return Err(Error::InvalidRecord("body extends past end of data".into()));
        }

        let body = data[pos..end].to_vec();
        pos = end;

        // Skip record terminator \r\n\r\n
        if data[pos..].starts_with(b"\r\n\r\n") {
            pos += 4;
        } else if data[pos..].starts_with(b"\n\n") {
            pos += 2;
        }

        let raw_length = pos - record_start;

        records.push(WarcRecord {
            headers,
            body,
            raw_offset: record_start,
            raw_length,
        });
    }

    Ok(records)
}

/// Build a single WARC record from its headers and body.
///
/// The C2PA manifest record carries no WARC-Target-URI; pass `None` for it. Other
/// record types (response, resource) supply their captured URI via `Some`.
pub fn build_record(
    warc_type: &str,
    content_type: &str,
    record_id: &str,
    target_uri: Option<&str>,
    body: &[u8],
) -> Vec<u8> {
    let date = warc_date_now();
    let target_line = match target_uri {
        Some(uri) => format!("WARC-Target-URI: {uri}\r\n"),
        None => String::new(),
    };
    let header = format!(
        "{VERSION}\r\nWARC-Type: {warc_type}\r\nWARC-Record-ID: <{record_id}>\r\n{target_line}WARC-Date: {date}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n\r\n",
        body.len()
    );
    let mut out = header.into_bytes();
    out.extend_from_slice(body);
    out.extend_from_slice(b"\r\n\r\n");
    out
}

fn warc_date_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_warc_date(secs)
}

// civil-from-days conversion; valid for all dates in the Unix era
fn format_warc_date(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3_600, (rem % 3_600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(month <= 2);
    format!("{y:04}-{month:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn find_double_crlf(data: &[u8]) -> Option<usize> {
    data.windows(4).position(|w| w == b"\r\n\r\n")
}

fn parse_headers(block: &[u8]) -> Result<HashMap<String, String>, Error> {
    let text =
        std::str::from_utf8(block).map_err(|_| Error::InvalidRecord("non-UTF-8 header".into()))?;
    let mut headers = HashMap::new();

    for line in text.lines().skip(1) {
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
        }
    }

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_record() {
        let raw = b"WARC/1.1\r\nWARC-Type: resource\r\nWARC-Record-ID: <urn:uuid:abc>\r\nContent-Type: text/plain\r\nContent-Length: 5\r\n\r\nhello\r\n\r\n";
        let records = parse_records(raw).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].body, b"hello");
        assert_eq!(records[0].warc_type(), Some("resource"));
    }

    /// An attacker-controlled Content-Length near usize::MAX must error
    /// cleanly, not overflow the `pos + content_length` bounds check and
    /// panic on an overflow-checked build.
    #[test]
    fn huge_content_length_errors_instead_of_overflowing() {
        let raw = b"WARC/1.1\r\nWARC-Type: resource\r\nWARC-Record-ID: <urn:uuid:abc>\r\nContent-Type: text/plain\r\nContent-Length: 18446744073709551615\r\n\r\nhello\r\n\r\n";
        assert!(parse_records(raw).is_err());
    }

    #[test]
    fn parse_c2pa_record() {
        let manifest = b"\x00\x01\x02\x03";
        let record = build_record(
            "c2pa-provenance",
            "application/c2pa",
            "urn:uuid:test-id",
            None,
            manifest,
        );
        let records = parse_records(&record).unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0].is_c2pa_manifest());
        assert_eq!(records[0].body, manifest);
        // The manifest record carries no WARC-Target-URI.
        assert_eq!(records[0].headers.get("warc-target-uri"), None);
    }

    #[test]
    fn format_warc_date_known_timestamp() {
        assert_eq!(format_warc_date(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_warc_date(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn build_and_parse_roundtrip() {
        let body = b"test body content";
        let record = build_record(
            "resource",
            "text/plain",
            "urn:uuid:123",
            Some("https://example.com/"),
            body,
        );
        let records = parse_records(&record).unwrap();
        assert_eq!(records[0].body, body);
    }
}
