#!/usr/bin/env python3
"""
repo_metrics.py

Measure repository scale by counting:
  - files
  - total lines
  - character count
  - total bytes (file sizes)
grouped by file extension, while excluding files ignored by Git.

Additionally, for selected extensions (default: .rs, .nepl, .js),
compute:
  - blank lines
  - comment lines (lines starting with // after leading whitespace)
  - code lines (non-blank and non-comment)

Recommended mode ("git", default if available):
    Uses `git ls-files --cached --others --exclude-standard` to get:
      - tracked files, plus
      - untracked files that are NOT ignored by .gitignore / .git/info/exclude / global excludes.

Fallback mode ("walk"):
    Walks the directory tree and filters using only the root .gitignore via the `pathspec` library.
    This does NOT fully reproduce Git's multi-.gitignore behavior in subdirectories.
"""
from __future__ import annotations

import argparse
import csv
import json
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

try:
    import pathspec  # type: ignore
except Exception:  # pragma: no cover
    pathspec = None


@dataclass
class ExtStats:
    files: int = 0
    lines: int = 0
    chars: int = 0
    bytes: int = 0

    blank: int = 0
    comment: int = 0
    code: int = 0


def _run(cmd: List[str], cwd: Path) -> bytes:
    proc = subprocess.run(
        cmd,
        cwd=str(cwd),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        err = proc.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(err or f"Command failed: {' '.join(cmd)}")
    return proc.stdout


def is_git_repo(path: Path) -> bool:
    try:
        _run(["git", "rev-parse", "--is-inside-work-tree"], path)
        return True
    except Exception:
        return False


def git_root(path: Path) -> Path:
    out = _run(["git", "rev-parse", "--show-toplevel"], path)
    return Path(out.decode("utf-8", errors="replace").strip()).resolve()


def list_git_tracked_and_unignored(root: Path) -> List[Path]:
    out = _run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        root,
    )
    rels: List[Path] = []
    for raw in out.split(b"\0"):
        if not raw:
            continue
        rel = raw.decode("utf-8", errors="surrogateescape")
        rels.append(Path(rel))
    return rels


def load_root_gitignore_spec(root: Path):
    gitignore_path = root / ".gitignore"
    if not gitignore_path.exists():
        return None

    if pathspec is None:
        raise RuntimeError("pathspec is required for --mode walk. Install: pip install pathspec")

    lines = gitignore_path.read_text(encoding="utf-8", errors="replace").splitlines()
    return pathspec.GitIgnoreSpec.from_lines(lines)


def list_files_walk(root: Path) -> List[Path]:
    spec = load_root_gitignore_spec(root)
    rels: List[Path] = []

    for abs_path in root.rglob("*"):
        if abs_path.is_dir():
            continue
        if not abs_path.is_file():
            continue
        if ".git" in abs_path.parts:
            continue

        rel = abs_path.relative_to(root)

        if spec is not None:
            rel_posix = rel.as_posix()
            if spec.match_file(rel_posix):
                continue

        rels.append(rel)

    return rels


def is_probably_binary(path: Path, sample_size: int = 8192) -> bool:
    try:
        with path.open("rb") as f:
            chunk = f.read(sample_size)
        return b"\x00" in chunk
    except OSError:
        return True


def count_text_file(
    path: Path,
    max_bytes: Optional[int],
    count_comment_style: bool,
    comment_prefix: str,
) -> Tuple[int, int, int, int, int, int]:
    """
    Returns:
      lines, chars, bytes, blank, comment, code

    Notes:
      - Reads as UTF-8 with replacement.
      - Uses newline="" so CRLF is preserved in returned strings and character counts.
    """
    st = path.stat()
    size = int(st.st_size)
    if max_bytes is not None and max_bytes > 0 and size > max_bytes:
        raise ValueError(f"file too large ({size} bytes) > max_bytes")

    lines = 0
    chars = 0
    blank = 0
    comment = 0
    code = 0

    with path.open("r", encoding="utf-8", errors="replace", newline="") as f:
        for line in f:
            lines += 1
            chars += len(line)

            stripped = line.strip()
            if stripped == "":
                blank += 1
                continue

            if count_comment_style and line.lstrip().startswith(comment_prefix):
                comment += 1
            else:
                code += 1

    return lines, chars, size, blank, comment, code


