# プロジェクトリスクマップ

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 重要リスク

| 優先度 | リスク | 根本原因 | 現在の状態 | 次の確認 |
|---|---|---|---|---|
| P1 | 静的検査 authority が過渡期 | legacy move/drop と ResourceIR の責務統合がまだ完了していない。 | ResourceIR regression は増えているが、最終 authority の確認が必要。 | `rust-compiler/static-resource-check.md` で pass 順序と gate を確認する。 |
| fixed | `--check` が ResourceIR gate を通らない | check-only API が過去の stack overflow 回避として typecheck-only に分離されたまま残っていた。 | `3742a1a7` で `prepare_module_for_codegen_with_source_map` を共有し、ResourceIR gate と drop insertion bridge まで実行する regression を追加済み。 | Actions 完了結果を後続 checkpoint で確認する。 |
| fixed | public monomorphize API が panic し得る | compile pipeline は diagnostic-returning API に移ったが、古い convenience API が公開されたまま。 | `c58dd6e3` で unresolved trait call を diagnostic-returning Result として返すよう修正済み。 | Actions 完了結果を後続 checkpoint で確認する。 |
| P1 | `MemPtr` / `RegionToken` の owner model が未完 | pointer projection と storage owner が型レベルで完全分離されていない。 | open issue が core/stdlib に残る。最新 commit も region_ptr forged token 周辺の regression。 | `stdlib/core.md` と `crosscutting/static-safety.md` で扱う。 |
| P1 | stdlib raw-memory-backed API migration | raw memory capability が移行中の exact boundary として残る。 | Vec/string/io で module split が進むが、public safe API と internal raw API の最終分離は未完。 | `stdlib/alloc-string.md`、`stdlib/alloc-collections.md`、`stdlib/std-io-fs-env-test.md`。 |
| P1 | `.n.md` test / assert contract | main return value に依存した test は失敗時の情報が不足する。 | open issue `ISS-20260429T102425370Z...` が残る。 | `quality/tests.md` と `crosscutting/diagnostics-tests-docs.md`。 |
| P2 | selfhost が部分実装 | S0/S1/S2 の骨格はあるが、S3 以降の静的検査・stdlib owner model への依存が大きい。 | S1/S2 は制限付きで進行可能。全面移植は不可。 | `selfhost/readiness.md`。 |
| fixed | selfhost typed IR が invalid sentinel / shared payload を持つ | HIR expression が kind-independent placeholder payload を通常 field として持ち、各 model が sentinel を混ぜていた。 | `c5f93163` で HIR expression payload が `SelfhostHirExprPayload` enum へ分離され、`ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D` は resolved。 | 新しい expression / type / resolver state 追加時に flat field と sentinel helper を再導入しない。 |
| fixed | selfhost builtin signature が fixed slot を使う | builtin arity と payload が型で対応せず、unused slot を `Error` で埋めていた。 | `0ac34132` で `SelfhostBuiltinSignature` の arity enum 化が入り、`ISS-20260507T153554496Z-SELFHOST-BUILTIN-SIGNATURES-USE-ERRO-AEFFF7D4` は resolved。 | arity 追加時に enum/match coverage を維持する。 |
| fixed | selfhost type record が primitive/function共通 field を持つ | primitive record に `first_arg = -1` と invalid result TypeId を入れていた。 | `4da7333` で `SelfhostTypeRecord::Primitive` / `Function` の payload 分離が入り、`ISS-20260507T154503761Z-SELFHOST-TYPE-RECORDS-USE-INVALID-TY-E984125D` は resolved。 | type 拡張時に flat field sentinel へ退行しない。 |
| fixed | selfhost HIR range が `(-1, 0)` empty sentinel を使う | child/param range の empty state が numeric sentinel だった。 | `6277239` で `SelfhostHirChildRange` / `SelfhostHirParamRange` の `Empty` / `Range` payload 分離が入り、`ISS-20260507T155231568Z-SELFHOST-HIR-RANGES-ENCODE-EMPTY-STA-8B562D49` は resolved。 | range 拡張時に empty sentinel へ退行しない。 |
| fixed | selfhost mono instance が `-1` invalid sentinel を使う | monomorphize cache の未割当状態を instance ID 内の `index = -1` で表していた。 | `b9e85f23` で `SelfhostMonoInstanceId` は stable table index に限定され、未割当は `Option<SelfhostMonoInstanceId>::None` へ移った。`ISS-20260507T155948337Z-SELFHOST-MONO-INSTANCE-IDS-USE-1-INV-434774DA` は resolved。 | cache / lookup 実装時に invalid ID helper を再導入しない。 |
| fixed | selfhost HIR expr ID が `-1` invalid sentinel を使う | HIR expression の未割当状態を expr ID 内の `index = -1` で表していた。 | `8ff05570` で `SelfhostHirExprId` は stable table index に限定され、未割当は `Option<SelfhostHirExprId>::None` へ移った。`ISS-20260507T160530818Z-SELFHOST-HIR-EXPRESSION-IDS-USE-1-IN-7A6D6ABC` は resolved。 | HIR builder / lookup 実装時に invalid ID helper を再導入しない。 |
| fixed | selfhost resolver DefId が `-1` invalid sentinel を使う | name binding 追加前の未割当状態を DefId 内の `index = -1` で表していた。 | `dc6b82bb` で binding の DefId は `Option<SelfhostDefId>` になり、未割当は `None` へ移った。`ISS-20260507T161157719Z-SELFHOST-DEFINITION-IDS-USE-1-INVALI-E74DCE86` は resolved。 | resolver/import/hoist 実装時に invalid DefId helper を再導入しない。 |
| P2 | selfhost lexer raw mode が enum coverage 外 | raw mode が `i32` sentinel、directive classifier が deep if chain。 | `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B` を追加。 | enum raw state と classifier source policy。 |
| P2 | 巨大 stdlib file split の残件 | stdlib 分割は進んだが open issue が残る。 | Vec/string/streamio/nm/stdio debug は分割済み。巨大 module policy warning は継続的に発見されている。 | `stdlib/overview.md` と `stdlib/tests.md`。 |
| fixed | Rust parser/backend の responsibility policy 不足 | typecheck/resource と違い responsibility source policy がなかった。 | `31291b37` で `parser_backend_responsibility_split_plan.md` と source policy が追加された。 | 実分割の継続確認。 |
| P2 | CI status 未確定 | latest main run が pending/in_progress で、連続 push による cancel が多い。 | latest `c5f93163` run `25508600937` は in_progress。直前の failure 観測 run では `tutorials-test` / `nm-compile` failure を確認し、tutorial / VFS tree failure は issue 化済み。 | `project/actions-status.md` を更新し、completed latest run の failure を確認する。 |

