# Project status report 2026-05-24

確認時点:

- 日時: 2026-05-24 08:43 +09:00
- branch: `docs/project-status-report-20260524`
- base commit: `addccd15 docs: split remaining vec noncopy transform work`
- remote sync: `main` は確認開始時点で `origin/main` と同期済み
- 確認範囲: `plan.md`, `todo.md`, `note.n.md`, `doc/`, `issues/`, `nepl-core/`, `stdlib/`, `stdlib/neplg2/`, `nodesrc/`, `web/`, `repo_metrics.ts`

## 全体結論

現在の主作業は「静的検査大規模修正」の Stage 6、特に non-Copy collection lifecycle を `Vec<T>` に対して Resource IR proof boundary へ接続する段階である。`push` / grow / free / clear / pop / replace / borrowed query は進んでいるが、`transform` と `sort` はまだ Copy-by-value、raw view、shallow swap 前提が残るため、親 issue は閉じられない。

この作業は selfhost 実装開始の前提である。selfhost 側は `stdlib/neplg2/` の分割構造がすでにかなり細かく、巨大な単一ファイルへは寄っていない。一方で、Rust 実装側は `resource/` と `typecheck/` の分割が進んでいる反面、`parser.rs`, `codegen_llvm.rs`, `loader.rs`, `codegen_wasm.rs`, `compiler.rs`, `types.rs` などがまだ大きく、次工程の「flat になっている Rust 実装のディレクトリ構造の階層化」の主要対象になる。

`repo_metrics.ts` で見た全体規模は 3,225 files / 533,032 lines / 3,315 test cases で、実装・検査・文書・issue が同時に大きいリポジトリである。従って、以後の作業は個別の不具合修正ではなく、責務境界、source policy、issue、doc を一体で更新する必要がある。

## 採用する開発方針

ユーザー指定の開発方針と Zenn 記事の方針に合わせ、次を前提にする。

- 間に合わせの例外や allowlist ではなく、仕様上の責務境界を作ってから実装する。
- core は純粋なコンパイラ実装に寄せ、CLI / filesystem / stdio / WASI などの環境依存は外側へ置く。
- raw pointer や collection slot の権限は public API の慣習に任せず、compiler-owned intrinsic / Resource IR / source capability で閉じる。
- `Option` / `Result` / enum / match / struct による明示的な診断と状態遷移を優先し、panic や暗黙の到達不能へ逃がさない。
- selfhost は Rust 実装をそのまま写すのではなく、Rust 側の責務分割で得た設計判断を反映した NEPLg2 実装として更新する。

## Issue 状況

`issues/index.json` は 1,243 件中 open 5 件、resolved 1,238 件である。open の内訳は stdlib 4 件、selfhost 1 件。

| priority | area | issue | 現状 |
| --- | --- | --- | --- |
| P1 | stdlib | `ISS-20260520T152218366Z-NON-COPY-COLLECTION-PAYLOAD-SUPPORT--A6A88543` | non-Copy collection payload support の親 issue。Vec の主要 lifecycle は進んだが transform/sort が未完了のため open 継続。 |
| P1 | stdlib | `ISS-20260523T051658073Z-VEC-NON-COPY-TRANSFORMS-NEED-BORROWE-A2D4AFE1` | `map` / `filter` / prefix / `partition` を borrowed predicate + MoveOut + output InitializeEmpty + rollback cleanup へ載せる次の主作業。 |
| P2 | selfhost | `ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD` | self-host compiler が部分実装に留まっている。静的検査と Rust 階層化の後に設計を更新してから再開する。 |
| P2 | stdlib | `ISS-20260425T000000Z-RV-STDLIB-009-01749CCF` | 巨大 stdlib ファイルの分割残件。最大級ファイルは 477 行前後まで下がっているが、全体の分割方針 issue として open。 |
| P2 | stdlib | `ISS-20260523T051715144Z-VEC-NON-COPY-SORT-NEEDS-BORROWED-COM-7B8AAE90` | sort の raw shallow swap / `Ord&Copy` comparison 前提を、borrowed comparison と slot swap lifecycle proof に置き換える後続作業。 |

## repo_metrics.ts による全体指標

実行コマンド:

