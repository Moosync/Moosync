import argparse
import os
import shutil

def make_url(public_path: str, filename: str) -> str:
    relative_path = f"assets/{filename}"
    if not public_path:
        return relative_path
    return os.path.join(public_path, relative_path).replace("\\", "/")

def main() -> None:
    parser = argparse.ArgumentParser(description="Generate an HTML bundle.")
    parser.add_argument("--output_dir", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--header", required=True)
    parser.add_argument("--content", required=True)
    parser.add_argument("--public_path", default="")
    parser.add_argument("--css", nargs="*", default=[])
    parser.add_argument("--js", nargs="*", default=[])
    parser.add_argument("--wasm", nargs="*", default=[])
    parser.add_argument("--fonts", nargs="*", default=[])
    parser.add_argument("--assets", nargs="*", default=[])
    
    args = parser.parse_args()

    assets_dir = os.path.join(args.output_dir, "assets")
    os.makedirs(assets_dir, exist_ok=True)

    public_path: str = args.public_path

    css_links: list[str] = []
    for src_path in args.css:
        filename = os.path.basename(src_path)
        shutil.copy2(src_path, os.path.join(assets_dir, filename))
        
        if filename.lower().endswith(".css"):
            url = make_url(public_path, filename)
            css_links.append(f'<link rel="stylesheet" href="{url}">')

    js_scripts: list[str] = []
    for src_path in args.js:
        filename = os.path.basename(src_path)
        shutil.copy2(src_path, os.path.join(assets_dir, filename))
        
        if filename.lower().endswith(".js"):
            url = make_url(public_path, filename)
            # UPDATED: Added type="module"
            js_scripts.append(f'<script type="module" src="{url}"></script>')

    preloads: list[str] = []
    
    for src_path in args.wasm:
        filename = os.path.basename(src_path)
        shutil.copy2(src_path, os.path.join(assets_dir, filename))
        
        url = make_url(public_path, filename)
        preloads.append(f'<link rel="preload" href="{url}" as="fetch" type="application/wasm" crossorigin>')

    for src_path in args.fonts:
        filename = os.path.basename(src_path)
        shutil.copy2(src_path, os.path.join(assets_dir, filename))
        
        url = make_url(public_path, filename)
        ext = filename.split(".")[-1]
        preloads.append(f'<link rel="preload" href="{url}" as="font" type="font/{ext}" crossorigin>')

    for src_path in args.assets:
        filename = os.path.basename(src_path)
        shutil.copy2(src_path, os.path.join(assets_dir, filename))

    head_content = "\n    ".join(css_links + preloads)
    body_content = "\n    ".join(js_scripts)

    html_content: str = f"""
<!DOCTYPE html>
<html>
<head>
    <title>{args.title}</title>
    {head_content}
</head>
<body>
    <h1>{args.header}</h1>
    <p>{args.content}</p>
    {body_content}
</body>
</html>
    """

    with open(os.path.join(args.output_dir, "index.html"), "w", encoding="utf-8") as f:
        f.write(html_content)

if __name__ == "__main__":
    main()