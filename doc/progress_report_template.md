# 進捗レポートの書き方

## 方針
短く、構造化して書く。  
このファイルは `nodesrc` の進捗共有用レポートの定型です。

## 見出しルール
- タイトルは `# 進捗報告: YYYY-MM-DD`
- 本文は `##` の見出しで区切る
- 1行に1内容を基本とする

## 形式
`#` はタイトル  
`## 直近の改良`  
`## これからする内容`  
`## 検証`  

## テンプレート

```text
# 進捗報告: 2026-04-25

## 直近の改良
- Discord Webhook投稿モードを追加した
- 環境変数でWebhook URLを読んで送信するようにした
- 投稿名を `NEPLg2 dev report` に固定した

## これからする内容
- 投稿成功/失敗時のログ文言を簡潔化する
- 例外が発生した場合の再試行結果を追記する

## 検証
- `node nodesrc/cli.js --discord "check"` 実行で `discord sent` を確認
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=...` 実行で `passedCount` を確認
```

## 記載ルール
- 未完了は `未対応` / `未確定` など状態を明記する
- 検証は「成功」「失敗」を併記する
- 冗長な説明は避け、次アクションを必ず1行以上入れる

