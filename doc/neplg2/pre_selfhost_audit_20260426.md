# NEPLg2.0 pre-self-host audit 2026-04-26

最終更新: 2026-04-26

---

## 目的

NEPLg2.0 self-host 実装を開始する前に、Rust 参照 compiler と stdlib の既知問題を Issue 管理へ集約する。
この文書は実装変更ではなく、開始条件と未解決リスクを固定するための監査記録である。

---

## 監査範囲

| 対象 | 件数 |
|---|---:|
| Rust source files (`nepl-core/src`, `nepl-cli/src`) | 35 |
| Rust `fn` | 791 |
| Rust `impl` | 47 |
| NEPL stdlib files | 70 |
| NEPL `fn` | 1263 |
| NEPL `trait` | 24 |
| NEPL `struct` | 48 |
| NEPL `enum` | 10 |
| `neplg2:test[skip]` | 53 |
| `unreachable` usage in stdlib | 25 |

確認は file inventory、function inventory、`TODO` / `placeholder` / `skip` / stub 検索、Rust test、stdlib doctest の失敗分類、既存 Issue との照合で行った。

---

## 追加 Issue

| Issue | 優先度 | 対象 | 理由 |
|---|---|---|---|
| [ISS-20260426T020000000Z-STRING-FIND-STUB-7C9A1E2B](../../issues/items/ISS-20260426T020000000Z-STRING-FIND-STUB-7C9A1E2B.md) | P1 | `stdlib/alloc/string.nepl` | `find` が常に `None` を返す |
| [ISS-20260426T020001000Z-SELFHOST-REQ-HASHKEY-4B6D8F10](../../issues/items/ISS-20260426T020001000Z-SELFHOST-REQ-HASHKEY-4B6D8F10.md) | P1 | `selfhost_req.rs`, `HashKey`, `HashMap` | user-defined key 要件 test が ignored かつ失敗 |
| [ISS-20260426T020002000Z-FUNCTION-NESTED-IGNORED-9D3C5A77](../../issues/items/ISS-20260426T020002000Z-FUNCTION-NESTED-IGNORED-9D3C5A77.md) | P2 | `nepl-core/tests/functions.rs` | 通る nested function regression が ignored のまま |
| [ISS-20260426T020003000Z-STDIO-SKIP-TESTS-2E6F0A4B](../../issues/items/ISS-20260426T020003000Z-STDIO-SKIP-TESTS-2E6F0A4B.md) | P1 | `stdlib/std/stdio.nepl` | self-host critical I/O の doctest skip が 27 件ある |
| [ISS-20260426T020004000Z-CLI-LIB-PLACEHOLDER-6B1D9E22](../../issues/items/ISS-20260426T020004000Z-CLI-LIB-PLACEHOLDER-6B1D9E22.md) | P2 | `nepl-cli/src/main.rs` | `--lib` が未実装 warning のみで成功パスに残る |
| [ISS-20260426T020005000Z-RUST-WARNING-DEBT-5F8E2C91](../../issues/items/ISS-20260426T020005000Z-RUST-WARNING-DEBT-5F8E2C91.md) | P3 | `nepl-core/src`, `nepl-cli/src` | warning debt が監査差分を隠す |

---

## 既存 Issue で管理済みの blocker

| Issue | 内容 |
|---|---|
| [ISS-20260425T000000Z-RV-CORE-007-5E3F920D](../../issues/items/ISS-20260425T000000Z-RV-CORE-007-5E3F920D.md) | codegen panic path |
| [ISS-20260425T000000Z-RV-CORE-009-58589A3F](../../issues/items/ISS-20260425T000000Z-RV-CORE-009-58589A3F.md) | Resource IR / move / borrow / drop |
| [ISS-20260425T000000Z-RV-CORE-010-05C6281D](../../issues/items/ISS-20260425T000000Z-RV-CORE-010-05C6281D.md) | name resolution 二重化 |
| [ISS-20260425T000000Z-RV-STDLIB-004-91534828](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-004-91534828.md) | collection free が要素 Drop を呼ばない |
| [ISS-20260425T000000Z-RV-STDLIB-005-EB6FBD85](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-005-EB6FBD85.md) | `read_all` が 4096 byte で切り捨てる |
| [ISS-20260425T000000Z-RV-STDLIB-006-673F4E12](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-006-673F4E12.md) | fs / cliarg の主要 doctest skip |
| [ISS-20260425T000000Z-RV-STDLIB-007-9CDFD520](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-007-9CDFD520.md) | `str` UTF-8 保証不足 |
| [ISS-20260425T000000Z-RV-STDLIB-012-C31422D8](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-012-C31422D8.md) | HashKey / Hasher capability と標準 trait の不整合 |
| [ISS-20260425T000000Z-RV-STDLIB-013-F256803D](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-013-F256803D.md) | collection doctest 群の drift |
| [ISS-20260425T000000Z-RV-STDLIB-018-663A986A](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-018-663A986A.md) | streamio doctest failure |
| [ISS-20260425T000000Z-RV-STDLIB-019-563743A1](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-019-563743A1.md) | collection doctest の末尾セミコロン |

これらは新規 Issue と重複しないよう、今回の監査では追加作成しない。

---

## 検証結果

### Rust compiler

`cargo test -p nepl-core -p nepl-cli` は通過した。
ただし ignored test と warning debt は残っている。

追加確認:

- `cargo test -p nepl-core --test functions function_nested -- --ignored`: pass。
- `cargo test -p nepl-core --test selfhost_req test_req_trait_extensions -- --ignored`: fail。主診断は `TypeInherentImplUnsupported`。
- `cargo check -p nepl-core -p nepl-cli`: pass。`nepl-core` 66 warnings、`nepl-cli` 1 warning。

### Stdlib doctest

`node nodesrc/tests.js -i stdlib --no-tree -o tmp/pre-selfhost-stdlib-review.json -j 4` は 379 件中 337 passed / 42 failed。
失敗は既存 Issue `RV-STDLIB-013`、`RV-STDLIB-018`、`RV-STDLIB-019` の範囲に分類できる。

主な失敗分類:

- `BTreeMap` / `BTreeSet` / `Queue` / `RingBuffer`: 値ブロック末尾セミコロンで `unit` になる。
- `Fenwick` / `SegmentTree`: stack extra values。
- `Vec sort`: overload mismatch。
- `HashMap` / `HashSet` string 系: runtime memory access out of bounds または return mismatch。
- `deserialize`: match arm 型不一致。

---

## self-host 開始条件

最低限、S0 source tree scaffold に入る前に次を判断する。

1. `ISS-20260426T020000000Z-STRING-FIND-STUB-7C9A1E2B` を先に直すか、S1 lexer では `find` を使わない方針を明記する。
2. `ISS-20260426T020003000Z-STDIO-SKIP-TESTS-2E6F0A4B` と既存 stdio / fs / cliarg issue の検証 harness を、S6 CLI parity より前に整える。
3. `ISS-20260426T020001000Z-SELFHOST-REQ-HASHKEY-4B6D8F10` は S3 type / collection 設計の前に、user-defined key を要求するかどうか決める。
4. 既存 P0 / P1 collection doctest failure は self-host stdlib dependency として先に green へ戻す。
5. `--lib` の契約は core WASM と CLI WASI を分ける checkpoint までに決める。

S0 の scaffold / pure data model だけなら、上記を未解決のまま開始できる。
ただし lexer / parser / CLI / collection に入る前に該当 Issue を閉じるか、self-host 実装側で使わない理由を `note.n.md` に記録する。
