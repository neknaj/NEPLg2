# NEPLg2.0 self-host 実行計画

最終更新: 2026-04-26

---

## 1. 目的

この文書は [self_host_plan.md](./self_host_plan.md) の実装を進めるための運用計画である。
仕様・構造・成功条件は `self_host_plan.md` を正とし、この文書では branch、commit、merge、検証、Issue 更新の単位を固定する。

NEPLg2.0 self-host は長期作業になるため、作業途中の大きな branch を持たない。
`main` を常に統合済みの基準点にし、1 branch は 1 checkpoint に対応させる。

---

## 2. 基本規則

- 作業開始時は必ず `main` に戻り、直前作業 branch がある場合は `main` へ fast-forward merge してから新しい branch を作る。
- `main` へ直接 commit しない。例外は fast-forward merge のみ。
- 1 branch は 1 つの主目的と 1 つ以上の明示 Issue に対応させる。
- branch 内の一時的なテスト悪化は許容するが、commit と `main` merge は green checkpoint のみとする。
- 各 commit 後に `doc/progress_report_template.md` の形式で Discord へ報告する。
- `plan.md` は変更しない。計画との差異、作業結果、保留判断は `note.n.md` に書く。
- 未着手作業は `todo.md` に残し、完了したら削除または次の未着手に置き換える。
- Issue の正は `issues/items/*.md` とし、旧 `doc/review20260425/` は更新しない。
- Rust 側コンパイラ修正と self-host 実装は同じ `main` を基準に統合する。
- self-host 実装中に Rust 側の不具合を見つけた場合は、self-host 側の回避実装で隠さず Issue として提出する。

---

## 2.1 Zenn 方針 review gate

NEPLg2.1 self-host の各 checkpoint は、Zenn 記事 `https://zenn.dev/bem130/articles/1b352797de94e7` と `AGENTS.md` の方針を作業開始時と commit 前に確認する。

新しい issue、実装 slice、公開 API、diagnostic enum、Resource proof 境界、または既存 module の責務変更に着手する場合は、次を必須 gate とする。

1. Zenn 記事を再確認し、今回の slice に関係する方針を `note.n.md` または対応 issue に記録する。
2. 独立した subagent review を行い、設計、実装、ドキュメントコメント、source policy、検証方針を確認する。
3. Blocker は同じ branch 内で修正する。修正できない場合は、原因、影響、完了条件を持つ issue へ分離する。
4. Non-blocker は `note.n.md`、`todo.md`、または issue に残し、次 checkpoint の入口で再確認する。
5. source policy を追加または更新し、同じ方針違反が戻らないようにする。ただし、コメントを減らすための行数制限や説明削減の検査を入れてはならない。

subagent review では、少なくとも次を確認する。

- 詳細な review 入力、確認項目、指摘分類、`note.n.md` 記録形式は `doc/neplg2/self_host_zenn_review_checklist.md` に従う。
- review 依頼は `nodesrc/selfhost_zenn_review_packet.js` で生成した packet を土台にし、branch、base / head、committed / staged / unstaged / untracked に分けた変更 file list、accepted / fail-closed、検証欄、既存 warning、今回差分由来 warning、Zenn 記事 URL、Zenn 再確認日時、`AGENTS.md`、checklist、prompt authority を省かない。
- 失敗が `Option` / `Result` / enum diagnostic で表され、文字列や sentinel 値で分岐していないこと。
- `match` の網羅性検査が効く設計であり、公開 API が数値 tag や文字列 tag に依存していないこと。
- core / CLI boundary、parser / checker / HIR / Resource IR / backend boundary が混ざっていないこと。
- ドキュメントコメントが目的、契約、戻り値や error variant の条件、計算量、制約、典型例、現状の実装詳細と将来も守る契約の区別を説明していること。
- 高速化が静的検査を省略する方向ではなく、探索範囲、依存関係、artifact、cache key、事前検査済み summary の設計で行われていること。
- 暫定実装がある場合、その名前、妥協内容、fail-closed 範囲、解除条件が検索可能な形で記録されていること。

---

## 3. Branch 命名

