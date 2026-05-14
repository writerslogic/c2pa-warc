<p align="center">
  <h1 align="center">c2pa-warc</h1>
  <p align="center">C2PA manifest embedding for WARC web archive files (ISO 28500)</p>
</p>

<p align="center">
  <a href="https://crates.io/crates/c2pa-warc"><img src="https://img.shields.io/crates/v/c2pa-warc.svg" alt="crates.io"></a>
  <a href="https://docs.rs/c2pa-warc"><img src="https://docs.rs/c2pa-warc/badge.svg" alt="docs.rs"></a>
  <a href="#license"><img src="https://img.shields.io/crates/l/c2pa-warc.svg" alt="License"></a>
</p>

## Overview

Stores and retrieves C2PA Manifest Stores in [WARC 1.1](https://iipc.github.io/warc-specifications/specifications/warc-format/warc-1.1/) files (ISO 28500). The manifest is stored as a WARC `resource` record with `Content-Type: application/c2pa`, appended at the end of the file.

WARC is used by national libraries, legal deposit systems, and digital preservation institutions to archive web content. This crate enables content provenance for archived web resources.

Zero dependencies.

## Quick Start

```toml
[dependencies]
c2pa-warc = "0.1"
```

### Append a manifest

```rust
use c2pa_warc::append_manifest;

let warc_data: &[u8] = /* existing WARC file bytes */;
let manifest: &[u8] = /* C2PA manifest store bytes */;
let signed = append_manifest(warc_data, manifest, "urn:uuid:12345678-1234-1234-1234-123456789012").unwrap();
```

### Read a manifest

```rust
use c2pa_warc::read_manifest;

let manifest = read_manifest(&warc_data).unwrap();
```

### Multiple manifests

When a WARC file contains multiple C2PA manifest records (e.g., after concatenation), the last record is used as the active manifest. Pre-existing manifest records are preserved, and their hashes become valid again if the file is split back into its original constituents.

## Design

- Manifest stored as a WARC `resource` record
- Content hashing via collection data hash over entire WARC records (headers + body)
- Supports per-record gzip compression (hashes compressed bytes)
- Append-only; no two-pass workflow required
- Multiple manifests allowed; last record is active

## Related Crates

| Crate | Description |
|---|---|
| [c2pa-structured-text](https://crates.io/crates/c2pa-structured-text) | Structured text embedding via ASCII armour delimiters |
| [c2pa-vtt](https://github.com/writerslogic/c2pa-vtt) | WebVTT subtitle embedding |
| [c2pa-text-binding](https://crates.io/crates/c2pa-text-binding) | Soft binding and content fingerprinting for text assets |
| [c2pa-rs](https://crates.io/crates/c2pa) | Official C2PA SDK |

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Built by [WritersLogic](https://writerslogic.com)
