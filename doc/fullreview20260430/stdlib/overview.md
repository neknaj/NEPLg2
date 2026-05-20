# stdlib overview review

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認対象

- `stdlib/index.n.md`
- `stdlib/core/**`
- `stdlib/alloc/**`
- `stdlib/std/**`
- `stdlib/platforms/**`
- `stdlib/nm/**`
- `stdlib/kp/**`
- `stdlib/tests/**`
- `nodesrc/test_stdlib*.js`
- `issues/index.json`

## 全体判定

stdlib は 2026-05-07 時点で、文字列、Vec、HashMap/HashSet、stdio、streamio、nm、TUI などの module split と source policy がかなり進んでいる。enum と match を使った状態管理も、HashMap/HashSet bucket、Vec storage、ANSI color/style、JSON value/escape、std/test assertion などで確認できる。

一方で、selfhost の基盤としてはまだ最終形ではない。`core/mem` と raw-memory-backed API は移行中で、collection の final non-Copy payload support、MemPtr/RegionToken の compiler-owned provenance、raw memory public surface の分離が未完である。2026-05-20 時点で旧 collection free/drop bug は Copy-only public surface と source policy により fixed になったが、non-Copy payload を多く持つ AST/HIR/diagnostic buffer を stdlib collection に載せる前に、後続 architecture issue を根本設計として解決する必要がある。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `stdlib/core` | char、Option/Result、test、traits、math、mem がある。`core/mem` は 1134 行。 | char/test/traits は良い。mem は安全性 critical な open issue が残る。 |
| `stdlib/alloc/string` | facade と storage/access/builder/search/slice/split/integer/float/utf8 に分割済み。 | selfhost に必要な文字列基盤は前進。raw boundary は migration 対象。 |
| `stdlib/alloc/collections` | Vec は大きく分割、HashMap/HashSet は enum bucket、List は Vec storage 化。Copy-only guard で旧 free/drop bug は閉じた。 | typed storage は前進。final non-Copy collection support は P1 architecture issue。 |
| `stdlib/alloc/hash/json/diag/io` | SHA-256 split、JSON typed value/escape、diag outcome、ByteBuf/ByteBuilder。 | 良いが JSON non-Copy payload と byte buffer owner は collection/drop問題に依存。 |
| `stdlib/std` | fs/env/stdio/streamio/test が分割。ANSI style は enum-first。 | I/O raw scratch buffer は ResourceIR/stdlib memory boundaryに依存。 |
| `stdlib/platforms/wasix/tui` | ANSI/TUI style/buffer/tty が分割。typed color helperへ寄せている。 | raw terminal state bufferは低レベル境界として監視対象。 |
| `stdlib/nm` | parser/htmlgen が分割、scanner/string helperを活用。 | 以前の raw aggregate detourは解消方向。nested boolean stateは必要に応じ改善。 |
| `stdlib/kp` | competitive helper は残るが raw memory use が多い。 | performance用として隔離しつつ ResourceIR proof を維持する必要あり。 |
| `stdlib/tests` | 各 module の `.n.md` が広い。 | `std/test` stdout/assert移行は進むが、`.n.md` contract issueは open。 |

## 重要な open issue

- `ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543`: final non-Copy collection payload support。
- `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`: core mem の raw address escape。
- `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47`: dealloc API が initialized storage の drop obligation を表さない。
- `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`: raw-memory-backed stdlib API の staged effect migration。
- `ISS-20260425T000000Z-RV-STDLIB-009-01749CCF`: 巨大 stdlib file の分割。

## selfhost readiness

stdlib は S1/S2 selfhost には十分使える範囲がある。特に `alloc/string/search`、`alloc/string/scanner`、`std/test`、`alloc/collections/vec`、diagnostic/string builder 系は selfhost lexer/parser/module graph を進める材料になる。

S3 以降では、`Vec<JsonValue>` や `Vec<SelfhostHirExpr>` のような non-Copy payload collection、diagnostic buffer、AST/HIR arena の owner/drop contract が blocker になる。現状の `Copy` 制約や raw-memory-backed exact boundary を無理に回避せず、`ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543` で collection drop/move/remove API と ResourceIR の責務を合わせてから使うべきである。

## ドキュメント上の注意

`stdlib/index.n.md` は現状ほぼ空で、stdlib 全体の module map と安全性契約を説明していない。今回のレビューでは doc 品質の詳細は `quality/docs.md` で扱うが、stdlib 利用者向けには core/alloc/std/platforms/nm/kp の入口と unsafe/raw boundary を一覧化する必要がある。
