# NEPLg2.1 compiler performance / cache design 2026-05-27

## 位置づけ

この文書は、NEPLg2.1 移行中の compile-time performance 改善方針を固定する。対象は Rust 実装の `nepl-core`、Web/WASM wrapper の `nepl-web`、Node test runner、stdlib prelude/import graph である。

2026-05-27 に Zenn の「試作段階における開発方針」を再確認した。現段階では後方互換よりも設計の正しさを優先する。ただし静的検査の正確性、source capability、Resource IR の安全性証明を削って速度を得ることはしない。timeout 延長、検査削除、coverage 削減は performance fix として扱わない。

## 目標

- 通常の 1 program compile は 0.5 秒未満にする。
- release WASM artifact を使う warm session では、最小規模の微小再compileを 10ms 未満にする。
- stdlib は、generic の具体化以外をできるだけ事前検査済み artifact として扱い、通常 compile では link / instantiate に近い形に寄せる。
- cache は純粋な query result として扱い、入力 hash、compiler version、target、profile、stdlib artifact hash を key にする。

## 判明した根本原因

### Resource IR が未到達 stdlib 関数まで検査していた

NEPLg2.1 の static-check 強化後、monomorphize が runtime helper や stdlib function を広く残し、Resource IR summary builder が entry から到達しない関数まで固定点計算していた。これにより、最小 program でも raw init summary / initialized check が数十秒規模になっていた。

対策として、Resource IR static check の直前で HIR call graph を entry から辿り、到達関数だけを残す。`CallIndirect`、曖昧な mangled prefix、raw LLVM body、raw wasm direct call のように call graph が閉じない場合は conservative-all に倒し、検査漏れを避ける。

### default prelude が allocator graph を暗黙 import していた

`std/prelude_base` は `core/traits/copy` を通じて `core/mem` 全体を読み、`core/mem` は allocator / raw memory / pointer wrapper まで引いていた。最小 program でも heavy stdlib graph が import され、typecheck と Resource IR の入力が大きくなっていた。

対策として、primitive `Copy` / `Clone` 定義を `core/traits/copy/primitive` に分離し、`core/traits/copy` facade は `MemPtr .T` impl のために `core/mem/types` だけへ依存する。さらに `core/mem/types` は compiler memory type definition と軽い metadata だけを持ち、bounds check は `core/mem/pointer/region` へ移した。これにより default prelude は `MemPtr` capability 互換を保ちながら allocator graph を読まない。

### Node/WASM test runner が毎回 full stdlib VFS を渡していた

`nodesrc/run_test.js` は local stdlib を毎回 JS object として WASM API へ渡していた。WASM 側はその object を走査し、`BTreeMap<PathBuf, String>` へ詰め直すため、compiler 本体の改善後も test runner 境界に overhead が残った。

対策として、`trunk build` artifact の mtime と local `stdlib/**/*.nepl` の newest mtime を比較し、artifact が新しければ bundled stdlib を使う。local stdlib が artifact より新しい場合だけ FS stdlib VFS を渡す。`run_test` の JSON timing には `stdlib_vfs_mode`、`stdlib_vfs_ms`、`wasm_call_ms`、`warmup_ms` を出し、どの層が支配的かを追えるようにした。

## 現在の測定値

測定日は 2026-05-27。対象は現在 branch の実装である。

| case | command / artifact | result |
|---|---|---|
| native minimal check | `target/debug/nepl-cli.exe --check -i tmp/minimal_perf.nepl --target wasm --stdlib-root stdlib` | elapsed 160ms、`resource_typecheck=5ms`、`resource_static_check=1ms` |
| native aggregate check | `target/debug/nepl-cli.exe --check -i tmp/perf_alloc_probe.nepl --target wasm --stdlib-root stdlib` | elapsed 166ms、`resource_typecheck=6ms`、`resource_static_check=1ms` |
| Web release cold minimal | `NEPL_RUN_TEST_SKIP_COMPILER_WARMUP=1` + `nodesrc/run_test.js` | `compile_ms=231`、`total_ms=257`、`stdlib_vfs_mode=bundled` |
| Web release warm minimal | `nodesrc/run_test.js` default warmup | `compile_ms=5`、`total_ms=254`、`warmup_ms=222` |
| Web release warm aggregate | `nodesrc/run_test.js` default warmup | `compile_ms=22`、`total_ms=247`、`warmup_ms=191` |

release WASM では、最小 program の warm compile が 10ms 未満になった。一方で aggregate/generic/stdlib-heavy case の微小変更を常に 10ms 未満にするには、source-level warmup だけでは不足する。次段階では CompilerSession と stdlib prechecked artifact が必要である。

## 実装済みの境界

- Resource IR 前の entry reachability pruning。
- unknown call graph では pruning を行わない conservative-all fallback。
- `Copy` / `Clone` primitive module と `core/traits/copy` facade の責務分離。
- `core/mem/types` の軽量化と `region_in_bounds` の pointer region module への移動。
- WASM wrapper の bundled stdlib lookup を `&'static str` source table + overlay source に分離。
- Node runner の stdlib VFS freshness 判定、warm compile、timing metadata。
- Resource IR の per-function timing instrumentation は環境変数 `NEPL_RESOURCE_PER_FUNCTION_TIMING` で明示した時だけ出す。
- `nepl-web` に `CompilerSession` wasm-bindgen class を公開し、WASM instance 内で bundled stdlib source table を保持する。
- Node runner は `CompilerSession` が利用できる場合に session API を優先し、JSON timing に `compiler_session` を出す。
- `nepl-web` build artifact に bundled stdlib content hash を埋め込み、Node runner の stdlib freshness 判定は hash を優先する。

