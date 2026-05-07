# nodesrc review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `nodesrc/cli.js`
- `nodesrc/tests.js`
- `nodesrc/run_doctest.js`
- `nodesrc/run_test.js`
- `nodesrc/parser.ts` / generated `nodesrc/parser.js`
- `nodesrc/html_gen.ts` / generated `nodesrc/html_gen.js`
- `nodesrc/run_source_policy_regressions.js`
- `nodesrc/test_*.js`
- `nodesrc/static/**`

## 良い点

`nodesrc` は NEPLg2 の実運用を支える thin tooling layer として、doctest、HTML generation、source policy、issue tooling、playground editor snapshot、Discord report をまとめている。特に `nodesrc/issues.js` による issue index 生成と `nodesrc/cli.js --discord` は agent 開発ルールに合っている。

`parser.ts` / `html_gen.ts` は TypeScript source を持ち、CI の bootstrap action で `tsc -p nodesrc/tsconfig.json` が実行される。過去の generated artifact drift は `test_doctest_diag_code_metadata.js` / `test_doctest_exit_code_metadata.js` で固定されている。

`run_source_policy_regressions.js` は、多数の source policy test を一括実行する入口になっている。ResourceIR、stdlib unsafe unwrap、string boundary、selfhost enum/match、diagnostic code-first、playground editor diagnostic contract など、通常の runtime test では検出しにくい設計退行を監視している。

`tests.js` と `run_test.js` は runner metadata、timing、timeout、WASI/WASIX fallback、LLVM dual backend に対応しており、今後 `.n.md` manifest を Rust/selfhost 共通化する土台として使える。

## 問題とリスク

`run_source_policy_regressions.js` は CI で `--warn-only` として呼ばれる。これは downstream CI を止めないための運用だが、policy failure を green と同じ扱いにしてはいけない。review / issue / Discord report で warning を追跡する運用を続ける必要がある。

`tests.js` と `run_doctest.js` の expectation logic はまだ完全な単一 module ではない。`shared_nmd_test_plan.md` の通り、Rust/selfhost 共通 runner を作る前に `DoctestCase` schema と expectation application を共通化する必要がある。

`nodesrc/cli.js` は多機能で、HTML generation、playground editor tests、Discord、search、doctest などの入口を持つ。今後も機能追加を続けるなら command dispatcher と mode-specific module を明確に分けないと肥大化する。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `nodesrc/cli.js` | docs/test/playground/discord の入口。 | 有用だが肥大化注意。 |
| `nodesrc/tests.js` | aggregate doctest runner。 | 良い。manifest schema 共通化が次。 |
| `nodesrc/run_doctest.js` | focused runner。 | 良い。expectation 共通化が次。 |
| `nodesrc/run_test.js` | WASM/WASI/WASIX/LLVM runner。 | 良い。selfhost backend 追加余地あり。 |
| `parser.ts/js` | n.md parser。 | drift regression あり。 |
| `html_gen.ts/js` | doc/html generator。 | nm/htmlgen stdlib と役割が重なるため境界維持が必要。 |
| source policy | 多数の設計回帰を検出。 | warning 運用の追跡が必須。 |

## 推奨対応

- `DoctestCase` schema と expectation logic を共通 module として切り出す。
- `run_source_policy_regressions.js --warn-only` の結果を Actions summary と issue 運用で確実に拾う。
- `cli.js` は新 mode 追加時に mode module へ分離し、top-level dispatcher の肥大化を避ける。
