# selfhost readiness review

確認対象 commit: `caca505d fix(selfhost): model lexer raw modes with enums`

## 判定

現段階で selfhost の実装を開始できるのは S1/S2 の限定範囲である。S3 以降を全面実装するには、Rust compiler の ResourceIR authority、diagnostic id 再設計、stdlib memory model、HIR/AST owner model が揃う必要がある。

remote main の `0fcc4839` により selfhost model の enum equality helper が direct match へ改善された。さらに `0ac34132` で builtin signature は arity enum へ移り、`4da7333` で type record は variant payload へ分離され、`6277239` で HIR child/param range は `Empty` / `Range` enum へ分離され、`b9e85f23` で mono instance absence は `Option<SelfhostMonoInstanceId>` へ移り、`8ff05570` で HIR expr absence は `Option<SelfhostHirExprId>` へ移り、`dc6b82bb` で resolver DefId absence は `Option<SelfhostDefId>` へ移り、`c5f93163` で HIR expression payload は `SelfhostHirExprPayload` enum へ分離された。`caca505d` では lexer raw mode も `SelfhostLexerRawMode` enum へ移った。これは S3 以降の基盤をよくする進捗だが、ResourceIR/stdlib memory model と diagnostics taxonomy がまだ固まり切っていないため、S3/S4 の全面実装開始条件はまだ満たしていない。

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
| typed IR sentinel / shared payload | `ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D` | fixed。enum equality、builtin signature、type record、HIR range payload、mono instance absence、HIR expr id absence、resolver DefId absence、HIR expression payload は fixed。 |
| enum equality numeric tag | `ISS-20260507T152220930Z-SELFHOST-ENUM-EQUALITY-HELPERS-LOWER-4E1FAA87` | fixed。親 typed IR sentinel issue の一部解決。 |
| builtin signature placeholder | `ISS-20260507T153554496Z-SELFHOST-BUILTIN-SIGNATURES-USE-ERRO-AEFFF7D4` | fixed。`SelfhostBuiltinSignature` の arity enum 化により fixed slot / `Error` placeholder は解消。 |
| type record flat payload | `ISS-20260507T154503761Z-SELFHOST-TYPE-RECORDS-USE-INVALID-TY-E984125D` | fixed。`SelfhostTypeRecord` の `Primitive` / `Function` payload 分離により primitive の `first_arg = -1` / invalid result は解消。 |
| HIR range empty sentinel | `ISS-20260507T155231568Z-SELFHOST-HIR-RANGES-ENCODE-EMPTY-STA-8B562D49` | fixed。`SelfhostHirChildRange` / `SelfhostHirParamRange` の `Empty` / `Range` payload 分離により `(-1, 0)` sentinel は解消。 |
| mono instance invalid ID | `ISS-20260507T155948337Z-SELFHOST-MONO-INSTANCE-IDS-USE-1-INV-434774DA` | fixed。未割当は `Option<SelfhostMonoInstanceId>::None` となり、`SelfhostMonoInstanceId(-1)` と validity helper は削除済み。 |
| HIR expr invalid ID | `ISS-20260507T160530818Z-SELFHOST-HIR-EXPRESSION-IDS-USE-1-IN-7A6D6ABC` | fixed。未割当は `Option<SelfhostHirExprId>::None` となり、`SelfhostHirExprId(-1)` と validity helper は削除済み。 |
| resolver DefId invalid ID | `ISS-20260507T161157719Z-SELFHOST-DEFINITION-IDS-USE-1-INVALI-E74DCE86` | fixed。未割当は `Option<SelfhostDefId>::None` となり、`SelfhostDefId(-1)` と invalid helper は削除済み。 |
| HIR expression flat payload | `ISS-20260507T161930297Z-SELFHOST-HIR-EXPRESSIONS-STORE-KIND--54E75EE3` | fixed。kind-specific field は `SelfhostHirExprPayload` enum に分離され、accessor は payload match を通す。 |
| lexer enum coverage | `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B` fixed | raw mode は `SelfhostLexerRawMode` enum へ移行済み。directive classifier も string helper を利用。 |
| selfhost partial implementation | `ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD` | S3/S4/S5 が未実装。 |
| raw memory backed stdlib API | `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84` | selfhost CLI/collection/stringが safe surface に乗り切っていない。 |
| collection free/drop contract | `ISS-20260425T000000Z-RV-STDLIB-004-91534828` | non-Copy payload を AST/HIR/diagnostic buffers に置く前に必要。 |
| n.md test contract | `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` | Rust/selfhost 共通テストで stdout/assert report が必要。 |

## selfhost 実装方針

selfhost は Rust compiler の完全な移植ではなく、現在の設計方針に合わせて再設計するべきである。特に、Rust 側に残っている巨大 parser/backend file、panic API、legacy move/drop authority を selfhost に複製しない。

型安全とメモリ安全は必達であり、`i32` sentinel、自由文字列 code、unchecked raw pointer discipline を selfhost compiler 内部 model として固定しない。暫定実装を置く場合も、最終設計は enum / Option / typed owner / match coverage で検査できる形にする。

## 次の順序

1. lexer raw mode と directive classifier が enum/match から退行しないよう source policy を維持する。
2. HIR expression payload enum の regression policy を維持し、expression kind 追加時に flat field へ戻さない。
3. module graph と import visibility を HashMap/ModuleId/SourceId table へ移行する。
4. Rust diagnostic redesign 後の taxonomy を selfhost diagnostic code へ反映する。
5. S3 typecheck を expression/type/HIR model の分割後に実装する。
6. ResourceIR は Rust 側の owner/cell/borrow/drop/effect authority を確認してから selfhost model を設計する。
