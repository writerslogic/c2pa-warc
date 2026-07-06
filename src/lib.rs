// Copyright 2026 WritersLogic. All rights reserved.
// Licensed under the Apache License, Version 2.0 or the MIT license,
// at your option.

//! C2PA manifest embedding for WARC web archive files (ISO 28500).
//!
//! Stores the C2PA Manifest Store as a WARC `resource` record with
//! `Content-Type: application/c2pa`, appended at the end of the file.
//! Updates are append-only; the last manifest record in the file is the
//! active one.

mod error;
mod reader;
mod record;
mod writer;

pub use error::Error;
pub use reader::{read_manifest, read_records};
pub use record::WarcRecord;
pub use writer::append_manifest;
