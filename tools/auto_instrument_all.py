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
            
            # Ignore test modules and traits (but check_file handles this by context. For simple script,
            # we just do it if it's a definition and not common)
            if is_definition and not is_common:
                # Check if we are inside a test module or trait definition in the lines so far
                in_test = False
                in_trait = False
                brace_count = 0
                for l in new_lines:
                    l_strip = l.strip()
                    if "mod test" in l_strip or "mod tests" in l_strip:
                        in_test = True
                    if l_strip.startswith("pub trait ") or l_strip.startswith("trait "):
                        in_trait = True
                    if "{" in l:
                        brace_count += l.count("{")
                    if "}" in l:
                        brace_count -= l.count("}")
                        if brace_count <= 0:
                            in_test = False
                            in_trait = False
                            brace_count = 0
                
                if not in_test and not in_trait:
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
                        target_new_idx = len(new_lines) - (idx - instrument_idx)
                        new_lines[target_new_idx] = new_macro
                    else:
                        # Insert a new instrument macro
                        new_lines.append(f"{indent}#[tracing::instrument(level = \"debug\", skip_all)]\n")
                    
        new_lines.append(line)
        idx += 1
        
    with open(path, "w") as f:
        f.writelines(new_lines)

print("All functions in all rs files instrumented with skip_all.")
