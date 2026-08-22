import os
import re

files_to_instrument = [
    "core/state_manager/src/hooks/mod.rs",
    "core/state_manager/src/hooks/extensions.rs",
    "core/state_manager/src/hooks/player.rs",
    "core/state_manager/src/hooks/scanner.rs"
]

FN_START_RE = re.compile(r'^(\s*)(?:pub\s+(?:\([^)]+\)\s+)?)?(?:async\s+)?(?:const\s+)?fn\s+([a-zA-Z0-9_]+)\s*(?:<[^>]+>)?\s*\(')

project_root = os.getcwd()

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
            
            # Find the closing parenthesis
            full_decl = striped
            f_idx = idx
            while ")" not in lines[f_idx] and f_idx + 1 < len(lines):
                f_idx += 1
                full_decl += " " + lines[f_idx].strip()
            
            # Check up to 5 lines ahead for '{' to see if it's a definition
            is_definition = False
            for lookahead in range(0, 5):
                if f_idx + lookahead < len(lines):
                    ahead_line = lines[f_idx + lookahead].strip()
                    if "{" in ahead_line:
                        is_definition = True
                        break
                    if ";" in ahead_line:
                        break
            
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
            
            is_common = fn_name in ["main", "default", "from", "into", "try_from", "fmt", "clone", "drop"]
            
            if is_definition and not is_common:
                if instrument_idx is not None:
                    # Replace the existing instrument macro
                    target_new_idx = len(new_lines) - (idx - instrument_idx)
                    new_lines[target_new_idx] = f"{indent}#[tracing::instrument(level = \"debug\", skip_all)]\n"
                else:
                    # Insert a new instrument macro
                    new_lines.append(f"{indent}#[tracing::instrument(level = \"debug\", skip_all)]\n")
                    
        new_lines.append(line)
        idx += 1
        
    with open(path, "w") as f:
        f.writelines(new_lines)

print("Hooks instrumentation completed successfully.")
