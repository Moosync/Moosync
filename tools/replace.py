import sys
import json
import os

def main():
    # Usage: python replace.py <template_path> <output_path> <json_substitutions>
    template_path = sys.argv[1]
    output_path = sys.argv[2]
    subs_json = sys.argv[3]

    # Parse the dictionary
    substitutions = json.loads(subs_json)

    with open(template_path, "r") as f:
        content = f.read()

    # Generic replace loop
    for key, value in substitutions.items():
        content = content.replace(key, value)

    with open(output_path, "w") as f:
        f.write(content)

if __name__ == "__main__":
    main()