| 種別 | 形式 | 例 | 用途 |
|---|---|---|---|
| self-host compiler | `selfhost/s<stage>-<topic>` | `selfhost/s0-source-tree` | `stdlib/neplg2/` 本体 |
| stdlib interface | `stdlib/<issue-topic>` | `stdlib/fs-write-api` | `std/fs`、`std/stdio`、`std/env` など |
| test / harness | `nodesrc/<topic>` | `nodesrc/selfhost-parity-runner` | Node.js 側の検証支援 |
| CI / workflow | `ci/<topic>` | `ci/selfhost-smoke` | GitHub Actions や test job |
| docs only | `docs/<topic>` | `docs/neplg2-selfhost-workflow` | 計画、README、記録 |
| regression fix | `fix/<issue-id-short>-<topic>` | `fix/rv-core-007-codegen-diagnostic` | 既存不具合の修正 |
| Rust compiler | `rust/<issue-id-short>-<topic>` | `rust/iss-core-parser-span` | `nepl-core` / `nepl-cli` 側の修正 |

`<topic>` は 2-5 語の kebab-case にする。
stage をまたぐ branch 名にしない。たとえば lexer と module loader を同時に進める branch は作らず、`selfhost/s1-lexer` と `selfhost/s2-module-vfs` に分ける。

---

## 4. Branch 手順

1. `git switch main`
2. 直前作業 branch が未 merge なら `git merge --ff-only <branch>`
3. `git status --short --branch` で clean を確認する
4. `git switch -c <branch>`
5. 実装、doc、Issue、todo、note を更新する
6. focused test を先に通し、最後に共通検証を通す
7. commit する
8. commit ごとに Discord へ報告する
9. `git switch main`
10. `git merge --ff-only <branch>`

branch は merge 後に再利用しない。
同じ領域の続きでも、次の checkpoint は新しい branch にする。

---

## 5. Rust 側 main 変更の取り込み

Rust 側コンパイラの修正は self-host の参照実装を変えるため、常に `main` を経由して取り込む。
self-host branch から Rust 側修正 branch へ直接 merge しない。

### 5.1 原則

- Rust 側修正は `rust/<issue-id-short>-<topic>` または `fix/<issue-id-short>-<topic>` branch で行う。
- Rust 側修正 branch は検証後に `main` へ merge する。
- self-host branch は `main` に入った Rust 側修正だけを取り込む。
- self-host branch が参照する Rust 挙動は、commit 時点の `main` の挙動として `note.n.md` に記録する。
- Rust 側修正により self-host parity fixture の期待値が変わる場合は、Rust 修正 commit 側で fixture と doc を更新する。

### 5.2 self-host branch が未 commit の場合

1. 変更中ファイルを確認する。
2. 未完了作業が小さければ一度 commit 可能な checkpoint へ縮める。
3. checkpoint にできない場合は作業を継続せず、必要に応じて `note.n.md` に中断理由を残す。
4. `main` へ戻り、Rust 側修正を merge する。
5. 新しい self-host branch を `main` から作り直す。

未 commit の大きな作業に Rust 側変更を重ねない。

### 5.3 self-host branch に commit がある場合

1. self-host branch の worktree を clean にする。
2. `main` に Rust 側修正を merge する。
3. self-host branch に戻る。
4. `git rebase main` で self-host commit を新しい `main` の上へ移す。
5. conflict が出た場合は、Rust 側の修正意図を優先して self-host 側の fixture / doc / implementation を合わせる。
6. rebase 後に focused test と共通検証をすべて再実行する。
7. `main` へは `git merge --ff-only <selfhost-branch>` で入れる。

`git merge main` による merge commit は使わない。
長期 branch を避け、rebase conflict が大きくなる前に checkpoint を切る。

### 5.4 Rust 側修正が self-host を壊した場合

Rust 側修正後に self-host fixture が失敗した場合は、次の順で判断する。

| 判定 | 対応 |
|---|---|
| Rust 側の挙動修正が正しい | self-host 側の期待値と実装を更新する |
| Rust 側の修正が regression | Rust 側 Issue を open に戻す、または新 Issue を作り、self-host branch は merge しない |
| 仕様が未確定 | `doc/neplg2/` に決定待ちを記録し、self-host 実装を進めない |
| self-host 側の仮実装が古い | self-host branch 内で修正し、note に Rust 側 commit との差異を記録する |

---

## 6. Rust 側 Issue 提出規則

self-host 実装中に見つかった問題は、Rust 側へ提出する前に分類する。

| 分類 | Issue area | Branch | 進め方 |
|---|---|---|---|
| Rust compiler の明確な bug | `core` または `cli` | `rust/<issue-id-short>-<topic>` | Rust 側を先に直して `main` へ merge |
| Rust compiler の panic / diagnostic 不備 | `core` | `rust/<issue-id-short>-diagnostic` | diagnostic 化と regression test を追加 |
| Rust 挙動が仕様未記載 | `core` または `doc` | `docs/<topic>` か `rust/<topic>` | 仕様決定を doc に残してから実装 |
| self-host だけの実装不足 | `selfhost` | `selfhost/s<stage>-<topic>` | Rust 側へ提出しない |
| stdlib interface 不足 | `stdlib` または `selfhost` | `stdlib/<topic>` | public API と doctest を先に整備 |
| test harness 不足 | `cli` または `selfhost` | `nodesrc/<topic>` | fixture / JSON / runner を追加 |

