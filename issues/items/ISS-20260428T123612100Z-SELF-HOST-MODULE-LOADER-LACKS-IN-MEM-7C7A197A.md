---
id: ISS-20260428T123612100Z-SELF-HOST-MODULE-LOADER-LACKS-IN-MEM-7C7A197A
title: "self-host module loader lacks in-memory VFS parse entry"
area: selfhost
status: fixed
resolved: true
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/core/module/loader.nepl, tests/stdlib/neplg2_module_loader.n.md"
---

# ISS-20260428T123612100Z-SELF-HOST-MODULE-LOADER-LACKS-IN-MEM-7C7A197A: self-host module loader lacks in-memory VFS parse entry

## 概要

`core/module/loader.nepl` はまだ Stage 0 marker API で、S2 の前提である filesystem 非依存の `path -> source` 境界を実装していない。所有された仮想 file 一覧、path lookup、source text を実行可能 parser へ渡す loader API が無い。

## 対象

- `stdlib/neplg2/core/module/loader.nepl, tests/stdlib/neplg2_module_loader.n.md`

## 根拠

- `doc/neplg2/self_host_plan.md` の S2 は、CLI が input file と stdlib root を読み、core 側へ `VirtualFileSystem` として渡す方針を要求している。
- 現在の `stdlib/neplg2/core/module/loader.nepl` は `selfhost_loader_stage0` が `0` を返すだけで、module parser 実装後も loader 経由で AST を得る経路が無い。
- core 層は `std/fs` に依存しない設計なので、filesystem 直結ではなく in-memory VFS を loader 境界として固定する必要がある。

## 問題

`core/module/loader.nepl` が Stage 0 のまま残っているため、parser が `SelfhostModuleAst` を返せるようになっても、self-host pipeline が「logical path を指定して module を読む」単位へ進めない。

## 影響

CLI/file_io から複数 input file や stdlib source を core に渡す安定 API が無い。import graph、cycle detection、stdlib map の実装が parser へ接続できず、S2 の module loading を開始できない。

## 修正方針

`core/module/loader.nepl` に小さな in-memory VFS と loaded module API を追加する。loader は path lookup 後に `selfhost_parse_module_source` へ source を渡し、成功時は AST 所有権を `SelfhostLoadedModule` として返す。missing file と parser diagnostic は `SelfhostDiagnostic` として返し、所有 AST の free helper も提供する。

## 検証

- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_loader.n.md --no-tree`
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_loader.n.md --no-tree`
- `node nodesrc/issues.js check`

## 2026-04-28 修正

- `stdlib/neplg2/core/module/loader.nepl` を Stage 0 marker から、`SelfhostVirtualFileSystem` / `SelfhostVirtualFile` / `SelfhostLoadedModule` を持つ実装へ進めた。
- VFS は `Vec<SelfhostVirtualFile>` を所有し、`selfhost_vfs_add`、`selfhost_vfs_len`、`selfhost_vfs_get`、`selfhost_vfs_find`、`selfhost_vfs_free` を提供する。
- `selfhost_load_module` は logical path を VFS から線形探索し、見つかった source を `selfhost_parse_module_source` へ渡して AST を返す。missing file は `selfhost.loader.file_not_found` diagnostic とし、path を note に入れる。
- `selfhost_loaded_module_ast` / `selfhost_loaded_module_path` / `selfhost_loaded_module_free` を追加し、AST の所有権と解放責務を loader 境界で明示した。
- `tests/stdlib/neplg2_module_loader.n.md` を追加し、in-memory VFS からの module load と missing file diagnostic を回帰テスト化した。
- 実装中に、lexer が span の `file_id` を常に `0` に固定していることを確認した。これは multi-file VFS の診断位置に関わる別問題として `ISS-20260428T123810929Z-SELF-HOST-LEXER-HARDCODES-SOURCE-FIL-B72A7A74` に分離した。

## 2026-04-28 検証

- `node nodesrc/tests.js -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-module-loader-focused.json -j 1`: total=2, passed=2
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-selfhost-loader-final.json -j 1`: total=27, passed=27
- `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-loader-with-syntax-regressions.json -j 1`: total=40, passed=40
- `trunk build`: pass
- remote main `5270999` へ rebase 後:
  - `trunk build`: pass
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-module-loader-focused-after-rebase.json -j 1`: total=2, passed=2
  - `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-loader-with-syntax-after-rebase.json -j 1`: total=40, passed=40
- remote main `6c0b9b6` へ rebase 後:
  - `trunk build`: pass
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-module-loader-focused-after-second-rebase.json -j 1`: total=2, passed=2
  - `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-loader-with-syntax-after-second-rebase.json -j 1`: total=40, passed=40
- remote main `8ce052f` へ rebase 後:
  - `trunk build`: pass
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-module-loader-focused-after-8ce052f.json -j 1`: total=2, passed=2
  - `node nodesrc/tests.js -i stdlib/neplg2 -i tests/stdlib/neplg2_module_loader.n.md -i tests/stdlib/neplg2_parser.n.md -i tests/stdlib/neplg2_lexer.n.md --no-tree -o tmp/neplg2-selfhost-loader-with-syntax-after-8ce052f.json -j 1`: total=40, passed=40
  - `node nodesrc/issues.js check`: files=282, pass
  - `git diff --check HEAD`: pass
- remote main `dbdfa74` へ rebase 後:
  - `trunk build`: pass
  - `node nodesrc/tests.js -i tests/stdlib/neplg2_module_loader.n.md --no-tree -o tmp/neplg2-module-loader-focused-after-dbdfa74.json -j 1`: total=2, passed=2
  - `node nodesrc/issues.js check`: files=284, pass
  - `git diff --check HEAD`: pass