## 技術方針との対応

- 技術的負債を残さない: open issue は safety-critical な根本設計に集中しており、回避実装で閉じるべきではない。
- 後方互換不要: `MemPtr` や raw-memory-backed API は互換性維持より役割分離を優先すべき。
- 暫定の雑設計禁止: ResourceIR と stdlib memory model の二重 authority を固定化しない。
- 静的検査の正確性必須: owner/cell/borrow/effect/drop は数値や文字列ではなく enum / typed state で保持し、match の網羅性検査を活用する。

## 監視すべき regression

- ResourceIR coverage が diagnostic を覆い隠し、owner/cell/borrow の根本違反が別エラーになる。
- `--check` が再び typecheck-only へ戻り、`resource.*` diagnostic の回帰監視に使えなくなる。
- public compiler API が diagnostic ではなく panic へ戻る。
- parser/backend の巨大 file が selfhost 実装へそのまま移植される。
- selfhost typed IR に invalid ID、empty range、flat payload field が戻る。
- selfhost builtin signature が fixed slot / `Error` placeholder / numeric arity tag へ退行する。
- selfhost type record が primitive/function 共通 flat field と invalid TypeId sentinel へ退行する。
- selfhost HIR range が `Empty` / `Range` enum ではなく `(-1, 0)` sentinel へ退行する。
- selfhost mono instance absence が `Option` ではなく `SelfhostMonoInstanceId(-1)` sentinel へ退行する。
- selfhost HIR expr absence が `Option` ではなく `SelfhostHirExprId(-1)` sentinel へ退行する。
- selfhost resolver DefId absence が `Option` ではなく `SelfhostDefId(-1)` sentinel へ退行する。
- lexer/parser state が enum ではなく i32 mode や string sentinel に戻る。
- raw-memory-boundary capability が facade root や raw-free helper に戻る。
- collection observer が owner を値で消費する API に戻る。
- fallible collection update が error path で collection/item owner を失う。
- diagnostics code が自由文字列に戻り、Rust/selfhost の enum taxonomy とずれる。
- n.md doctest が stdout assertion report ではなく return value contract に固定される。

## 現時点の結論

進捗は大きいが、根本安全性の最終形はまだ未完である。今後の修正は、既存 API を少しずつ救済する方向ではなく、owner model、ResourceIR authority、stdlib safe surface、test contract を同じ設計へ収束させる必要がある。
