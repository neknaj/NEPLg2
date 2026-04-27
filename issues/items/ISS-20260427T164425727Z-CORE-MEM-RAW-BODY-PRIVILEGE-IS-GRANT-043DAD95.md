---
id: ISS-20260427T164425727Z-CORE-MEM-RAW-BODY-PRIVILEGE-IS-GRANT-043DAD95
title: "core mem raw body privilege is granted by source path suffix"
area: core
status: fixed
resolved: true
priority: P1
type: security
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/typecheck.rs, nepl-core/src/source_map.rs, nepl-core/tests/effects.rs, nepl-cli/tests/cli_output.rs"
---

# ISS-20260427T164425727Z-CORE-MEM-RAW-BODY-PRIVILEGE-IS-GRANT-043DAD95: core mem raw body privilege is granted by source path suffix

## 概要

`raw_body_memory_operations_allowed` は SourceMap path が `/core/mem.nepl` で終わるかどうかで raw memory instruction 特権を与えている。compiler-owned module capability ではなくファイル名 suffix に依存しており、`core/mem.nepl` の unsafe boundary と loader/module identity の責務分割が不十分である。

## 対象

- `nepl-core/src/typecheck.rs, nepl-core/src/source_map.rs, nepl-core/tests/effects.rs, nepl-cli/tests/cli_output.rs`

## 根拠

- `nepl-core/src/typecheck.rs:2532` の `raw_body_memory_operations_allowed` は SourceMap から path string を取り出す。
- 同関数は path separator を正規化した後、`normalized.ends_with("/core/mem.nepl") || normalized == "core/mem.nepl"` だけで許可している。
- `nepl-core/src/typecheck.rs:2544` の `raw_memory_intrinsic_allowed` は raw `load` / `store` intrinsic 許可にも同じ判定を使う。
- `ISS-20260427T161537286Z-CORE-MEM-RAW-BOUNDARY-REJECTS-CUSTOM-52920E69` では custom stdlib root を通すため suffix 判定へ変更したが、これは transitional fix であり capability model ではない。
- `nepl-core/src/effects.rs:60` 以降で raw memory intrinsic 自体は `Effect::Impure` になっているため、許可境界の正しさが memory safety に直結する。

## 問題

alternate stdlib root を許可する要件自体は正しいが、path suffix は「その source が compiler によって監査された `core/mem` module か」を証明しない。生成 source、test fixture、user module、別 stdlib root が同じ suffix を持つ場合、raw body / raw intrinsic の unsafe 特権を与えるかどうかが loader の module identity ではなく文字列形状で決まってしまう。

## 影響

特権が広すぎる場合、user source が raw memory instruction を pure/effect 検査の例外として使える。狭すぎる場合、正当な configured stdlib が拒否される。どちらも `core/mem.nepl` の責務を compiler-owned unsafe boundary として管理する設計から外れており、self-host で stdlib/module loader が複雑になるほど事故りやすい。

## 修正方針

loader が import 解決時に「configured stdlib の `core/mem`」「compiler generated internal module」「user module」を区別する module identity/capability を SourceMap または typed module table へ付与する。`raw_body_memory_operations_allowed` は path suffix ではなくその capability を確認する。custom stdlib root は capability 付きで許可し、同名 user file は拒否する。

## 対応結果

- `SourceMap` に `SourceCapabilities` を追加し、source file ごとに compiler-owned raw memory boundary capability を保持できるようにした。
- `Loader` が configured stdlib root の `core/mem.nepl` を読み込む場合だけ `SourceCapabilities::raw_memory_boundary()` を付与するようにした。
- `raw_body_memory_operations_allowed` / raw memory intrinsic 許可は SourceMap path suffix ではなく capability を確認するように変更した。
- 既存の custom stdlib root は Loader が capability を付与するため引き続き許可される一方、同じ `/core/mem.nepl` suffix を持つ user file は raw memory instruction を使えない。

## 検証

既存の custom stdlib root regression は維持する。加えて、任意の user file / fixture を `core/mem.nepl` という path に置いて raw `i32.store` / `memory.grow` / `#intrinsic "store"` を書いた場合は compile_fail になることを確認する。SourceMap path のみを改変しても capability が付かないことを unit test で固定する。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test effects raw_memory -- --nocapture`: `8 passed`
- `cargo test -p nepl-core --test effects loader_does_not_mark_user_core_mem_path_by_suffix -- --nocapture`: `1 passed`
- `cargo check -p nepl-core`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/raw_body_precheck.n.md --no-tree -o tmp/raw-memory-capability-raw-body-precheck.json -j 1`: `total=6`, `passed=6`
