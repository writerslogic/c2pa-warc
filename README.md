<!-- repo-header:start -->
<img src="https://github.com/writerslogic.png?size=160" alt="c2pa-warc logo" width="120" align="left">

<h1>c2pa-warc</h1>

<p><strong>C2PA manifest embedding for WARC web archive files (ISO 28500)</strong></p>

<br clear="left">

[![CI](https://img.shields.io/github/actions/workflow/status/writerslogic/c2pa-warc/ci.yml?style=flat-square&labelColor=20232a&branch=main&label=CI)](https://github.com/writerslogic/c2pa-warc/actions/workflows/ci.yml) [![OpenSSF Scorecard](https://img.shields.io/ossf-scorecard/github.com/writerslogic/c2pa-warc?style=flat-square&labelColor=20232a&label=OpenSSF)](https://securityscorecards.dev/viewer/?uri=github.com/writerslogic/c2pa-warc) [![OpenSSF Best Practices](https://www.bestpractices.dev/projects/14418/badge)](https://www.bestpractices.dev/projects/14418) [![License](https://img.shields.io/github/license/writerslogic/c2pa-warc?style=flat-square&labelColor=20232a&color=007ec6&label=license)](https://github.com/writerslogic/c2pa-warc/blob/main/LICENSE-APACHE) [![Code of Conduct](https://img.shields.io/badge/code%20of%20conduct-Contributor%20Covenant%202.1-6a4c93?style=flat-square&labelColor=20232a)](https://github.com/writerslogic/c2pa-warc/blob/main/CODE_OF_CONDUCT.md) [![C2PA](https://img.shields.io/badge/standard-C2PA%20related-6a4c93?style=flat-square&labelColor=20232a)](https://c2pa.org/) [![GitHub Sponsors](https://img.shields.io/badge/GitHub%20Sponsors-Sponsor-EA4AAA?style=flat-square&labelColor=20232a)](https://github.com/sponsors/dcondrey) <a href="https://crates.io/crates/c2pa-warc"><img src="https://img.shields.io/crates/v/c2pa-warc.svg?style=flat-square&labelColor=20232a&color=007ec6" alt="crates.io"></a> <a href="https://docs.rs/c2pa-warc"><img src="https://img.shields.io/docsrs/c2pa-warc?style=flat-square&labelColor=20232a&color=007ec6" alt="docs.rs"></a>
<!-- repo-header:end -->

## Overview

Stores and retrieves C2PA Manifest Stores in [WARC 1.1](https://iipc.github.io/warc-specifications/specifications/warc-format/warc-1.1/) files (ISO 28500). The manifest is stored as a WARC record of a dedicated `c2pa-provenance` type with `Content-Type: application/c2pa`, as the last record in the file.

WARC is used by national libraries, legal deposit systems, and digital preservation institutions to archive web content. This crate enables content provenance for archived web resources.

> [!IMPORTANT]
> **The C2PA Technical Specification defines no WARC embedding method.** Unlike HTML, structured text, ZIP, ONNX, or SafeTensors — each of which has a normative clause — WARC embedding is a proposal, and what this crate implements is that proposed shape. It conforms to [WARC 1.1](https://iipc.github.io/warc-specifications/specifications/warc-format/warc-1.1/) and produces a valid archive readable by any WARC tool; it does not claim C2PA specification conformance, because there is nothing yet to conform to.

Zero dependencies.

## Quick Start

```toml
[dependencies]
c2pa-warc = "0.2"
```

The same crate is published for JavaScript/WebAssembly and Python, built from this source:

```bash
npm install c2pa-warc   # wasm-bindgen build
pip install c2pa-warc   # PyO3 abi3 wheel, CPython 3.9+
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

## Example

`cargo run --example make_sample_warc` writes a minimal WARC (`examples/sample.warc`) containing a `warcinfo` record, one captured `response` record, and a trailing `c2pa-provenance` record whose block is a real C2PA Manifest Store, then extracts it back to `examples/sample.manifest.c2pa`. The manifest record header:

```
WARC/1.1
WARC-Type: c2pa-provenance
WARC-Record-ID: <urn:uuid:...>
WARC-Date: ...
Content-Type: application/c2pa
Content-Length: 3576
```

## Design

- Manifest stored as a WARC record of a dedicated `c2pa-provenance` type, as the last record in the file; it carries no `WARC-Target-URI`
- At most one manifest record; an update removes the existing one and appends the replacement
- Manifest records are identified by `WARC-Type: c2pa-provenance` together with `Content-Type: application/c2pa`

## Scope

This crate implements embedding and extraction only. Content binding — the C2PA collection data hash, computed over each record's uncompressed bytes — is out of scope; use the [official C2PA SDK](https://crates.io/crates/c2pa) to build and sign manifests.

## Related Crates

Part of a family of single-purpose crates, one per C2PA embedding method. Each
is standalone and independently versioned.

| Crate | Description |
|---|---|
| [c2pa-structured-text](https://crates.io/crates/c2pa-structured-text) | Structured text: ASCII-armoured manifest in a comment or front matter |
| [c2pa-unstructured-text](https://crates.io/crates/c2pa-unstructured-text) | Unstructured text: invisible Unicode variation-selector run |
| [c2pa-html](https://crates.io/crates/c2pa-html) | HTML: `script` and `link` elements in the document head |
| [c2pa-http](https://crates.io/crates/c2pa-http) | HTTP: the `c2pa-manifest` `Link` header, with a Tower middleware |
| [c2pa-text-binding](https://crates.io/crates/c2pa-text-binding) | Soft binding and content fingerprinting for text assets |
| [c2pa-vtt](https://crates.io/crates/c2pa-vtt) | WebVTT caption and subtitle embedding |
| [c2pa-zip](https://crates.io/crates/c2pa-zip) | ZIP-based documents: EPUB, DOCX, ODT, OXPS |
| [c2pa-fonts](https://crates.io/crates/c2pa-fonts) | OpenType/TrueType (SFNT) font embedding |
| [c2pa-ml](https://crates.io/crates/c2pa-ml) | ML model containers: GGUF, SafeTensors, ONNX |
| [c2pa](https://crates.io/crates/c2pa) | Official C2PA SDK |

## Security

Found a vulnerability? Please report it privately — see [SECURITY.md](./SECURITY.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

Built by [WritersLogic](https://writerslogic.com)
