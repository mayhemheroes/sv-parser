#![no_main]
use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    _ = sv_parser::parse_sv_str::<_, &str, _>(data, "fuzz.f", &HashMap::new(), &[], true, true);
});
