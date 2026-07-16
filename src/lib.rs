// Copyright 2026 WritersLogic. All rights reserved.
// Licensed under the Apache License, Version 2.0 or the MIT license,
// at your option.

//! C2PA manifest embedding for WARC web archive files (ISO 28500).
//!
//! Stores the C2PA Manifest Store as a WARC record of a dedicated
//! `c2paprovenance` type with `Content-Type: application/c2pa`, as the last
//! record in the file. A file carries at most one manifest record; updating
//! removes the existing one and appends the replacement.

mod error;
mod reader;
mod record;
mod writer;

pub use error::Error;
pub use reader::{read_manifest, read_records};
pub use record::{build_record, WarcRecord};
pub use writer::append_manifest;
