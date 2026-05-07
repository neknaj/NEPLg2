# selfhost compiler overview

確認対象 commit: `caca505d fix(selfhost): model lexer raw modes with enums`

## 確認対象

- `stdlib/neplg2/README.md`
- `stdlib/neplg2/index.n.md`
- `stdlib/neplg2/cli/**`
- `stdlib/neplg2/core/**`
- `doc/neplg2/self_host_plan.md`
- `doc/neplg2/self_host_execution_plan.md`
- `doc/neplg2/compiler_diagnostics_redesign_plan.md`
- `issues/index.json`

前回レビュー本文は参照せず、現行ソース、現行 doc、issue、recent commit のみを根拠に確認した。

## 全体判定

`stdlib/neplg2` は self-host compiler の骨格と S1/S2 周辺の実装がかなり揃っている。CLI と core の責務分離、typed diagnostic code、lexer/parser/module graph の段階化は現在の開発方針に合っている。

ただし、selfhost 全面実装を開始できる状態ではない。S1 lexer/parser parity と S2 module graph/import resolver は進めてよいが、S3 typecheck 以降は ResourceIR、stdlib raw memory boundary、diagnostic taxonomy の未解決設計に強く依存する。`ty` / `builtins` / HIR range / mono instance absence / HIR expr id absence / resolver DefId absence / HIR expression payload は直近 main で typed payload 化された。さらに `caca505d` で lexer raw mode も enum 化されたため、次の焦点は ResourceIR 連携、stdlib owner model、lexer/model の回帰監視へ移る。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `stdlib/neplg2/cli` | argv parser、typed option、reporter、driver、file I/O 境界がある。 | CLI/core 分離は良い。argv parser の raw Vec access は stdlib API 不足の影響として残る。 |
| `stdlib/neplg2/core/infra` | span/text/diag/outcome/pipeline/options がある。diagnostic code は enum-first。 | S1/S2 には十分。S3+ 用 code taxonomy は追加が必要。 |
| `stdlib/neplg2/core/syntax` | token/lexer/module AST/parser がある。char literal、raw block、indent、directive を扱う。lexer raw mode は `SelfhostLexerRawMode` enum 化済み。 | parity と numeric sentinel 回帰監視を継続する。 |
| `stdlib/neplg2/core/module` | VFS loader、import spec、stdlib map、module graph がある。 | S2 の基盤として妥当。線形探索と duplicate path 仕様は HashMap/ID 設計前の制約。 |
| `stdlib/neplg2/core/resolve` | DefId、DefKind、scope binding table がある。DefId absence は `Option<SelfhostDefId>` 化済み。 | 名前解決の入口はある。parent scope / import / hoist は今後の拡張。 |
| `stdlib/neplg2/core/ty` | TypeId、TypeKind、TypeArena、構造比較がある。 | function type model と type record payload 分離はある。primitive/function flat field sentinel は fixed。 |
| `stdlib/neplg2/core/hir` | flat HIR module と expression/child/param table がある。child/param range は `Empty` / `Range`、expr absence は `Option<SelfhostHirExprId>`、expression payload は `SelfhostHirExprPayload` enum へ分離済み。 | expression 追加時に payload enum と match accessor の source policy を維持する必要がある。 |
| `stdlib/neplg2/core/resource` | move_state placeholder。 | S4 resource check は未実装。Rust ResourceIR 方針への追従が必要。 |
| `stdlib/neplg2/core/mono` | generic instance key / seed / range model がある。instance absence は `Option<SelfhostMonoInstanceId>` 化済み。 | key model は前進。cache 実装時に invalid ID sentinel へ退行しない監視が必要。 |
| `stdlib/neplg2/core/codegen` | WASM/LLVM placeholder。 | S5 backend は未着手。 |

## 追加・更新した issue

- `ISS-20260507T150754473Z-SELFHOST-TYPE-HIR-AND-BUILTIN-MODELS-8EBC822D`
  - HIR expression payload は `c5f93163` で variant enum 化済み。enum equality、builtin signature、type record、HIR range、mono instance absence、HIR expr id absence、resolver DefId absence も fixed。
- `ISS-20260507T151236784Z-SELFHOST-LEXER-RAW-MODES-AND-DIRECTI-B080723B`
  - `caca505d` で fixed。lexer raw mode と directive 分類の enum/match coverage を回帰監視する。

## selfhost readiness

今すぐ進めてよい範囲:

- Rust/selfhost lexer parity fixture の拡張。
- module AST と parser item model の拡張。ただし AST/HIR payload は sentinel に戻さない。
- VFS/module graph/import path map の S2 実装。
- diagnostic code enum と reporter JSON/human 出力の拡張。

まだ進めるべきでない範囲:

- S3 typecheck の本格実装。typed absence と resolver/type/HIR model 再設計が先。
- ResourceIR / borrow / drop の selfhost 実装。Rust 側 ResourceIR authority と stdlib memory model の最終形を参照する必要がある。
- non-Copy payload を大量に collection へ置く設計。`collection free` と raw-memory-backed API migration の残件に依存する。
- codegen backend の本実装。layout/mono/diagnostic/error surface が未確定。

## 結論

selfhost は「開始不可」ではなく「S1/S2 を限定的に進める段階」である。S3 以降を根本から正しく進めるには、現時点の placeholder や sentinel model を温存せず、enum / Option / typed range / variant payload によって静的検査が効く IR model へ再設計する必要がある。
