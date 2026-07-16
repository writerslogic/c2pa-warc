use std::collections::HashMap;

use crate::error::Error;

const VERSION: &str = "WARC/1.1";
const C2PA_CONTENT_TYPE: &str = "application/c2pa";
const C2PA_WARC_TYPE: &str = "c2paprovenance";

#[derive(Debug, Clone)]
pub struct WarcRecord {
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
    pub raw_offset: usize,
    pub raw_length: usize,
}

impl WarcRecord {
    pub fn warc_type(&self) -> Option<&str> {
        self.headers.get("warc-type").map(|s| s.as_str())
    }

    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }

    pub fn record_id(&self) -> Option<&str> {
        self.headers.get("warc-record-id").map(|s| s.as_str())
    }

    pub fn is_c2pa_manifest(&self) -> bool {
        self.warc_type() == Some(C2PA_WARC_TYPE) && self.content_type() == Some(C2PA_CONTENT_TYPE)
    }
}

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

        if pos + content_length > data.len() {
            return Err(Error::InvalidRecord("body extends past end of data".into()));
        }

        let body = data[pos..pos + content_length].to_vec();
        pos += content_length;

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

// The C2PA manifest record carries no WARC-Target-URI; pass `None` for it. Other
// record types (response, resource) supply their captured URI via `Some`.
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

    #[test]
    fn parse_c2pa_record() {
        let manifest = b"\x00\x01\x02\x03";
        let record = build_record(
            "c2paprovenance",
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
