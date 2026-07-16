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

Stores and retrieves C2PA Manifest Stores in [WARC 1.1](https://iipc.github.io/warc-specifications/specifications/warc-format/warc-1.1/) files (ISO 28500). The manifest is stored as a WARC record of a dedicated `c2paprovenance` type with `Content-Type: application/c2pa`, as the last record in the file.

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

A WARC file carries at most one manifest record, always last. An update removes the existing manifest record and appends the replacement, which leaves the bytes of every other record unchanged. When WARC files are concatenated, the combining tool appends a fresh manifest covering all records in the combined file and removes the constituent manifest records, which may be referenced as ingredients.

## Design

- Manifest stored as a WARC record of a dedicated `c2paprovenance` type, as the last record in the file; it carries no `WARC-Target-URI`
- At most one manifest record; an update removes the existing one and appends the replacement
- Manifest records are identified by `WARC-Type: c2paprovenance` together with `Content-Type: application/c2pa`

## Scope

This crate implements embedding and extraction only. Content binding — the C2PA collection data hash, computed over each record's uncompressed bytes — is out of scope; use the [official C2PA SDK](https://crates.io/crates/c2pa) to build and sign manifests.

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
