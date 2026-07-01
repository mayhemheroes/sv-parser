#!/usr/bin/env bash
#
# mayhem/test.sh — RUN sv-parser's OWN functional test suite (compiled by
# mayhem/build.sh via `cargo test --no-run --workspace`). These are the crate's
# real known-answer parser tests in sv-parser-parser/src/tests.rs — they assert
# that specific SystemVerilog snippets parse to Ok/Err with expected error
# positions, NOT just exit status. A PATCH that neuters the parser to a no-op /
# exit(0) makes those assertions fail, so this is a genuine behavioral oracle
# (anti-reward-hacking, SPEC §6.3).
#
# Emits a CTRF summary. Exit 0 iff failed==0 AND passed>0. Does NOT recompile the
# world (build.sh already produced the runners; cargo resolves from cache).
set -uo pipefail
[ -n "${SOURCE_DATE_EPOCH:-}" ] || unset SOURCE_DATE_EPOCH
: "${MAYHEM_JOBS:=$(nproc)}"
cd "$SRC"

# emit_ctrf <tool> <passed> <failed> [skipped] [pending] [other]
emit_ctrf() {
  local tool="$1" passed="$2" failed="$3" skipped="${4:-0}" pending="${5:-0}" other="${6:-0}"
  local tests=$(( passed + failed + skipped + pending + other ))
  cat > "${CTRF_REPORT:-$SRC/ctrf-report.json}" <<JSON
{
  "results": {
    "tool": { "name": "$tool" },
    "summary": {
      "tests": $tests,
      "passed": $passed,
      "failed": $failed,
      "pending": $pending,
      "skipped": $skipped,
      "other": $other
    }
  }
}
JSON
  printf 'CTRF {"results":{"tool":{"name":"%s"},"summary":{"tests":%d,"passed":%d,"failed":%d,"pending":%d,"skipped":%d,"other":%d}}}\n' \
    "$tool" "$tests" "$passed" "$failed" "$pending" "$skipped" "$other"
  [ "$failed" -eq 0 ]
}

# RUN the prebuilt suite. build.sh already compiled it (cargo test --no-run
# --workspace), so this resolves from the build cache. env -u RUSTFLAGS mirrors the
# build so cargo hits the same (un-sanitized) test artifacts and does not rebuild
# under ASan. Capture libtest's "test result: ok. N passed; M failed; K ignored".
LOG="$(mktemp)"
env -u RUSTFLAGS cargo test -p sv-parser-parser -p sv-parser-pp --no-fail-fast 2>&1 | tee "$LOG"

# Parse every libtest summary line and sum them across all test binaries.
PASSED=0; FAILED=0; SKIPPED=0; SAW=0
while IFS= read -r line; do
  if [[ "$line" =~ test\ result:.*\ ([0-9]+)\ passed\;\ ([0-9]+)\ failed\;\ ([0-9]+)\ ignored ]]; then
    PASSED=$(( PASSED + ${BASH_REMATCH[1]} ))
    FAILED=$(( FAILED + ${BASH_REMATCH[2]} ))
    SKIPPED=$(( SKIPPED + ${BASH_REMATCH[3]} ))
    SAW=1
  fi
done < "$LOG"
rm -f "$LOG"

# No summary marker at all ⇒ the suite never ran (neutered/no-op) ⇒ hard FAIL.
if [ "$SAW" -eq 0 ]; then
  echo "FATAL: no libtest 'test result:' marker seen — suite did not run" >&2
  emit_ctrf "cargo-test" 0 1
  exit 1
fi

# Sanity floor: the real suite has hundreds of #[test]s; 0 passed ⇒ vacuous oracle.
if [ "$PASSED" -eq 0 ]; then
  echo "FATAL: 0 tests passed — oracle would be vacuous" >&2
  emit_ctrf "cargo-test" 0 $(( FAILED > 0 ? FAILED : 1 ))
  exit 1
fi

emit_ctrf "cargo-test" "$PASSED" "$FAILED" "$SKIPPED"
