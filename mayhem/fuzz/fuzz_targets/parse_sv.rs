// Additive in-process libFuzzer harness for sv-parser (SystemVerilog parser).
//
// sv-parser exposes a public library API; this harness feeds the fuzzer bytes as
// SystemVerilog source text directly into parse_sv_str (the same entrypoint the
// crate documents), with no disk I/O and includes disabled (ignore_include=true)
// so nothing is opened from the read-only image dir. The parse Result is dropped.
// This preserves the original mayhemheroes target name (parse_sv) and API. Upstream
// source is untouched; this crate only CALLS it.
#![no_main]

use std::collections::HashMap;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    // ignore_include=true keeps the parser from touching the filesystem; the path
    // "fuzz.sv" is only used for diagnostics. allow_incomplete=true exercises the
    // full parser recovery path. Empty pre-defines and include-paths.
    let _ = sv_parser::parse_sv_str::<_, &str, _>(
        data,
        "fuzz.sv",
        &HashMap::new(),
        &[],
        true,  // ignore_include
        true,  // allow_incomplete
    );
});
