# selfhost readiness 総括

## 判断

現時点で selfhost の実装は、限定された範囲なら進められる。ただし full type checker、Resource IR、borrow checker、effect checker、codegen の最終仕様化を一気に進める段階ではない。

理由は、selfhost の中核となる static check と memory model が、Rust compiler 側の Resource IR と stdlib memory boundary に強く依存するためである。ここが未確定のまま selfhost を拡張すると、動くコードは増えても型安全とメモリ安全の保証が弱くなる。

## すぐ進めてよい範囲

### syntax と lexer state

lexer/parser の構文処理は進めてよい。特に raw block mode や directive state を `i32` から enum に変え、`match` の網羅性検査が効くようにする修正は優先してよい。

必要な修正:

- raw mode を `SelfhostRawMode` のような enum にする。
- pending raw mode も `Option<SelfhostRawMode>` で扱う。
- directive recognition を byte 列直書きではなく stdlib string/prefix API と directive enum に寄せる。
- `lex_raw_kind` と raw branch を `match` で処理する。

関連 issue:

- `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B`

### HIR と typed model

HIR payload、child range、param range、def id absence は直近で改善されている。この方向で AST/HIR/typed model の sentinel を消し、`Option` と enum payload へ寄せる作業は続行できる。

今後確認すること:

- ID 未割当を `0` や `-1` で表さない。
- variant payload を shared nullable field に戻さない。
- arity や kind を numeric tag で比較しない。
- `match` で全 variant を処理する。

### diagnostic registry

selfhost diagnostic ID は早めに設計してよい。Rust compiler 側の診断 ID 再設計に従い、selfhost 内部では enum registry を authority にする。文字列 ID は CLI、JSON、snapshot、doc などの外部境界だけで使う。

この作業は memory model と比較して干渉が小さく、将来の `.n.md` 共通 test とも相性が良い。

### module graph と test harness

module graph、VFS fixture、name resolution の deterministic test は進めてよい。`.n.md` test は Rust/selfhost 共通運用にする前提で、stdout assertion report と exit code の分離を使うべきである。

関連 issue:

- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD`
- `ISS-20260507T161416607Z-VFS-CROSS-FILE-DEFINITION-PATH-TREE--CCFBA9F9`

## まだ本格化を避ける範囲

### selfhost Resource IR

Resource IR の skeleton は設計してよいが、最終 model は Rust compiler 側 Resource IR と stdlib memory boundary を見て揃える必要がある。特に owner state、cell state、borrow state、effect boundary、drop obligation は、Rust 側と別設計にすると後で統合不能になる。

### selfhost type checker

type checker は AST/HIR の型表現整備までは進められる。しかし、resource-aware type checking、move/borrow/effect 診断、drop insertion を含む本体は、diagnostic ID と Resource IR の model を確定してから進めるべきである。

### allocator-heavy runtime

selfhost で string/Vec/hash map などを大きく使う実装は避けられない。ただし、`core/mem` と collections の安全境界が未確定のため、allocator-heavy な runtime 仕様を先に固定するのは危険である。

## selfhost blocker

| blocker | 影響 | 必要な解決 |
| --- | --- | --- |
| `core/mem` raw API | selfhost runtime が安全性を迂回できる | safe/raw boundary と provenance を compiler-owned にする |
| collection Drop obligation | AST/HIR/diagnostic containers の解放が不完全になる | collection free/dealloc と Resource IR drop plan を接続 |
| diagnostic ID 未同期 | Rust/selfhost の test と report が揃わない | enum registry と stable ID taxonomy を共通方針化 |
| `.n.md` return-value 運用 | selfhost/Rust 共通 test が弱い | stdout assertion report と exit code に統一 |
| lexer numeric state | enum/match の静的検査が効かない | raw/directive mode を enum 化 |

## 進捗状況

| selfhost 領域 | 状態 | 次の作業 |
| --- | --- | --- |
| `neplg2/cli` | 動く実装あり、継続整理 | diagnostic/reporting と test harness に合わせる |
| `neplg2/core/syntax` | 実装中 | lexer raw/directive state の enum 化 |
| `neplg2/core/ast` | 実装中 | sentinel と shared payload の有無を継続確認 |
| `neplg2/core/hir` | 改善済み | payload enum と range enum の回帰を policy で守る |
| `neplg2/core/ty` | 実装中 | type record variant を増やすときは enum/match を維持 |
| `neplg2/core/resolve` | 実装中 | `Option<SelfhostDefId>` model を維持 |
| `neplg2/core/check` | 未完成 | Rust Resource IR 方針に従って設計 |
| `neplg2/core/resource` | 未完成 | owner/cell/borrow/effect/drop model を Rust 側と揃える |
| `neplg2/core/codegen` | 未完成 | static check 完了後に本格化 |
| `stdlib/std/test` 連携 | 実装中 | stdout report を selfhost runner でも使う |

## 実装方針

selfhost は次の順番で進めるのが妥当である。

1. lexer/parser/model の enum/match 化を進め、数値 sentinel を減らす。
2. selfhost diagnostic enum registry を実装する。
3. `.n.md` test harness を stdout assertion report 前提に揃える。
4. Rust Resource IR と stdlib memory design の確定に合わせて selfhost Resource IR skeleton を作る。
5. type checker、borrow/effect/drop checker、codegen の順に進める。

この順序なら、静的検査の正確性を犠牲にせずに selfhost の実装面積を増やせる。
