# NEPLg2 stdlib full review 2026-04-26

最終更新: 2026-04-26

## 目的

stdlib の全体像を、実行テスト、静的検索、既存 Issue 台帳との照合で確認し、未追跡の問題を Issue として登録する。
特に、過去の compiler bug を避けるために残った不自然な書き方、無駄な中間変数、scaffold のまま公開されている API、コメント品質の劣化を確認する。

## 確認範囲

| 項目 | 件数 |
|---|---:|
| `stdlib/**/*.nepl` | 88 |
| stdlib NEPL source lines | 29955 |
| stdlib / stdlib-facing `.n.md` | 79 |
| `fn` | 1169 |
| `trait` | 25 |
| `struct` | 49 |
| `enum` | 10 |
| `neplg2:test[skip]` | 53 |
| `#intrinsic "unreachable"` | 26 |
| boilerplate comment marker | 393 |

対象は `stdlib/alloc`、`stdlib/core`、`stdlib/std`、`stdlib/features`、`stdlib/platforms`、`stdlib/kp`、`stdlib/nm`、`stdlib/neplg2`、`stdlib/neplg3` と、対応する `stdlib/tests` / `tests/stdlib` の doctest である。

## 実行確認

- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/stdlib-full-review-current.json -j 4`: `total=404`, `passed=404`, `failed=0`, `errored=0`
- `node nodesrc/tests.js -i tests/stdlib/streamio.n.md --no-tree -o tmp/stdlib-streamio-pipe-cast-workaround.json -j 1`: `total=13`, `passed=13`, `failed=0`

Rust 側は変更していないため、このレビュー追加 commit では `trunk build` を要求しない。Rust compiler / web dist を変更した場合は `doc/agent_handoff_20260426.md` の検証方針に従う。

## 既存 Issue として扱う項目

| Issue | 内容 |
|---|---|
| `ISS-20260425T000000Z-RV-STDLIB-004-91534828` | collection free が要素の Drop を呼ばない |
| `ISS-20260425T000000Z-RV-STDLIB-006-673F4E12` | fs / cliarg の主要 doctest skip |
| `ISS-20260425T000000Z-RV-STDLIB-007-9CDFD520` | `str` の UTF-8 保証不足 |
| `ISS-20260425T000000Z-RV-STDLIB-009-01749CCF` | 巨大 stdlib ファイルの分割 |
| `ISS-20260425T000000Z-RV-STDLIB-010-BF35FCBB` | `Result` / `Option` unsafe helper の通常コード利用 |
| `ISS-20260425T000000Z-RV-STDLIB-012-C31422D8` | `HashKey` / `Hasher` capability と標準 trait の不整合 |
| `ISS-20260426T020003000Z-STDIO-SKIP-TESTS-2E6F0A4B` | stdio の skipped doctest |
| `ISS-20260426T021003000Z-MEM-BULK-COPY-41F6B8D2` | byte copy path の bulk memory copy 不足 |

これらは今回のレビューで再確認したが、重複 Issue は作らない。

## 新規 Issue

| Issue | 優先度 | 対象 | 要点 |
|---|---|---|---|
| `ISS-20260426T060156433Z-STRING-NUMERIC-PARSERS-WRAP-OVERFLOW-E952EC90` | P1 | `stdlib/alloc/string.nepl` | 数値 parser が overflow を `Result::Err` にせず wrap し得る |
| `ISS-20260426T060223863Z-BYTEBUF-CONVERSIONS-HIDE-ALLOCATION--3BF03711` | P2 | `stdlib/alloc/io.nepl` | ByteBuf / str 変換が allocation failure を空値へ潰す |
| `ISS-20260426T060250100Z-JSONVALUE-STORES-STRUCTURED-JSON-PAY-8494C374` | P2 | `stdlib/alloc/encoding/json.nepl` | `JsonValue` の structured payload が raw `i32` handle |
| `ISS-20260426T060311796Z-SHA256-MODULE-PUBLISHES-BUFFERING-SC-F4601536` | P2 | `stdlib/alloc/hash/sha256.nepl` | `sha256_finalize` が digest ではなく buffer を返す scaffold |
| `ISS-20260426T060333140Z-TUI-BOX-HELPERS-RELY-ON-CALLERS-TO-A-2F61EDB2` | P2 | `stdlib/platforms/wasix/tui.nepl` | box helper が `cols < 2` を呼び出し側回避にしている |
| `ISS-20260426T060358681Z-STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE-2D7384D1` | P2 | 複数 stdlib file | doc comment policy に反する boilerplate comment が残る |
| `ISS-20260426T073020449Z-STDLIB-HAS-NESTED-IF-DECISION-TREES--8ADF5907` | P2 | `stdlib/` | match で表現すべき有限分岐が nested if として残る |

各 Issue は追加時点で Discord report を送信済み。

## 対応状況

- `ISS-20260426T060156433Z-STRING-NUMERIC-PARSERS-WRAP-OVERFLOW-E952EC90` は `stdlib/string-numeric-overflow` branch で修正し、u128/i128/i64/i32 の境界値 doctest を追加して verified にした。
- numeric parser 修正中に、`Result<i64,_>` の wildcard pattern が invalid wasm を生成する compiler issue `ISS-20260426T061837095Z-WILDCARD-RESULT-I64-PATTERN-CAN-GENE-C5C0C655` を追加した。追加時点で Discord report 済み。
- `ISS-20260426T060223863Z-BYTEBUF-CONVERSIONS-HIDE-ALLOCATION--3BF03711` は `stdlib/bytebuf-result-conversions` branch で修正し、`alloc/io`、`std/streamio`、`std/io`、`std/fs` の Result-returning 経路が allocation failure を成功値へ潰さないようにした。
- JSON typed value 修正中に、`json_escape` の固定文字分岐が `if` の深いネストになっていることを受け、stdlib 全体で match 化すべき有限分岐を監査する issue `ISS-20260426T073020449Z-STDLIB-HAS-NESTED-IF-DECISION-TREES--8ADF5907` を追加した。追加時点で Discord report 済み。
- 同じ JSON typed value 修正中に、`match` の整数 literal arm が parser で受理されない core issue `ISS-20260426T073513044Z-MATCH-CANNOT-USE-INTEGER-LITERAL-ARM-C0298FAB` と、`str` / `i32` unify により `json_string 0` が compile 成功する core issue `ISS-20260426T074114888Z-STR-UNIFIES-WITH-I32-AND-ACCEPTS-RAW-A824A1D7` を追加した。どちらも追加時点で Discord report 済み。

## compiler workaround の確認

`tests/stdlib/streamio.n.md` の `stream_writer_space_and_i64` には、旧 pipe 右辺 bug を避けるための `let two <i64> cast 2` が残っていた。
`ISS-20260426T023624387Z-PIPE-004372E8` は解決済みなので、`ISS-20260426T055122421Z-STREAMIO-DOCTEST-KEEPS-OBSOLETE-PIPE-F37DE397` として登録し、`|> writeln <i64> cast 2` に戻した。

今後も compiler bug 回避の痕跡は stdlib 側へ温存せず、compiler 側の resolved Issue と照合して自然な書き方へ戻す。
固定値や enum variant の有限分岐は、仕様の対応関係が読めるように `match` を優先し、`if` 連鎖が必要に見える場合は compiler 側の制約や未解決 bug として切り分ける。

## 修正順の提案

1. `STRING-NUMERIC-PARSERS-WRAP-OVERFLOW`: 入力値を誤って受理するため P1 として先に直す。
2. `BYTEBUF-CONVERSIONS-HIDE-ALLOCATION`: self-host artifact / binary I/O の失敗検出に関わる。
3. `JSONVALUE-STORES-STRUCTURED-JSON-PAY`: diagnostic JSON と metadata 生成の土台になる。
4. `SHA256-MODULE-PUBLISHES-BUFFERING-SC`: cache key / fingerprint で誤用される前に public API 契約を固定する。
5. `TUI-BOX-HELPERS-RELY-ON-CALLERS`: pure helper の edge case を固定し、runner / facade の検証を強くする。
6. `STDLIB-DOC-COMMENTS-STILL-CONTAIN-GE`: `RV-STDLIB-009` のファイル分割と合わせて段階的に進める。

## plan.md との差分

`plan.md` は変更していない。
今回のレビューは現行 stdlib の品質と self-host 前提 API の不足を Issue 台帳へ反映するもので、言語仕様本文の変更ではない。
