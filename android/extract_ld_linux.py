"""Extracts ld-linux from termux-pacman glibc archives and aligns ELF segments to 16 KiB pages."""

import argparse
import tarfile

from tools.elf_align import align_elf_16k


def extract_member(archive_path: str, member_name: str, align_16k: bool = False) -> bytes:
    """Extracts a member from a .tar.xz archive and optionally aligns ELF64 segments to 16 KiB."""
    with tarfile.open(archive_path, "r:xz") as tar:
        fobj = tar.extractfile(member_name)
        if fobj is None:
            raise RuntimeError(f"Member not found: {member_name}")
        raw = fobj.read()
    return align_elf_16k(raw, force_repack=True) if align_16k else raw


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", required=True)
    parser.add_argument("--member", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--align-16k", action="store_true")
    args = parser.parse_args()

    raw = extract_member(args.archive, args.member, align_16k=args.align_16k)
    with open(args.out, "wb") as out_f:
        out_f.write(raw)


if __name__ == "__main__":
    main()
