// Generates a minimal WARC that carries a C2PA Manifest Store, for the WARC
// community review at iipc/warc-specifications#120.
//
// The file contains a warcinfo record, one captured `response` record, and a
// trailing `c2paprovenance` record whose block is a real C2PA Manifest Store
// (JUMBF). Run with `cargo run --example make_sample_warc`; it writes
// `examples/sample.warc` and the extracted `examples/sample.manifest.c2pa`.

use c2pa_warc::{append_manifest, build_record, read_manifest};

fn main() {
    let manifest = include_bytes!("manifest.c2pa");

    let warcinfo = build_record(
        "warcinfo",
        "application/warc-fields",
        "urn:uuid:00000000-0000-0000-0000-0000000000a0",
        None,
        b"software: c2pa-warc example\r\nformat: WARC file version 1.1\r\n",
    );

    let http = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<!doctype html><title>Example</title><p>Archived page.</p>";
    let response = build_record(
        "response",
        "application/http;msgtype=response",
        "urn:uuid:00000000-0000-0000-0000-0000000000b0",
        Some("https://example.com/"),
        http,
    );

    let mut base = warcinfo;
    base.extend_from_slice(&response);

    let warc = append_manifest(
        &base,
        manifest,
        "urn:uuid:00000000-0000-0000-0000-0000000000c0",
    )
    .expect("append manifest");

    std::fs::write("examples/sample.warc", &warc).expect("write sample.warc");

    let decoded = read_manifest(&warc).expect("read back manifest");
    std::fs::write("examples/sample.manifest.c2pa", &decoded).expect("write decoded manifest");
    assert_eq!(decoded, manifest, "round-trip must be byte-identical");

    println!(
        "wrote examples/sample.warc ({} bytes) and examples/sample.manifest.c2pa ({} bytes)",
        warc.len(),
        decoded.len()
    );
}
