// Copyright 2026 WritersLogic. All rights reserved.
// Licensed under the Apache License, Version 2.0 or the MIT license,
// at your option.

//! C2PA manifest embedding for WARC web archive files (ISO 28500).
//!
//! Stores the C2PA Manifest Store as a WARC `resource` record with
//! `Content-Type: application/c2pa`, appended at the end of the file.

mod error;
mod record;
mod reader;
mod writer;

pub use error::Error;
pub use record::WarcRecord;
pub use reader::read_manifest;
pub use writer::append_manifest;