### 6.1 提出に必須の情報

Rust 側 Issue を作る場合は、`issues/items/*.md` に次を必ず書く。

- self-host の stage
- 最小再現 `.nepl` または `.n.md`
- Rust 側の実際の結果
- 期待する結果
- 影響する parity 条件
- suspect file が分かる場合は `nepl-core/src/...` または `nepl-cli/src/...`
- self-host 側で回避しない理由
- 検証予定

`node nodesrc/issues.js new` で作成し、手で本文を補足する。
作成後は `node nodesrc/issues.js index` と `node nodesrc/issues.js check` を通す。

### 6.2 blocker の扱い

Rust 側 bug が self-host の stage 完了条件を阻害する場合、self-host branch は次のどちらかにする。

- commit 可能な doc / fixture / Issue 提出だけを commit し、実装 commit は作らない。
- 実装途中なら checkpoint にせず中断し、Rust 修正 branch を先に作る。

blocker を避けるための self-host 限定 workaround は作らない。
workaround が必要に見える場合は、Rust 側の仕様または diagnostic を先に決める。

### 6.3 Rust 修正 commit の条件

Rust 側修正 commit は次を満たす。

- 既存 Rust compiler test、nodesrc test、または `.n.md` doctest のいずれかに regression test がある。
- self-host parity に関係する場合は、期待 JSON または fixture を同じ commit で更新する。
- Issue の `status`、`resolved`、`updated` を更新する。
- `note.n.md` に self-host への影響を書く。
- 共通検証を通す。

Rust 側修正が大きい場合は、原因特定 commit、regression test commit、実装修正 commit に分ける。
ただし、`main` へ merge する時点では全 commit が green checkpoint でなければならない。

### 6.4 self-host branch への復帰

Rust 修正を `main` に merge した後、self-host 作業へ戻る手順は次の通り。

1. `git switch main`
2. `git merge --ff-only <rust-branch>`
3. `git switch -c selfhost/s<stage>-<topic>` または既存 branch を rebase
4. Rust 修正で変わった fixture を確認する
5. focused self-host test を実行する
6. self-host commit を続行する

---

## 7. Commit 単位

commit は「review 可能で、単独で検証でき、戻す場合も意味が明確な単位」にする。
作業量ではなく、境界と検証可能性で切る。

| 種別 | 1 commit に含めるもの | 含めないもの |
|---|---|---|
| docs checkpoint | 計画、README、note、todo、Issue metadata | 実装コード |
| source tree scaffold | 空でない module skeleton、実行可能な最小 doctest、README | parser や backend の実装 |
| stdlib interface | public API、target 実装、doctest、関連 Issue 更新 | compiler core の移植 |
| harness | nodesrc tool、fixture、JSON 出力、doc | 言語仕様変更 |
| compiler data model | 型・AST・HIR などの表現と unit doctest | parser/checker の広範囲実装 |
| compiler pass slice | 1 pass の小さな機能、parity fixture、diagnostic | 次 stage の実装 |
| Rust compiler fix | 原因を示す regression test、`nepl-core` / `nepl-cli` 修正、self-host parity への影響記録 | self-host 側 workaround |
| issue close | 修正、回帰テスト、Issue `status/resolved/updated` 更新、note | 未検証の追加修正 |

commit の大きさは行数では判定しない。生成 index、Issue 移行、広範囲の機械移行、丁寧なドキュメントコメント整備のように、意味が 1 つで検証可能なら大きな commit でも許容する。
手書き実装が大きい場合は、行数ではなく責務境界を見る。data model、pure helper、public API、integration、diagnostic projection、source policy のどれが混ざっているかを確認し、review 可能な境界へ分割できるなら分割する。

---

## 8. Commit Message

基本形は次の通りにする。

```text
<scope>: <summary>
```

推奨 scope:

| scope | 対象 |
|---|---|
| `docs(neplg2)` | NEPLg2.0 self-host 計画 |
| `selfhost(s0)` | `stdlib/neplg2/` S0 |
| `selfhost(s1)` | lexer / parser |
| `selfhost(s2)` | module loader |
| `selfhost(s3)` | type / check |
| `selfhost(s4)` | HIR / resource / mono |
| `selfhost(s5)` | WASM backend |
| `selfhost(s6)` | CLI |
| `selfhost(s7)` | bootstrap comparison |
| `stdlib(fs)` | filesystem interface |
| `stdlib(stdio)` | stdio / stderr interface |
| `nodesrc(selfhost)` | self-host test harness |
| `ci(selfhost)` | self-host CI |
| `rust(core)` | `nepl-core` の参照実装修正 |
| `rust(cli)` | `nepl-cli` の参照実装修正 |

