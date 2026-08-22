import os
import re

def get_all_rs_files(project_root):
    rs_files = []
    # Check core/ and ui/slint/src/
    search_dirs = [
        os.path.join(project_root, "core"),
        os.path.join(project_root, "ui", "slint", "src")
    ]
    for search_dir in search_dirs:
        if not os.path.exists(search_dir):
            continue
        for root, _, files in os.walk(search_dir):
            if "plugin_macro" in root or "theme_macro" in root or "benches" in root:
                continue
            for file in files:
                if file.endswith(".rs"):
                    if "test" in file or "bench" in file or file == "build.rs":
                        continue
                    rs_files.append(os.path.relpath(os.path.join(root, file), project_root))
    return rs_files

FN_START_RE = re.compile(r'^(\s*)(?:pub\s+(?:\([^)]+\)\s+)?)?(?:async\s+)?(?:const\s+)?fn\s+([a-zA-Z0-9_]+)\s*(?:<[^>]+>)?\s*\(')

project_root = os.getcwd()
files_to_instrument = get_all_rs_files(project_root)

for rel_path in files_to_instrument:
    path = os.path.join(project_root, rel_path)
    if not os.path.exists(path):
        continue
    
    with open(path, "r") as f:
        lines = f.readlines()
        
    new_lines = []
    idx = 0
    while idx < len(lines):
        line = lines[idx]
        striped = line.strip()
        
        match = FN_START_RE.match(line)
        if match:
            indent = match.group(1)
            fn_name = match.group(2)
            
            # Check if this function already has instrument above it
            instrument_idx = None
            for lookback in range(1, 6):
                if idx - lookback >= 0:
                    prev = lines[idx - lookback].strip()
                    if "#[tracing::instrument" in prev or "#[instrument" in prev:
                        instrument_idx = idx - lookback
                        break
                    if prev.endswith("}") or prev.endswith(";"):
                        break
            
            if instrument_idx is not None:
                # Get the existing macro line
                existing_macro = lines[instrument_idx].strip()
                
                # Check if it was level = "trace"
                level = "debug"
                if 'level = "trace"' in existing_macro or 'level = "trace"' in line:
                    level = "trace"
                elif 'level = "error"' in existing_macro:
                    level = "error"
                elif 'level = "info"' in existing_macro:
                    level = "info"
                elif 'level = "warn"' in existing_macro:
                    level = "warn"
                
                new_macro = f"{indent}#[tracing::instrument(level = \"{level}\", skip_all)]\n"
                
                # Replace the instrument macro at instrument_idx!
                target_new_idx = len(new_lines) - (idx - instrument_idx)
                new_lines[target_new_idx] = new_macro
        
        new_lines.append(line)
        idx += 1
        
    with open(path, "w") as f:
        f.writelines(new_lines)

print("All instrument macros converted to skip_all.")
