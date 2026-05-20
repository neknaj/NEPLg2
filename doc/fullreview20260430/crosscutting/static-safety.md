# 静的安全性横断レビュー

## レビュー範囲

この文書は、NEPLg2 の型安全、メモリ安全、効果境界、Resource IR、診断 ID、selfhost 側の静的検査準備を横断して確認した結果です。

確認対象:

- Rust compiler: `nepl-core/src/compiler.rs`, `nepl-core/src/resource/**`, `nepl-core/src/diagnostic*.rs`
- stdlib memory surface: `stdlib/core/mem.nepl`, `stdlib/alloc/collections/**`, `stdlib/alloc/string/**`
- selfhost compiler model: `stdlib/neplg2/core/**`
- issue registry: `issues/index.json`
- CI status: `gh run list`

レビュー基準:

- 技術的負債を残さず、根本原因を修正する。
- 暫定設計を採用しない。
- 型安全とメモリ安全を必達とする。
- 数値や文字列ではなく enum を用いて静的検査が効く形にする。
- `match` による網羅性検査が効く分岐にする。

## 現状の到達点

Rust compiler の静的検査パイプラインは、現在の `check` 経路でも codegen 準備経路と同じ安全性ゲートを通る構造になっている。`check_module_with_source_map` が `prepare_module_for_codegen_with_source_map` を呼び、型検査、Resource IR monomorphize、Resource static check、drop elaboration bridge gate、drop insertion を同一経路で実行するため、`--check` だけが重要な検査を迂回する構造にはなっていない。

Resource IR 側は、所有、初期化状態、借用、効果、drop 計画を typed model として管理している。`ResourceId`, `StorageId`, `ResourceBlockId` の ID 型、`ResourceOp`, `EffectOp`, `CellState`, `OwnerState`, `BorrowState` などの enum が中心であり、状態を単なる `i32` や文字列で管理する設計からは離れている。

診断 ID も `DiagnosticCode` enum を中心にした階層化へ進んでいる。`Loader`, `Lexer`, `Parser`, `Resolve`, `Type`, `Effect`, `Resource`, `Backend` の領域別 enum があり、文字列表現は表示、シリアライズ境界として扱われている。`ALL_DIAGNOSTIC_CODES` とテストにより、重複と階層名の崩れを検出できる。

selfhost 側では直近の refactor で、HIR と name binding の未割当 sentinel が改善された。`SelfhostNameBinding.def_id` は `Option<SelfhostDefId>`、`SelfhostHirExpr` は共通メタデータと `SelfhostHirExprPayload` enum に分離され、子範囲や引数範囲も `Empty`/`Range` の enum へ寄せられている。これは開発方針に沿った進捗として評価できる。

## 未完了リスク

### `core/mem` と stdlib raw memory 境界

`stdlib/core/mem.nepl` には `alloc_raw`, `dealloc_raw`, `realloc_raw` と raw address を扱う API が残っている。`MemPtr<T>` と `RegionToken<T>` も存在するが、コンパイラ所有の provenance、初期化状態、drop obligation と完全には接続されていない。現在のコメントでも、無効ポインタや範囲外アクセスが未定義動作になる raw API として説明されている。

これは単なる実装漏れではなく、stdlib API と Resource IR の責務境界の設計問題である。安全 API と unsafe/raw API の境界、所有権の発行元、dealloc 前に必要な drop 義務、再配置後の old pointer の失効を静的検査へ接続する必要がある。

関連 open issue:

