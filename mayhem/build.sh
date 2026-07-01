#!/usr/bin/env bash
#
# mayhem/build.sh — build sv-parser's in-process libFuzzer target as a sanitized
# binary (OSS-Fuzz Rust path: cargo-fuzz + ASan via RUSTFLAGS), then compile the
# crate's own #[test] suite (cargo test --no-run) for the mayhem/test.sh oracle.
#
# Runs inside the commit image (RUST mayhem/Dockerfile) as `mayhem` in /mayhem.
# The Rust toolchain + cargo registry live at $CARGO_HOME=/opt/toolchains/rust/cargo.
#
# AIR-GAPPED CONTRACT (SPEC §6.5): the PATCH tier re-runs THIS script OFFLINE.
#   - This FIRST build (in CI, online) populates the cargo registry under $CARGO_HOME.
#   - The PATCH re-run resolves crates from that cache (rlenv sets CARGO_NET_OFFLINE=true),
#     so we do NOT hard-code `--offline` here.
set -euo pipefail

# clang rejects SOURCE_DATE_EPOCH='' — must be unset or a valid integer.
[ -n "${SOURCE_DATE_EPOCH:-}" ] || unset SOURCE_DATE_EPOCH

: "${MAYHEM_JOBS:=$(nproc)}"
export CARGO_BUILD_JOBS="$MAYHEM_JOBS"

cd "$SRC"

# Sanitizers (§6.1): honor the KNOB. Non-empty $SANITIZER_FLAGS ⇒ ASan (OSS-Fuzz Rust
# path); explicit empty `--build-arg SANITIZER_FLAGS=` ⇒ un-sanitized build.
RUST_SAN=""
if [ -n "${SANITIZER_FLAGS:-}" ]; then
  RUST_SAN="-Zsanitizer=address"
fi

# Debug info (§6.2 item 10): produced binary MUST carry DWARF < 4. rustc nightly
# defaults to DWARF-5, so pin -Zdwarf-version=3 for Rust; pin the libfuzzer-sys cc
# shim's DWARF via CFLAGS/CXXFLAGS (clang defaults to DWARF-5 too).
export RUSTFLAGS="${RUSTFLAGS:-} ${RUST_DEBUG_FLAGS:-} --cfg fuzzing ${RUST_SAN} -Zdwarf-version=3 -Cdebuginfo=1 -Cforce-frame-pointers"
export CFLAGS="${CFLAGS:-} -gdwarf-3"
export CXXFLAGS="${CXXFLAGS:-} -gdwarf-3"

# The bundled ASan runtime archive that `-Zsanitizer=address` links is precompiled
# with clang (DWARF-5) with full debug info — that would otherwise land DWARF-5
# compile units in the final binary and fail the DWARF < 4 gate. Strip the debug
# info from that runtime archive (a TOOLCHAIN artifact, NOT project code).
# Idempotent: --strip-debug on an already-stripped archive is a no-op (offline-safe).
if [ -n "${RUST_SAN}" ]; then
  RT_LIB_DIR="$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/lib"
  for asan in "$RT_LIB_DIR"/librustc-*_rt.asan.a; do
    [ -f "$asan" ] || continue
    if [ -w "$asan" ]; then
      objcopy --strip-debug "$asan" "$asan.stripped" && mv "$asan.stripped" "$asan"
      echo "stripped debug info from bundled ASan runtime: $asan"
    fi
  done
fi

FUZZ_DIR="mayhem/fuzz"
TRIPLE="x86_64-unknown-linux-gnu"

# Discover every target from the fuzz crate's fuzz_targets/ dir.
FUZZ_TARGETS=()
for f in "$FUZZ_DIR"/fuzz_targets/*.rs; do
  FUZZ_TARGETS+=("$(basename "${f%.*}")")
done
[ "${#FUZZ_TARGETS[@]}" -gt 0 ] || { echo "ERROR: no fuzz targets under $FUZZ_DIR/fuzz_targets/" >&2; exit 1; }

echo "=== cargo fuzz build (image nightly, ASan via RUSTFLAGS) ==="
echo "RUSTFLAGS=$RUSTFLAGS"
echo "targets: ${FUZZ_TARGETS[*]}"

for t in "${FUZZ_TARGETS[@]}"; do
  echo "--- building fuzz target: $t ---"
  cargo fuzz build --fuzz-dir "$FUZZ_DIR" -O --debug-assertions "$t"
  bin="$SRC/$FUZZ_DIR/target/$TRIPLE/release/$t"
  [ -x "$bin" ] || { echo "ERROR: expected fuzz binary not found at $bin" >&2; exit 1; }
  cp "$bin" "/mayhem/$t"
  echo "built /mayhem/$t"
done

# ── Build the crate's OWN functional test suite for the mayhem/test.sh oracle ──
# The real known-answer tests live in sv-parser-parser/src/tests.rs (69 parser
# known-answer tests) and sv-parser-pp/src/{preprocess,range}.rs (52 preprocessor
# tests) — both crates have NO dev-dependencies. We deliberately scope the test
# build to these two packages and do NOT build the whole workspace: the top-level
# sv-parser crate carries criterion/plotters benchmark dev-deps whose transitive
# `zerocopy` needs AVX512 stdarch features unstable on the pinned nightly, which
# would fail an UN-related bench dependency (not the parser under test). Scoping to
# the two test-bearing packages keeps a strong behavioral oracle without dragging in
# that bench toolchain incompatibility. Compile (not run) here so test.sh only RUNS
# a prebuilt runner (cache-resolved). Build UN-sanitized (env -u RUSTFLAGS) so the
# test binary is a plain runnable ELF regardless of the ASan fuzz build above.
echo "=== compiling functional test suite (cargo test --no-run) ==="
env -u RUSTFLAGS cargo test --no-run -p sv-parser-parser -p sv-parser-pp

echo "build.sh complete"
