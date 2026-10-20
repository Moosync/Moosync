import os

try:
    from tools.instrument_common import FN_START_RE, transform_rs_files
except ImportError:
    from instrument_common import FN_START_RE, transform_rs_files

def update_skips(lines):
    new_lines = []
    idx = 0
    while idx < len(lines):
        line = lines[idx]
        match = FN_START_RE.match(line)
        if match:
            indent = match.group(1)
            instrument_idx = None
            for lookback in range(1, 6):
                if idx - lookback >= 0:
                    prev = lines[idx - lookback].strip()
                    if "#[tracing::instrument" in prev or "#[instrument" in prev:
                        instrument_idx = idx - lookback
                        break
                    if prev.endswith(("}", ";")):
                        break
            if instrument_idx is not None:
                existing_macro = lines[instrument_idx].strip()
                level = "debug"
                for lvl in ("trace", "error", "info", "warn"):
                    if f'level = "{lvl}"' in existing_macro:
                        level = lvl
                        break
                new_macro = f"{indent}#[tracing::instrument(level = \"{level}\", skip_all)]\n"
                target_new_idx = len(new_lines) - (idx - instrument_idx)
                new_lines[target_new_idx] = new_macro
        new_lines.append(line)
        idx += 1
    return new_lines

transform_rs_files(os.getcwd(), update_skips)
print("All instrument macros converted to skip_all.")
