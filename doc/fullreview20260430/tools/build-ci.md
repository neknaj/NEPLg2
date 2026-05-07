# build and CI review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `Cargo.toml`
- `nepl-core/Cargo.toml`
- `nepl-cli/Cargo.toml`
- `nepl-web/Cargo.toml`
- `web/package.json`
- `Trunk.toml`
- `.github/workflows/ci.yml`
- `.github/actions/bootstrap-build/action.yml`

## 良い点

root workspace は `nepl-core`、`nepl-cli`、`nepl-language`、`nepl-lsp` を含む。`nepl-web` は `cdylib` として別 workspace 扱いで、Trunk build の wasm binding に使われる。

CI は shared bootstrap action で Node、TypeScript compile、Rust toolchain、wasm32 target、Trunk、wasm-bindgen、Cargo build、Trunk build を 1 回実行し、その artifact を後続 job が再利用する。重い bootstrap を共通化している点は良い。

workflow は compile-test、rust-test、nm-compile、wasi doctest、nmd doctest、tutorials、stdlib、LLVM compile-only、LLVM dual backend、Pages pending/final deploy を分けている。Pages final status は test artifacts を `dist/tests` へ集約する。

workflow concurrency は `ci-${{ github.ref }}` で cancel-in-progress になっており、main への連続 push 中に古い run が cancelled になる。この挙動を failure と誤認しないことは review 文書に記録済みである。

## 問題とリスク

最新 main の Actions は、今回確認時点では `c5f93163` の run `25508600937` が in_progress であり、green 判定は未確定である。直前の `b9e85f23` run `25507326678` は後続 push で cancelled になったが、`build` / `compile-test` は success、`tutorials-test` / `nm-compile` は failure になっていた。latest completed run で再判定する。

examples doctest job がない。CI は `examples/nm.nepl` compile と `examples/counter.nepl` emit smoke を持つが、`examples/*.nepl` の embedded doctest をまとめて実行しない。これは `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895` で追跡する。

source policy は `build` job で `--warn-only` 実行される。CI artifact と downstream test を止めない利点はあるが、安全 policy の failure を pass と誤認しない review 運用が必要である。

`Trunk.toml` は Windows で `npm.cmd` を使い、CI action で Linux 向けに `npm` へ置換している。実用上は動くが、platform-specific command rewrite は fragile である。将来は Trunk hook を OS 非依存にするか、CI 側で明示的に npm build を済ませる設計へ寄せたい。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| Rust workspace | core/cli/language/lsp。 | 良い。 |
| `nepl-web` build | wasm-bindgen + Trunk。 | 良い。 |
| TypeScript build | `web` と `nodesrc` を compile。 | 良い。 |
| doctest CI | tests/tutorials/stdlib を分割。 | examples gap あり。 |
| LLVM CI | compile-only と dual backend。 | 良い。 |
| Pages | pending/final artifact split。 | 良い。直前 run では final deploy まで success。 |
| source policy | warn-only。 | 追跡運用が必要。 |

## 推奨対応

- latest main の completed Actions result を最終 review で再確認し、doctest job failure が残る場合は根本原因を分離する。
- examples doctest CI を追加し、final Pages status に含める。
- source policy warning は Actions summary だけでなく issue / Discord report で追跡する。
- `Trunk.toml` の OS 依存 hook rewrite を減らす。
