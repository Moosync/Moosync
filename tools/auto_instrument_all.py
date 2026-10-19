import os

try:
    from tools.instrument_common import FN_START_RE, IGNORE_DIRECTIVE_RE, get_all_rs_files
except ImportError:
    from instrument_common import FN_START_RE, IGNORE_DIRECTIVE_RE, get_all_rs_files

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
            
            # Check if this function already has instrument or ignore directive above it
            instrument_idx = None
            is_ignored = False
            comment_lines = []
            for lookback in range(1, 8):
                if idx - lookback >= 0:
                    prev = lines[idx - lookback].strip()
                    if prev.startswith("//"):
                        comment_lines.append(prev.lstrip("/").strip())
                    elif "#[tracing::instrument" in prev or "#[instrument" in prev:
                        instrument_idx = idx - lookback
                        break
                    elif prev.startswith("#["):
                        continue
                    elif prev.endswith(("}", ";")):
                        break

            joined_comments = " ".join(reversed(comment_lines))
            if "moosync::no_instrument" in joined_comments:
                directive_match = re.search(r'moosync::no_instrument\s*\(\s*reason\s*=\s*["\']([^"\']+)["\']\s*\)', joined_comments)
                if directive_match and directive_match.group(1).strip():
                    is_ignored = True

            is_common = fn_name in ["main", "default", "from", "into", "try_from", "fmt", "clone", "drop"]

            # Ignore test modules and traits (but check_file handles this by context. For simple script,
            # we just do it if it's a definition and not common)
            if is_definition and not is_common and not is_ignored:
                # Check if we are inside a test module or trait definition in the lines so far
                in_test = False
                in_trait = False
                brace_count = 0
                for l in new_lines:
                    l_strip = l.strip()
                    if "mod test" in l_strip or "mod tests" in l_strip:
                        in_test = True
                    if l_strip.startswith(("pub trait ", "trait ")):
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
