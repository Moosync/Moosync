#!/usr/bin/env bash
# Helper script to collect Bazel test coverage and Clippy diagnostics for SonarQube.
set -euo pipefail

WORKSPACE_DIR="${BUILD_WORKSPACE_DIRECTORY:-$(pwd)}"
cd "$WORKSPACE_DIR"

RUN_COVERAGE=true
RUN_CLIPPY=false
RUN_SCAN=true
REMAINING_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --coverage)
      RUN_COVERAGE=true
      shift
      ;;
    --no-coverage)
      RUN_COVERAGE=false
      shift
      ;;
    --clippy)
      RUN_CLIPPY=true
      shift
      ;;
    --no-scan|--collect-only)
      RUN_SCAN=false
      shift
      ;;
    *)
      REMAINING_ARGS+=("$1")
      shift
      ;;
  esac
done

if [ "$RUN_COVERAGE" = true ]; then
  echo "==> Running Bazel test coverage on //:core_tests..."
  bazel coverage -k //:core_tests || true
fi

if [ "$RUN_CLIPPY" = true ]; then
  echo "==> Running Bazel Clippy aspect to emit diagnostics..."
  bazel build --@rules_rust//rust/settings:clippy_output_diagnostics=true --output_groups=+clippy_output //:core_tests
fi

echo "==> Collecting test coverage..."
COVERAGE_OUT="${WORKSPACE_DIR}/coverage.lcov"
> "$COVERAGE_OUT"

FOUND_COVERAGE=0
if [ -d "bazel-testlogs" ]; then
  while IFS= read -r f; do
    if [ -s "$f" ]; then
      cat "$f" >> "$COVERAGE_OUT"
      echo "" >> "$COVERAGE_OUT"
      FOUND_COVERAGE=$((FOUND_COVERAGE + 1))
    fi
  done < <(find bazel-testlogs/ -type f -name "coverage.dat" 2>/dev/null | sort)
fi

if [ "$FOUND_COVERAGE" -gt 0 ]; then
  echo "Collected $FOUND_COVERAGE coverage report(s) into coverage.lcov ($(wc -l < "$COVERAGE_OUT") lines)."
else
  echo "No coverage.dat files found in bazel-testlogs."
fi

echo "==> Collecting Clippy diagnostics..."
CLIPPY_OUT="${WORKSPACE_DIR}/clippy-report.json"
> "$CLIPPY_OUT"

FOUND_CLIPPY=0
if [ -d "bazel-bin" ]; then
  while IFS= read -r f; do
    if [ -s "$f" ]; then
      while IFS= read -r line; do
        if [[ "$line" =~ "\"\\\$message_type\":\"diagnostic\"" ]] && [[ ! "$line" =~ "\"code\":null" ]]; then
          echo "{\"reason\":\"compiler-message\",\"message\":$line}" >> "$CLIPPY_OUT"
          FOUND_CLIPPY=$((FOUND_CLIPPY + 1))
        elif [[ "$line" =~ "\"reason\":\"compiler-message\"" ]]; then
          echo "$line" >> "$CLIPPY_OUT"
          FOUND_CLIPPY=$((FOUND_CLIPPY + 1))
        fi
      done < "$f"
    fi
  done < <(find bazel-bin/ -type f -name "*.clippy.diagnostics" 2>/dev/null | sort)
fi

if [ "$FOUND_CLIPPY" -gt 0 ]; then
  echo "Collected $FOUND_CLIPPY Clippy diagnostic report(s) into clippy-report.json."
else
  # Keep an empty clippy-report.json so SonarQube imports without error
  > "$CLIPPY_OUT"
  echo "No Clippy warnings/diagnostics found."
fi

# Run sonar-scanner if scan is enabled and scanner arguments or files are ready
if [ "$RUN_SCAN" = true ]; then
  SONAR_BIN="$(command -v sonar-scanner 2>/dev/null || echo "/opt/sonar-scanner/bin/sonar-scanner")"
  if [ -x "$SONAR_BIN" ]; then
    echo "==> Executing $SONAR_BIN ${REMAINING_ARGS[*]}..."
    exec "$SONAR_BIN" "${REMAINING_ARGS[@]}"
  else
    echo "ERROR: sonar-scanner not found at '$SONAR_BIN'" >&2
    exit 1
  fi
fi
