#![no_main]

use c2pa_warc::{read_manifest, read_records};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = read_manifest(data);
    let _ = read_records(data);
});