## CompilerSession first checkpoint

2026-05-27 の first checkpoint では、公開 API の境界を先に session 化した。現在の `CompilerSession` は bundled stdlib source table の保持までを行い、parse / typecheck / Resource IR summary cache はまだ持たない。

この段階での目的は、Web / Node 側の呼び出し元を「1 compile call = stateless function」から「WASM instance 内の session」へ移すことである。これにより、後続の parsed stdlib module、public signature table、Resource IR summary template を同じ API に追加できる。

追加測定:

| case | command / artifact | result |
|---|---|---|
| Web release session minimal smoke | `nodesrc/run_test.js` default warmup | `compiler_session=true`、`compile_ms=3`、`wasm_call_ms=3`、`stdlib_vfs_mode=bundled` |
| Web release session aggregate smoke | `tmp/perf_alloc_probe.nepl` + default warmup | `compiler_session=true`、`compile_ms=16`、`wasm_call_ms=16`、`stdlib_vfs_mode=bundled` |
| Web release session cold minimal | `NEPL_RUN_TEST_SKIP_COMPILER_WARMUP=1` + `tmp/minimal_perf.nepl` | `compiler_session=true`、`compile_ms=160`、`wasm_call_ms=113`、`stdlib_vfs_mode=bundled` |

`CompilerSession` がまだ semantic cache を持たないため、aggregate case は 10ms 未満には届いていない。次の checkpoint では、stdlib parse/import/type arity/typecheck artifact を session に持たせる。

stdlib freshness は、同一 timestamp や process-local mtime cache の stale 判定を避けるため、artifact に埋め込まれた `fnv1a64` content hash と local stdlib tree hash の比較を優先する。旧 artifact で hash API が存在しない場合だけ mtime fallback を使う。

## 次段階の CompilerSession 設計

`CompilerSession` は、純粋な compiler query を process 内で保持する単位である。CLI では 1 process 1 session、Web / Node test runner では WASM instance 1 session とする。

session が持つ query cache は次の階層に分ける。

| query | key | value | invalidation |
|---|---|---|---|
| source text | canonical path + content hash | UTF-8 source | file / VFS overlay change |
| lex | source hash + lexer version | token stream | source text change |
| parse | token hash + parser version + type arity hint hash | AST module | source text or imported type arity change |
| import graph | canonical path + import directive hash | dependency edges / reverse edges | imported module public surface change |
| type arity | module public type decl hash | type arity table | public type declaration change |
| name/typecheck | module hash + dependency public surface hash + target/profile | typed HIR / diagnostics / trait table | dependency public surface or local source change |
| monomorphize | typed HIR hash + instantiation root set | monomorphized HIR | reachable root or generic instantiation change |
| resource summary | function HIR hash + source capability hash | Resource IR summaries | function body or capability change |
| codegen | monomorphized reachable HIR hash + target/profile | wasm / llvm fragment | reachable lowered HIR change |

各 query は入力値だけで決まり、FileSystem や StdIO を内部で読まない。host 依存の file read は CLI / Web wrapper 側で source text table に変換してから session へ渡す。

## stdlib prechecked artifact

stdlib artifact は、release build または test runner startup で作成する。

artifact に含めるもの:

- bundled stdlib source hash table。
- module import graph と reverse dependency graph。
- public type / trait / function signature table。
- generic function の body hash と type parameter/kind boundary。
- trait impl index と method signature compatibility result。
- Resource IR summary template。generic type substitution が必要な部分は template として残す。
- source capability table。capability span は source hash と path に結び、古い source へ流用しない。

通常 program compile では、stdlib module の parse / import / signature / trait impl / capability validation を再実行しない。entry source と overlay source だけを新規 query として処理し、stdlib generic instantiation と reachable codegen fragment だけを必要に応じて具体化する。

## incremental compile

微小変更では、changed source hash から reverse import graph を辿り、影響を受けた query だけを invalidation する。

MVP では次の順に実装する。

1. Web / Node に `CompilerSession` API を追加し、bundled stdlib source table を保持する。
2. `CompilerSession` に warm parsed stdlib module cache を追加し、entry source が変わっても stdlib parse/import/type arity/typecheck artifact を再利用する。
3. Resource IR summary を function hash 単位で cache し、entry から到達する changed functions だけを再計算する。
4. codegen fragment を function hash 単位にし、unchanged functions は index と signature table だけを再接続する。
5. diagnostic rendering は最後に行い、cache には typed diagnostic enum と source span だけを保持する。

10ms 未満の対象は、CompilerSession が warm で、stdlib artifact が current で、変更が entry source の一部または小さな user module に閉じる場合である。stdlib 自体を変更した直後は artifact refresh が必要なので、この budget の対象外とする。

## safety contract

- call graph が静的に閉じない場合は、performance より正確性を優先して conservative-all にする。
- raw memory boundary / compiler memory type capability / owner token capability は source hash と path に結び、別 source へ再利用しない。
- generic substitution 後の Resource IR summary は、type argument hash を key に含める。
- cache hit しても、diagnostics は現在の source map へ再投影できる span だけを表示する。
- release artifact の stdlib hash が local stdlib より古い場合は、bundled stdlib を使わず FS stdlib override に戻す。
- session が保持する bundled stdlib hash と local stdlib content hash が一致しない場合は、mtime に関係なく FS stdlib override に戻す。

## 関連 issue

- [ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5](../../issues/items/ISS-20260524T225852366Z-PER-PROGRAM-COMPILE-TIME-EXCEEDS-DEF-189918C5.md)
- [ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92](../../issues/items/ISS-20260527T050120000Z-COMPILER-SESSION-STDLIB-PRECHECK-CACHE-A71E4C92.md)
