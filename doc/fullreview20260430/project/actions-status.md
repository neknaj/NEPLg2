# GitHub Actions 状況

確認対象 commit: `caca505d fix(selfhost): model lexer raw modes with enums`

## 確認方法

レビュー用の test 状況は local test ではなく、`gh` で GitHub Actions から取得した結果を根拠にする。

実行した主なコマンド:

- `gh run list --branch main --limit 8`
- `gh run view 25506281464 --json status,conclusion,name,headSha,headBranch,createdAt,updatedAt,jobs`
- `gh run view 25506711533 --json status,conclusion,headSha,createdAt,updatedAt,jobs`
- `gh run view 25507054306 --json status,conclusion,headSha,createdAt,updatedAt,jobs`
- `gh run view 25507326678 --json status,conclusion,updatedAt,jobs`
- `gh run list --branch main --limit 8`
- `gh run list --branch main --limit 5`
- `gh run list --limit 6`

## 最新 run

| 項目 | 内容 |
|---|---|
| run id | `25509824320` |
| workflow | `CI (NEPL-g2)` |
| branch | `main` |
| head sha | `caca505d` |
| status | in_progress |
| conclusion | 未確定 |
| createdAt | `2026-05-07T16:51:51Z` |

この run は確認時点では in_progress で、latest main の green 判定は未確定である。

## 直前 run

`dc6b82bb` の run `25508091075` は後続 push により cancelled になった。latest completed result ではなく、連続 push 中の古い run として扱う。

`f3a4c60b` の run `25507959628` は後続 push により cancelled になった。`build` job は source policy と docs build まで進んだが、artifact upload 前に cancel され、Pages final 側は bootstrap artifact 不在で失敗扱いになっている。これは後続 push による中断であり、product failure としては扱わない。

## failure を観測した直近 run

`b9e85f23` の run `25507326678` も後続 push により cancelled になった。cancel 前に以下の job 状態を確認した。

| job | 状態 | 備考 |
|---|---|---|
| `build` | success | source policy regressions と HTML build は通過。 |
| `compile-test` | success | Rust compile tests と wasm32 compile tests は通過。 |
| `nm-compile` | failure | `NM compile tests` が failure。latest run で再確認する。 |
| `tutorials-test` | failure | `Run tutorials doctests` が failure。getting_started / VFS tree failure は issue 化済み。 |
| `wasi-test` / `nmd-doctest` / `stdlib-test` / `rust-test` / `llvm-test` / `llvm-dual-test` | cancelled | 後続 push により中断。 |

この failure set は後続 push で変わる可能性があるため、latest completed run の log / artifact で再判定する。tutorial getting_started failure と VFS cross-file definition path failure は、それぞれ issue 化済みである。

## 直近 run の傾向

| head | 状態 | 備考 |
|---|---|---|
| `caca505d` | in_progress | 最新 main。lexer raw mode を `SelfhostLexerRawMode` enum 化。 |
| `9655c078` | waiting | review validity 文書。後続 push により待機。 |
| `c5f93163` | cancelled | HIR expression payload を variant enum 化。後続 push により古い run。 |
| `dc6b82bb` | cancelled | resolver DefId absence を Option 化。後続 push により cancel。 |
| `f3a4c60b` | cancelled | VFS definition path failure issue 追加。後続 push により cancel。 |
| `00288fb3` | queued / old | getting_started tutorial failure issue 追加。後続 push により古い run。 |
| `8ff05570` | cancelled | HIR expr id absence を Option 化。後続 push により古い run。 |
| `b9e85f23` | cancelled, `tutorials-test` / `nm-compile` failure observed | mono instance absence を Option 化。`build` / `compile-test` は success。 |
| `6277239` | cancelled | HIR range payload 分離。後続 push により古い run。 |
| `4da7333` | in_progress / cancelling | type record payload 分離。後続 push により古い run。 |
| `0ac34132` | cancelled | builtin signature arity enum 化。cancel 前に複数 doctest job failure を確認。 |
| `3951d807` | cancelled | examples CI coverage gap issue 追加。後続 push により cancel。 |
| `32e69bf4` | cancelled | Zed build artifacts issue 追加。後続 push により cancel。 |
| `0fcc4839` | cancelled | selfhost enum equality direct match 化。後続 push により cancel。 |
| `c64396d6` | cancelled | stdlib review 追加。後続 push により cancel。 |
| `b350213c` | cancelled | selfhost compiler review 追加。後続 push により cancel。 |

CI workflow は `concurrency.group: ci-${{ github.ref }}` と `cancel-in-progress: true` を持つため、main への連続 push 中は古い run が cancelled になる。review では cancelled run を failure と同一視せず、latest head の completed conclusion と job failure を基準にする。

## workflow 構成

`.github/workflows/ci.yml` で確認した job:

- `build`
- `compile-test`
- `rust-test`
- `nm-compile`
- `wasi-test`
- `nmd-doctest`
- `tutorials-test`
- `stdlib-test`
- `llvm-test`
- `llvm-dual-test`
- `pages-fast-bundle`
- `pages-fast-deploy`
- `pages-final-bundle`
- `pages-final-deploy`

test coverage は Rust compile/run、WASI doctest、`.n.md` doctest、tutorial doctest、stdlib doctest、LLVM backend、pages build/deploy まで広い。review では特に `wasi-test`、`nmd-doctest`、`tutorials-test`、`stdlib-test`、`llvm-dual-test` の状態を重視する。

## 現時点の判定

- 最新 main の CI 結果は未確定。
- `b9e85f23` run の `tutorials-test` / `nm-compile` failure は古い run 内の途中結果として記録し、latest `caca505d` completed run の結果で再判定する。
- review commit 前には `node nodesrc/issues.js check` と `git diff --check` を実施する。