def ext_key(rel_path: Path, suffix_mode: str) -> str:
    if suffix_mode == "all":
        suf = "".join(rel_path.suffixes).lower()
    else:
        suf = rel_path.suffix.lower()
    return suf if suf else "(no_ext)"


def print_table(stats: Dict[str, ExtStats], show_loc: bool) -> None:
    rows = sorted(
        ((ext, s) for ext, s in stats.items()),
        key=lambda t: (t[1].bytes, t[1].chars, t[1].lines, t[1].files),
        reverse=True,
    )

    headers = ["ext", "files", "lines", "chars", "bytes"]
    if show_loc:
        headers += ["blank", "comment", "code"]

    data_rows: List[List[str]] = []
    for ext, s in rows:
        r = [ext, f"{s.files:,}", f"{s.lines:,}", f"{s.chars:,}", f"{s.bytes:,}"]
        if show_loc:
            r += [f"{s.blank:,}", f"{s.comment:,}", f"{s.code:,}"]
        data_rows.append(r)

    widths = [len(h) for h in headers]
    for r in data_rows:
        widths = [max(w, len(cell)) for w, cell in zip(widths, r)]

    def fmt_row(r: List[str]) -> str:
        parts = []
        for i, cell in enumerate(r):
            parts.append(cell.ljust(widths[i]) if i == 0 else cell.rjust(widths[i]))
        return "  ".join(parts)

    print(fmt_row(headers))
    print(fmt_row(["-" * w for w in widths]))
    for r in data_rows:
        print(fmt_row(r))

    total = ExtStats()
    for s in stats.values():
        total.files += s.files
        total.lines += s.lines
        total.chars += s.chars
        total.bytes += s.bytes
        total.blank += s.blank
        total.comment += s.comment
        total.code += s.code

    print()
    total_row = ["TOTAL", f"{total.files:,}", f"{total.lines:,}", f"{total.chars:,}", f"{total.bytes:,}"]
    if show_loc:
        total_row += [f"{total.blank:,}", f"{total.comment:,}", f"{total.code:,}"]
    print(fmt_row(total_row))


def write_csv(path: Path, stats: Dict[str, ExtStats], show_loc: bool) -> None:
    with path.open("w", newline="", encoding="utf-8") as f:
        writer = csv.writer(f)
        headers = ["ext", "files", "lines", "chars", "bytes"]
        if show_loc:
            headers += ["blank", "comment", "code"]
        writer.writerow(headers)
        for ext, s in sorted(stats.items()):
            row = [ext, s.files, s.lines, s.chars, s.bytes]
            if show_loc:
                row += [s.blank, s.comment, s.code]
            writer.writerow(row)


def write_json(path: Path, stats: Dict[str, ExtStats], show_loc: bool) -> None:
    payload = []
    for ext, s in sorted(stats.items(), key=lambda kv: kv[0]):
        d = {"ext": ext, "files": s.files, "lines": s.lines, "chars": s.chars, "bytes": s.bytes}
        if show_loc:
            d.update({"blank": s.blank, "comment": s.comment, "code": s.code})
        payload.append(d)
    with path.open("w", encoding="utf-8") as f:
        json.dump(payload, f, ensure_ascii=False, indent=2)


def parse_ext_set(value: str) -> Set[str]:
    """
    Accepts: "rs,nepl,js" or ".rs,.nepl,.js" and normalizes to {".rs", ".nepl", ".js"}.
    """
    items = [x.strip() for x in value.split(",") if x.strip()]
    out: Set[str] = set()
    for it in items:
        out.add(it.lower() if it.startswith(".") else f".{it.lower()}")
    return out


