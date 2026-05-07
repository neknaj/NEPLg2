# selfhost readiness review

確認対象 commit: `31291b37 fix(core): add parser backend responsibility policy`

## 判定

現段階で selfhost の実装を開始できるのは S1/S2 の限定範囲である。S3 以降を全面実装するには、Rust compiler の ResourceIR authority、diagnostic id 再設計、stdlib memory model、typed IR sentinel 排除が揃う必要がある。

## 開始可能な作業

- Rust lexer parity に基づく token/lexer fixture の拡充。
- `#indent`、char literal、string/raw block、directive token の selfhost/Rust parity の継続確認。
- module parser の item model 拡張。ただし HIR/AST payload に numeric sentinel を増やさない。
- VFS/module graph/import path map の S2 整備。
- CLI args/reporter/driver の Rust CLI contract 追従。
- diagnostic code enum の S3+ taxonomy 追加。

## 保留すべき作業

- Typecheck 本実装。
- ResourceIR / borrow checker / drop insertion の selfhost 実装。
- non-Copy payload を多用する AST/HIR/diagnostic collection 設計。
- WASM/LLVM backend 本実装。
- `.n.md` 共通テストを return value contract に固定すること。

## blocker

| blocker | 関連 issue | 内容 |
|---|---|---|
| typed IR sentinel | `ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D` | resolver/type/HIR/mono/builtin の invalid sentinel と placeholder payload。 |
| lexer enum coverage | `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B` | raw mode と directive classifier が enum/match coverage 外。 |
| selfhost partial implementation | `ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD` | S3/S4/S5 が未実装。 |
| raw memory backed stdlib API | `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` | selfhost CLI/collection/stringが safe surface に乗り切っていない。 |
| collection free/drop contract | `ISS-20260425T000000Z-RV-STDLIB-004-91534828` | non-Copy payload を AST/HIR/diagnostic buffers に置く前に必要。 |
| n.md test contract | `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` | Rust/selfhost 共通テストで stdout/assert report が必要。 |

## selfhost 実装方針

selfhost は Rust compiler の完全な移植ではなく、現在の設計方針に合わせて再設計するべきである。特に、Rust 側に残っている巨大 parser/backend file、panic API、legacy move/drop authority を selfhost に複製しない。

型安全とメモリ安全は必達であり、`i32` sentinel、自由文字列 code、unchecked raw pointer discipline を selfhost compiler 内部 model として固定しない。暫定実装を置く場合も、最終設計は enum / Option / typed owner / match coverage で検査できる形にする。

## 次の順序

1. lexer raw mode と directive classifier を enum/match 化する。
2. resolver/type/HIR/mono/builtin の sentinel model を typed absence へ再設計する。
3. module graph と import visibility を HashMap/ModuleId/SourceId table へ移行する。
4. Rust diagnostic redesign 後の taxonomy を selfhost diagnostic code へ反映する。
5. S3 typecheck を expression/type/HIR model の分割後に実装する。
6. ResourceIR は Rust 側の owner/cell/borrow/drop/effect authority を確認してから selfhost model を設計する。