例:

```text
selfhost(s0): add compiler source tree skeleton
stdlib(fs): add result-returning write APIs
nodesrc(selfhost): add lexer parity runner
rust(core): fix parser span parity diagnostic
```

commit 本文が必要な場合は、主 Issue と検証を短く書く。

```text
Issue: ISS-20260426T010001Z-STDFS-WRITE-B7C4D923
Verify: node nodesrc/issues.js check; trunk build; playground editor 13/13
```

---

## 9. 共通検証

commit 前の共通検証は次を必須にする。

```bash
node nodesrc/issues.js check
git diff --cached --check
trunk build
node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-selfhost.json
```

JSON 出力は `caseCount`、`passedCount`、`failedCount` を確認する。
`trunk build` の既存 warning は記録のみとし、この作業で増えた warning は原因を確認する。

docs only の場合も、web/compiler artifact と Node CLI の前提を崩していない確認として `trunk build` と playground editor test を実行する。

---

## 10. 追加検証

| 作業種別 | 追加検証 |
|---|---|
| `issues/` 更新 | `node nodesrc/issues.js index`、`node nodesrc/issues.js check` |
| Markdown link 更新 | 関連 doc の link existence check |
| `stdlib/neplg2/` 更新 | `node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/selfhost-stdlib.json -j 2` |
| `std/fs` 更新 | read/write/invalid path の doctest または integration test |
| `std/stdio` 更新 | stdout/stderr split と error propagation の test |
| lexer/parser 更新 | Rust 実装との token / AST JSON parity |
| module loader 更新 | import alias、open import、cycle、missing file |
| WASM backend 更新 | binary section unit test と smoke execution |
| CLI 更新 | argv table test、exit code、diagnostic JSON、artifact output |

追加検証で失敗を見つけた場合は、原因が今回の作業範囲にあるかを切り分ける。
今回の範囲外の既存不具合なら Issue を作り、今回の commit へ混ぜない。

---

## 11. Stage 別 checkpoint

### S0: Source tree

| 順序 | Branch | 主 Issue | Commit 単位 | 完了条件 |
|---:|---|---|---|---|
| 1 | `selfhost/s0-source-tree` | `ISS-20260426T010000Z-SELFHOST-SOURCE-TREE-A1E5F24C` | `stdlib/neplg2/` root、README、top-level module skeleton | focused doctest が収集される |
| 2 | `selfhost/s0-infra-span-diag` | 同上 | `infra/span.nepl`、`diag.nepl`、`outcome.nepl` | span / diagnostic doctest |
| 3 | `nodesrc/selfhost-focused-tests` | 同上 | self-host focused test command の補助 | `stdlib/neplg2` の test 結果 JSON |

### S1: Lexer / parser

| 順序 | Branch | Commit 単位 | 完了条件 |
|---:|---|---|---|
| 1 | `selfhost/s1-source-text` | byte offset、line map、UTF-8 前提の source text | line/column doctest |
| 2 | `selfhost/s1-token-model` | Token / TokenKind / keyword table | token model doctest |
| 3 | `selfhost/s1-lexer-basic` | literal、identifier、operator、comment | simple token parity |
| 4 | `selfhost/s1-lexer-indent` | `#indent` と offside rule | indent fixture parity |
| 5 | `selfhost/s1-ast-model` | module / item / expr / typeexpr / pattern AST | AST construction doctest |
| 6 | `selfhost/s1-parser-items` | import / fn / let / struct / enum | item parser parity |
| 7 | `selfhost/s1-parser-expr` | call、match、if、block、type annotation | expr parser parity |

### S2: Module loader

| 順序 | Branch | Commit 単位 | 完了条件 |
|---:|---|---|---|
| 1 | `selfhost/s2-vfs-model` | VirtualFileSystem と logical path | pure VFS doctest |
| 2 | `selfhost/s2-import-spec` | import spec 表現と parser 接続 | import clause fixture |
| 3 | `selfhost/s2-module-graph` | graph、cycle detection、missing module diagnostic | cycle / missing test |
| 4 | `selfhost/s2-stdlib-map` | stdlib root と user root の解決 | stdlib import fixture |

### S3: Type / check

