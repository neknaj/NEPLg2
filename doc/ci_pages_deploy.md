# GitHub Pages デプロイ設計

## 目的

GitHub Pages は Web Playground を最優先で公開する。metrics、docs、test report は付随コンテンツであり、生成やテストが失敗しても Playground の deploy を止めない。

## 成果物の境界

- `playground-site`: `trunk build` が生成した Playground 本体だけを含む。`pages-fast-bundle` と `pages-final-bundle` は必ずこれを土台にする。
- `bootstrap-build`: CI のテストで再利用する compiler build cache であり、Pages job では展開しない。
- `pages-content`: docs、tutorials、metrics の任意コンテンツを含む。生成 step は `continue-on-error` とし、成功したものだけ final Pages に重ねる。
- test JSON artifacts: test job が `always()` でアップロードし、`pages-final-bundle` が取得できたものだけ `dist/tests/` に配置する。

## デプロイ順序

1. `build` が Playground 本体を検証して `playground-site` を作る。
2. `pages-fast-bundle` が `playground-site` に pending status を追加し、`pages-fast-deploy` がすぐ deploy する。
3. test jobs と `pages-content` は並行して進む。失敗しても fast deploy は影響を受けない。
4. `pages-final-bundle` は `playground-site` に任意コンテンツと取得可能な test JSON を重ね、final status を書いて deploy する。

## サイズ方針

`html_play` は検索 index を各ページに埋め込まず、scope ごとに `search-index*.json` として 1 回だけ出力する。これにより doc/std のページ数に比例した全体検索 index の重複を防ぐ。

任意コンテンツは 700MiB を超えた場合、docs/tutorials を省略して Playground と metrics/status を優先する。Pages artifact 全体は 900MiB のガードを維持する。