- `ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04`
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`
- `ISS-20260427T152954558Z-CORE-MEM-EXPOSES-RAW-ADDRESS-ESCAPE--4185EA5D`
- `ISS-20260427T164432612Z-CORE-MEM-DEALLOC-APIS-DO-NOT-ENCODE--204F1F47`
- `ISS-20260427T204839136Z-STDLIB-RAW-MEMORY-BACKED-APIS-REQUIR-E503CD84`

### collections の drop obligation

2026-05-20 現状追記: 旧 issue `ISS-20260425T000000Z-RV-STDLIB-004-91534828` は、現行 public surface を Copy-only に閉じ、横断 source policy で constructor / update / observer / cleanup / owner recovery / storage view を監視する closure audit により fixed になった。unsupported non-Copy payload を collection に入れて storage-only `free` へ到達する入口は閉じている。

ただし、所有要素を持つ collection の破棄順序、panic 時の部分初期化、再確保時の move/drop を Resource IR にどう見せるかは final non-Copy support として未完了である。これは `ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543` で扱う。

関連 issue:

- `ISS-20260425T000000Z-RV-STDLIB-004-91534828`: fixed
- `ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543`: open

### selfhost lexer の数値状態

レビュー中に remote main へ `caca505d fix(selfhost): model lexer raw modes with enums` が入り、raw block mode と pending raw mode は `SelfhostLexerRawMode` enum で扱うように修正された。`lex_raw_kind` と `lex_raw_mode_is_active` は `match` を使い、numeric sentinel に戻らない source policy regression も追加された。

また `lex_starts_with_indent_directive` は `str_starts_with_at` を使う形へ改善され、byte ごとの手書き比較ではなくなった。この領域は未解決 blocker から回帰監視対象へ移った。

関連 issue:

- `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B`: fixed

### Resource IR と selfhost 静的検査の接続

Rust compiler 側の Resource IR は安全性の中核として進んでいるが、selfhost 側で同等の静的検査を実装する準備はまだ部分的である。selfhost の AST/HIR/型表現は enum 化が進みつつあるが、Resource IR、borrow lifetime、effect boundary、drop obligation を selfhost 側で実装するための最終モデルは未完了である。

ここで妥協すると selfhost は「動くが型安全とメモリ安全が検査されない compiler」になるため、parser や syntax の前進とは別に、Resource IR と静的検査の設計を Rust 側と揃える必要がある。

## 進捗状況

| 領域 | 状態 | 根拠 |
| --- | --- | --- |
| Rust compiler `--check` 経路 | 実装済み、継続検証中 | codegen 準備と同じ Resource IR gate を通る |
| Resource IR model | 実装中だが中核は成立 | typed ID と enum 状態で owner/cell/borrow/effect を表現 |
| drop elaboration bridge | 実装中 | bridge gate と drop insertion が pipeline に入っている |
| diagnostic code model | 実装済み寄り、selfhost 同期は未完了 | enum registry はあるが open issue が残る |
| stdlib `core/mem` 安全境界 | 未完了 P1 | raw API と compiler-owned provenance の接続が未完了 |
| collections drop obligation | 旧 bug fixed / final support 未完了 P1 | Copy-only guard は成立。non-Copy payload lifecycle は後続 issue |
| stdlib string/Vec 分割 | 進行済み、継続レビュー必要 | facade と submodule 分割は進んだ |
| selfhost HIR payload model | 改善済み | payload enum と `Option` 化が入った |
| selfhost lexer state | 解決済み、回帰監視 | `SelfhostLexerRawMode` enum と `match` 化、source policy 追加済み |
| selfhost Resource IR/static check | 未着手から設計段階 | Rust 側モデルへの追従が必要 |

## 判断

現時点で Rust compiler の静的検査基盤は、開発方針に沿った方向へ大きく改善されている。一方で、メモリ安全の最重要境界は stdlib `core/mem` と collections の API 設計に残っている。ここを残したまま selfhost の高度な実装へ進むと、selfhost 側が安全性を表現できない API に依存することになる。

したがって、次の優先順位は次の通りである。

1. `MemPtr<T>`/`RegionToken<T>`/raw API の authority を Resource IR に接続する。
2. collection の `Drop`/free/dealloc obligation を型と Resource IR で表現する。
3. selfhost lexer の raw/directive state が numeric sentinel へ戻らないよう source policy を維持する。
4. selfhost Resource IR と static check のモデルを Rust 側診断 ID 設計と整合させる。
5. CI の source policy を warn-only から必須 gate へ移行できる状態にする。