```powershell
node --experimental-strip-types repo_metrics.ts --json tmp\project_status_repo_metrics_20260524.json
```

全体:

- total: 3,225 files / 533,032 lines / 27,174,156 bytes
- source lines: 193,583
- document lines: 137,293
- doc comment lines: 21,487
- test lines: 82,573
- test cases: 3,315
- binary skip: `NEPLg2.png`, `web/src/fonts/HackGenConsoleNF-Regular.ttf`

主要拡張子:

| ext | files | lines | source | doc/document | test | test cases |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `.rs` | 629 | 180,341 | 109,598 | 318 | 58,687 | 1,464 |
| `.nepl` | 615 | 67,023 | 33,860 | 21,169 | 7,100 | 548 |
| `.n.md` | 210 | 86,862 | 0 | 61,140 | 14,479 | 1,301 |
| `.md` | 1,382 | 111,553 | 0 | 76,153 | 2 | 2 |
| `.js` | 275 | 40,021 | 35,478 | 0 | 1,079 | 0 |
| `.ts` | 35 | 11,346 | 10,302 | 0 | 0 | 0 |

領域別:

| area | files | lines | source | doc_comment | document | test | test cases |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| source_tree | 1,224 | 202,380 | 146,424 | 20,820 | 4,883 | 16,633 | 984 |
| top_level_docs_tests | 382 | 61,353 | 1,111 | 657 | 30,223 | 16,658 | 1,220 |
| other | 1,619 | 269,299 | 46,048 | 10 | 102,187 | 49,282 | 1,111 |

tracked and unignored file の上位ディレクトリ規模は、`nepl-core` 166,981 lines、`issues` 107,775 lines、`stdlib` 70,205 lines、`nodesrc` 40,810 lines、`tests` 37,257 lines、`doc` 21,620 lines である。

## 静的検査 / Resource IR

進捗:

- `Vec<T>` の `push` / grow / free / clear / pop / replace / query は、Resource IR の collection slot lifecycle proof boundary へ接続済み。
- `collection_slot_borrow_ref<T>` が compiler-owned typed primitive として追加され、source capability、typecheck、Resource IR lowering、coverage、Wasm/LLVM precheck へ接続されている。
- borrowed query API として `count_ref`, `find_index_ref`, `any_ref`, `all_ref` が追加され、non-Copy payload を値として取り出さず `(&T)->bool` の scope 内観測に制限している。
- collection slot return/path summary は alias-aware に進み、`Vec<DropPayload>` の push/grow/return summary 後に initialized slot state を保持する方向へ進んだ。
- summary performance は、重い regression が約 84s から約 39s / 37s へ改善された記録が `note.n.md` に残っている。

残件:

- `Vec transform` は未完了。`map` / `filter` / prefix / `partition` を個別証明器へ散らさず、borrowed predicate observation、slot `MoveOut`、output `InitializeEmpty`、discard actual drop、rollback cleanup を扱う generic Resource IR proof engine へ載せる必要がある。
- `Vec sort` は未完了。raw shallow swap と `Ord&Copy` comparison 前提を、borrowed comparison と slot swap lifecycle proof に置き換える設計が必要。
- source policy 側に現在のドリフトがある。これは Stage 6 の完了条件とは別に、静的検査大規模修正の入口で潰すべき警告である。

`node nodesrc/run_source_policy_regressions.js --warn-only` の現状警告:

- `stdlib declaration doctest gaps increased: 1058 > 1032`
- `source_capability/rule.rs has 262 lines; responsibility split limit is 240`
- `collection_slot_event_target.rs must be monitored by resource responsibility line limits`
- `nepl-core/src/codegen_wasm.rs has 2530 lines; responsibility freeze limit is 2525`
- `TypeDiagnosticCode::CollectionSlotLifecycleBoundaryRestricted must appear exactly once in ALL_DIAGNOSTIC_CODES`
- `vec/mutation/pop.nepl doctest count changed` (`5 !== 3`)

## Rust 実装

`nepl-core` は現行 compiler の中心であり、tracked line count で最大の領域である。`resource/` は 451 files まで細分化されており、static check / ownership / collection slot proof の局所化はかなり進んでいる。一方で、root 直下または大きなサブシステムには flat な巨大ファイルが残っている。