| 順序 | Branch | Commit 単位 | 完了条件 |
|---:|---|---|---|
| 1 | `selfhost/s3-type-arena` | TypeId、TypeKind、arena、subst | type arena doctest |
| 2 | `selfhost/s3-unify` | unify、occurs check、rollback 境界 | unify fixture |
| 3 | `selfhost/s3-resolve-scope` | DefId、scope、hoist、name resolver | scope parity |
| 4 | `selfhost/s3-expr-reduce` | NEPLg2.0 reduce_calls の移植 | call reduction parity |
| 5 | `selfhost/s3-overload` | overload candidate と checkpoint | overload fixture |
| 6 | `selfhost/s3-traits-effects` | trait capability、Copy/Clone/Drop、effect | focused compiler fixture |

### S4: HIR / resource / mono

| 順序 | Branch | Commit 単位 | 完了条件 |
|---:|---|---|---|
| 1 | `selfhost/s4-hir-model` | HIR data model と lower skeleton | HIR construction doctest |
| 2 | `selfhost/s4-hir-walk` | iterative traversal helper | deep HIR traversal test |
| 3 | `selfhost/s4-move-state` | move state machine | move fixture parity |
| 4 | `selfhost/s4-borrow-drop` | borrow / drop plan / branch merge | borrow/drop fixture parity |
| 5 | `selfhost/s4-mono-instance` | instance cache と name mangling | mono fixture |

### S5: WASM backend

| 順序 | Branch | Commit 単位 | 完了条件 |
|---:|---|---|---|
| 1 | `selfhost/s5-byte-builder` | ByteBuilder API 使用方針 | byte builder doctest |
| 2 | `selfhost/s5-leb128-section` | LEB128 と section writer | known byte output |
| 3 | `selfhost/s5-wasm-layout` | value layout、locals、function type | layout fixture |
| 4 | `selfhost/s5-wasm-smoke-i32` | `println_i32` / function call | wasm smoke |
| 5 | `selfhost/s5-wasm-aggregate` | struct / enum / Result | Rust backend parity |

### S6: CLI

| 順序 | Branch | Commit 単位 | 完了条件 |
|---:|---|---|---|
| 1 | `selfhost/s6-args` | pure argv parser | argv table test |
| 2 | `selfhost/s6-file-io` | fs read/write bridge | artifact output test |
| 3 | `selfhost/s6-reporter` | stderr human diagnostic と JSON | stdout/stderr split |
| 4 | `selfhost/s6-driver` | compile result、exit code、artifact write | CLI smoke |

### S7: Bootstrap comparison

| 順序 | Branch | Commit 単位 | 完了条件 |
|---:|---|---|---|
| 1 | `nodesrc/selfhost-bootstrap-runner` | Pass A / Pass B 実行 tool | comparison JSON |
| 2 | `selfhost/s7-trace-json` | stage trace JSON | 差分切り分け可能 |
| 3 | `ci/selfhost-bootstrap-smoke` | CI smoke job | bootstrap smoke green |
| 4 | `ci/selfhost-bootstrap-final` | final comparison job | artifact / diagnostic / exit code parity |

---

## 12. Issue 更新ルール

- 実装 branch の開始時点で主 Issue を決める。
- 作業中に不足 interface や別原因を見つけた場合は、実装前に `node nodesrc/issues.js new` で Issue を作る。
- Issue を修正した commit では、同じ commit に `status`、`resolved`、`updated`、検証結果を含める。
- Issue を完全に閉じない準備 commit では `status: investigating` までに留める。
- 旧 `RV-...` を参照する場合は、新 ID と `legacy_id` の両方を commit 本文か note に残す。

---

## 13. Discord 報告

各 commit 後に次を必ず含める。

- commit hash
- branch 名
- 主 Issue
- 直近の改良
- これからする内容
- 検証結果

複数 commit を同じ branch に作る場合も、commit ごとに報告する。
報告後に `main` へ merge した場合、merge が fast-forward なら追加報告は不要とする。

---

## 14. 次の着手順

1. `selfhost/s0-source-tree`
2. `selfhost/s0-infra-span-diag`
3. `nodesrc/selfhost-focused-tests`
4. `stdlib/fs-write-api`
5. `stdlib/fs-dirlist-api`
6. `stdlib/stdio-result-stderr`
7. `stdlib/text-utf8-validation`
8. `selfhost/s1-source-text`
9. `selfhost/s1-token-model`
10. `selfhost/s1-lexer-basic`

I/O interface は lexer/parser と並行したい誘惑があるが、CLI と bootstrap comparison の根になるため、S1 の途中までに必ず片付ける。
