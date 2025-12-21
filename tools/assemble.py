import sys
import os
import shutil

def main():
    # Usage: python assemble.py <output_dir> <file1> <file2> ...
    output_dir = sys.argv[1]
    input_files = sys.argv[2:]

    # 1. Clean/Create Output Directory
    if os.path.exists(output_dir):
        shutil.rmtree(output_dir)
    os.makedirs(output_dir)

    # 2. Copy all input files into the directory (flattening structure)
    for f_path in input_files:
        basename = os.path.basename(f_path)
        dest_path = os.path.join(output_dir, basename)
        
        # If input is a directory (rare in this context but possible), copytree it
        if os.path.isdir(f_path):
             shutil.copytree(f_path, dest_path, dirs_exist_ok=True)
        else:
            shutil.copy2(f_path, dest_path)

if __name__ == "__main__":
    main()