最大級 Rust ファイル:

| path | lines | 判断 |
| --- | ---: | --- |
| `nepl-core/src/parser.rs` | 4,162 | syntax parser の階層化対象。selfhost parser 設計の参照元にもなるため、先に責務を切るべき。 |
| `nepl-core/src/codegen_llvm.rs` | 4,058 | backend 固有 lowering / runtime binding / diagnostic を分ける対象。 |
| `nepl-core/src/loader.rs` | 2,624 | module loading と source/package boundary の分離対象。 |
| `nepl-core/src/codegen_wasm.rs` | 2,529 | source policy freeze limit 2,525 を超過中。小手先ではなく backend 責務分割が必要。 |
| `nepl-core/src/compiler.rs` | 2,373 | orchestration と phase boundary の分割対象。 |
| `nepl-core/src/typecheck/prefix_check.rs` | 2,265 | prefix/typecheck rule 分割対象。 |
| `nepl-core/src/types.rs` | 2,264 | type model の module split 対象。 |
| `nepl-core/src/typecheck/driver.rs` | 1,703 | typecheck orchestration の階層化対象。 |
| `nepl-core/src/diagnostic_codes.rs` | 1,535 | diagnostic registry と個別 domain code の分割対象。 |
| `nepl-core/src/monomorphize.rs` | 1,412 | monomorphization pipeline の責務分割対象。 |

次工程の Rust 階層化では、Resource IR でできているような「policy が監視できる単位」まで切ることが重要である。単にファイルを短くするだけでは、selfhost 設計に移すべき phase boundary が見えない。

## stdlib

`stdlib/` は 609 tracked files / 70,205 tracked lines で、NEPL 実装と doctest が同じ木にある。`repo_metrics.ts` では `.nepl` が 615 files / 67,023 lines / 548 test cases と出ており、source_tree 内の doc comment 20,820 lines の大部分を担っている。

最大級 stdlib/selfhost ファイル:

| path | lines | 判断 |
| --- | ---: | --- |
| `stdlib/core/result.nepl` | 477 | core API の説明と doctest が厚い。分割する場合は Result の variant 操作、unwrap 系、predicate 系などの意味境界が必要。 |
| `stdlib/core/mem/pointer/region.nepl` | 467 | memory/pointer domain の安全境界に関わるため、単純なサイズ削減ではなく責務境界で切る対象。 |
| `stdlib/tests/vec.n.md` | 463 | Vec の回帰テスト集約点。transform/sort 修正時に増えやすい。 |
| `stdlib/std/stdio/read/buffer.nepl` | 444 | stdio surface。core から分離された環境依存側として維持する。 |
| `stdlib/std/env/cliarg/raw.nepl` | 430 | CLI/env surface。selfhost CLI 設計との対応を見る対象。 |
| `stdlib/alloc/collections/vec/query/predicate.nepl` | 359 | borrowed query 追加済み。transform engine への足場。 |
| `stdlib/alloc/collections/vec/types.nepl` | 359 | Vec owner/storage 型の中心。non-Copy lifecycle の設計変更と連動する。 |

`ISS-20260425T000000Z-RV-STDLIB-009-01749CCF` はまだ open だが、過去の streamio 系 facade split のように分割済みの領域も多い。今後の stdlib 分割は「巨大だから切る」ではなく、Resource IR と source policy が証明したい概念に合わせて切るべきである。

## selfhost

`stdlib/neplg2/` は現時点で、`cli`, `core/infra`, `core/syntax`, `core/hir`, `core/ty`, `core/proof`, `core/module`, `core/mono` などに分かれている。最大級でも 436 lines 程度で、Rust 実装のような 2,000-4,000 lines 級の flat file にはなっていない。

進捗:

- syntax parser、AST、HIR、type/proof/module/mono の基礎的なファイル配置は存在する。
- CLI args、diag code、module graph、stdlib map、proof solver など、コンパイラとして必要な概念はすでに名前を持っている。
- selfhost issue は resolved ではなく、実コンパイラとしてはまだ部分実装に留まる。

次の判断:

