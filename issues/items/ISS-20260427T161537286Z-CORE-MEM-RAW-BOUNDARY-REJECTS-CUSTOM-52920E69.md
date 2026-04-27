---
id: ISS-20260427T161537286Z-CORE-MEM-RAW-BOUNDARY-REJECTS-CUSTOM-52920E69
title: "core mem raw boundary rejects custom stdlib roots"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-27
updated: 2026-04-28
target: "nepl-core/src/typecheck.rs, nepl-core/tests/effects.rs, nepl-cli/tests/cli_output.rs"
---

# ISS-20260427T161537286Z-CORE-MEM-RAW-BOUNDARY-REJECTS-CUSTOM-52920E69: core mem raw boundary rejects custom stdlib roots

## 概要

`raw_body_memory_operations_allowed` は `/stdlib/core/mem.nepl` という directory 名を固定で見ていたため、CLI tests や user が `--stdlib-root` / `NEPL_STDLIB_ROOT` で `custom_stdlib/core/mem.nepl` のような alternate stdlib root を使うと、compiler-owned `core/mem` raw memory boundary まで `D3025` で拒否される。

## 対象

- `nepl-core/src/typecheck.rs, nepl-core/tests/effects.rs, nepl-cli/tests/cli_output.rs`

## 根拠

- `nepl-core/src/typecheck.rs` の `raw_body_memory_operations_allowed` は normalized path が `/stdlib/core/mem.nepl` で終わる場合だけ raw memory body を許可していた。
- `nepl-cli/tests/cli_output.rs` の stdlib root tests は temporary directory に `custom_stdlib/core/mem.nepl` を構成するため、この条件に一致しない。
- CLI failure では `mem_size` / `mem_grow` / `load_i32` / `store_i32` / `mem_copy` / `mem_move` など、audited core/mem helper が user raw body と同じ扱いで拒否された。

## 問題

compiler-owned raw memory boundary は stdlib root directory 名ではなく module suffix `core/mem.nepl` に紐づくべきである。directory 名固定のままだと、CLI の stdlib root override と SourceMap path が変わるたびに正当な core/mem helper が拒否される。

## 影響

explicit stdlib root で audited `core/mem` allocator / load-store helpers が compile できず、CI の stdlib-root tests が user code の検査前に失敗する。

## 修正方針

normalized path の suffix を `/core/mem.nepl` として判定し、stdlib root の directory 名に依存しない。`core/mem.nepl` 以外の user raw body は従来通り拒否する。

## 検証

`custom_stdlib/core/mem.nepl` の SourceMap regression を追加し、失敗していた CLI stdlib-root tests を実行する。

## 対応結果

- `raw_body_memory_operations_allowed` を `/stdlib/core/mem.nepl` 固定から `/core/mem.nepl` suffix 判定へ変更した。
- `nepl-core/tests/effects.rs` に `/tmp/custom_stdlib/core/mem.nepl` でも raw memory body が許可される regression を追加した。

## 後続課題

この修正は custom stdlib root を動かすための transitional fix であり、unsafe boundary の最終設計ではない。SourceMap path suffix だけで compiler-owned `core/mem` 特権を与える設計問題は `ISS-20260427T164425727Z-CORE-MEM-RAW-BODY-PRIVILEGE-IS-GRANT-043DAD95` に分離した。次の修正では loader/module identity に raw memory capability を持たせ、configured stdlib の `core/mem` だけを許可する。

## 実施した検証

- `cargo fmt --check`: pass
- `cargo test -p nepl-core --test effects`: `13 passed`
- `cargo test -p nepl-cli --test cli_output stdlib_root`: `4 passed`
- `trunk build`: pass
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