def main(argv: Optional[List[str]] = None) -> int:
    p = argparse.ArgumentParser(
        description="Count lines/chars/bytes per extension for files NOT ignored by Git, plus comment/code/blank for selected extensions."
    )
    p.add_argument("--root", default=".", help="Path to repo root or any subdir (default: .)")
    p.add_argument(
        "--mode",
        choices=["auto", "git", "walk"],
        default="auto",
        help="auto: use git if available, else walk. git: require Git. walk: filesystem walk + root .gitignore only.",
    )
    p.add_argument(
        "--suffix-mode",
        choices=["last", "all"],
        default="last",
        help="Group extensions by last suffix (.tar.gz -> .gz) or all suffixes (.tar.gz).",
    )
    p.add_argument(
        "--max-bytes",
        type=int,
        default=5_000_000,
        help="Skip text counting for files larger than this many bytes (0 disables). Default: 5,000,000.",
    )
    p.add_argument(
        "--binary",
        choices=["skip", "bytes"],
        default="skip",
        help="Binary handling: skip (default) or count bytes as chars and set lines=0.",
    )
    p.add_argument("--csv", default=None, help="Write results to a CSV file.")
    p.add_argument("--json", default=None, help="Write results to a JSON file.")

    # LOC breakdown options
    p.add_argument(
        "--loc-exts",
        default="rs,nepl,js",
        help="Extensions to compute blank/comment/code for (comma-separated). Default: rs,nepl,js",
    )
    p.add_argument(
        "--comment-prefix",
        default="//",
        help="Comment prefix for LOC breakdown. Default: //",
    )
    p.add_argument(
        "--no-loc",
        dest="show_loc",
        action="store_false",
        help="Disable blank/comment/code columns.",
    )
    p.set_defaults(show_loc=True)

    args = p.parse_args(argv)

    root = Path(args.root).resolve()

    use_git = False
    if args.mode in ("auto", "git"):
        use_git = is_git_repo(root)
        if args.mode == "git" and not use_git:
            print("ERROR: --mode git but not inside a Git repository.", file=sys.stderr)
            return 2

    if use_git:
        root = git_root(root)
        rel_paths = list_git_tracked_and_unignored(root)
    else:
        rel_paths = list_files_walk(root)

    loc_exts = parse_ext_set(args.loc_exts)
    max_bytes = None if args.max_bytes == 0 else args.max_bytes

    stats: Dict[str, ExtStats] = defaultdict(ExtStats)
    skipped: List[Tuple[str, str]] = []

    for rel in rel_paths:
        abs_path = root / rel
        if not abs_path.is_file():
            continue

        key = ext_key(rel, args.suffix_mode)
        do_loc = args.show_loc and (rel.suffix.lower() in loc_exts)

        if is_probably_binary(abs_path):
            if args.binary == "skip":
                skipped.append((str(rel), "binary"))
                continue
            try:
                size = int(abs_path.stat().st_size)
            except OSError:
                skipped.append((str(rel), "unreadable"))
                continue
            s = stats[key]
            s.files += 1
            s.lines += 0
            s.chars += size
            s.bytes += size
            # LOC breakdown stays 0 for binary
            continue

        try:
            lines, chars, size, blank, comment, code = count_text_file(
                abs_path,
                max_bytes=max_bytes,
                count_comment_style=do_loc,
                comment_prefix=args.comment_prefix,
            )
        except ValueError:
            skipped.append((str(rel), "too_large"))
            continue
        except OSError:
            skipped.append((str(rel), "unreadable"))
            continue

        s = stats[key]
        s.files += 1
        s.lines += lines
        s.chars += chars
        s.bytes += size

        # Always accumulate blanks; comment/code only meaningful for selected extensions
        s.blank += blank
        s.comment += comment
        s.code += code

    print_table(stats, show_loc=args.show_loc)

    if skipped:
        print()
        print(f"Skipped files: {len(skipped)} (showing up to 20)")
        for path, reason in skipped[:20]:
            print(f"  - {path} [{reason}]")
        if len(skipped) > 20:
            print("  ...")

    if args.csv:
        write_csv(Path(args.csv), stats, show_loc=args.show_loc)
    if args.json:
        write_json(Path(args.json), stats, show_loc=args.show_loc)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