- Rust 実装の階層化で、parser / loader / compiler orchestration / diagnostics / backend の責務境界を確定する。
- その境界を `doc/self_host.md` と selfhost 関連 doc へ反映する。
- その後、`stdlib/neplg2/` の既存分割へ実装を足す。Rust の巨大ファイル構造を selfhost に持ち込まない。

## NEPLg3

AGENTS.md では `/stdlib/neplg3/` に selfhost compiler を作る方針が記載されている。CLI は WASI、Core は WASI なしの WASM とし、Rust 側の `nepl-cli/src/main.rs` と `nepl-core/*.rs` の責務差に近い分割を目指す。

現時点では、主作業は NEPLg2 の静的検査と non-Copy collection lifecycle であり、NEPLg3 は設計上の参照点として扱う段階である。NEPLg3 を実装へ進める前に、NEPLg2 selfhost の設計更新と実装開始順序を確定する必要がある。

## web / playground / tooling

`web/`, `nepl-web/`, `nepl-web-playground/`, `nodesrc/` は、compiler build、doctest、source policy、playground 実行のための実用基盤である。`nodesrc` は tracked 40,810 lines あり、source policy regression、issue 管理、doctest 実行、比較ツールを含む。

現状の優先順位:

- `nodesrc/issues.js check --dir issues` は issue 整合性の基本ゲートとして維持する。
- `nodesrc/run_source_policy_regressions.js --warn-only` は静的検査の責務境界ドリフト検出として、Stage 6 中も定期的に見る。
- `repo_metrics.ts` は、巨大化・doc/test/source の増減を commit 単位で見る基盤として有効である。
- playground / editor 周辺は todo に残件があるが、現在の主作業ではない。

## todo.md との対応

`todo.md` には self-host doc comment boilerplate、NEPLg3 migration、playground terminal/editor/mobile fixture、tutorial rewrite などの未来作業が残っている。現在の主作業である non-Copy collection lifecycle / static check Stage 6 とは粒度がずれている項目もある。

今回の確認では `todo.md` は変更しない。Stage 6 と Rust 階層化の完了後、実際に次へ着手する単位へ整理するのがよい。

## 検証状況

通過:

- `git pull --ff-only`: already up to date
- `cargo check -p nepl-core`
- `node nodesrc/issues.js check --dir issues`
- `node --experimental-strip-types repo_metrics.ts --json tmp\project_status_repo_metrics_20260524.json`
- `git diff --check`

警告あり:

- `node nodesrc/run_source_policy_regressions.js --warn-only` は exit 0 だが、source policy warning が 6 件ある。これは current main の観測事項として扱い、次の静的検査作業で潰す。

## 推奨する次工程

1. source policy drift を先に修正する。特に `collection_slot_event_target.rs` の resource responsibility 監視漏れ、`TypeDiagnosticCode::CollectionSlotLifecycleBoundaryRestricted` の registry 漏れ、`codegen_wasm.rs` freeze limit 超過は、後続の大規模修正の信頼性に直結する。
2. `ISS-20260523T051658073Z-VEC-NON-COPY-TRANSFORMS-NEED-BORROWE-A2D4AFE1` に着手する。`filter` / prefix / `map` / `partition` を generic Resource IR proof engine へ載せる。
3. transform engine が固まってから、`ISS-20260523T051715144Z-VEC-NON-COPY-SORT-NEEDS-BORROWED-COM-7B8AAE90` へ進む。sort は borrowed comparison と slot swap lifecycle proof の設計 doc を先に作る。
4. non-Copy collection lifecycle の親 issue を close できる状態にしてから、Rust 実装の階層化へ進む。対象は `parser.rs`, `codegen_llvm.rs`, `loader.rs`, `codegen_wasm.rs`, `compiler.rs`, `types.rs`, `diagnostic_codes.rs`, `monomorphize.rs`。
5. Rust 階層化で得た phase boundary を selfhost 設計 doc へ反映する。Rust のファイル構造を直写しせず、NEPLg2 側の core/CLI/proof/module/diagnostic 分割に合わせる。
6. selfhost 実装を再開する。最初の実装単位は CLI ではなく、core parser -> HIR -> diagnostics -> proof/type boundary の最小閉路を作るのがよい。
