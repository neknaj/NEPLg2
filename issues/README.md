# Issue 管理

このディレクトリを NEPLg2 の Issue 管理の正とする。
旧 `doc/review20260425/` は移行元の履歴スナップショットとして残し、更新は原則として `issues/items/` に対して行う。

## ID

Issue ID は次の形式にする。

```text
ISS-<UTC timestamp>-<slug>-<random-or-hash>
```

例:

```text
ISS-20260426T011530123Z-STDFS-WRITE-B7C4D923
```

- `UTC timestamp` は `YYYYMMDDTHHMMSSmmmZ` または移行用の `YYYYMMDDTHHMMSSZ`。
- 新規 Issue は `nodesrc/issues.js new` が `crypto.randomBytes` の suffix を付ける。
- 旧 review からの移行 Issue は `legacy_id` と title の hash を suffix にする。
- 領域ごとの連番を採用しないため、複数箇所で同時に Issue を作成しても中央カウンタ衝突が起きない。

## ファイル構成

| パス | 内容 |
|---|---|
| `items/*.md` | 個別 Issue。frontmatter を正とする |
| `index.json` | ツール生成の機械可読 index |
| `index.md` | ツール生成の人間向け index |

## Frontmatter

各 Issue は必ず次のフィールドを持つ。

| フィールド | 値 |
|---|---|
| `id` | `ISS-...` |
| `title` | 短い要約 |
| `area` | `core` / `cli` / `stdlib` / `examples` / `selfhost` など |
| `status` | `open` / `investigating` / `fixed` / `verified` / `wontfix` |
| `resolved` | `true` / `false` |
| `priority` | `P0` / `P1` / `P2` / `P3` |
| `type` | `bug` / `performance` / `architecture` / `test` / `doc` / `security` / `maintenance` |
| `created` | 作成日 |
| `updated` | 更新日 |

任意フィールド:

| フィールド | 内容 |
|---|---|
| `target` | 主な対象ファイル・ディレクトリ |
| `legacy_id` | 旧 `RV-...` ID |
| `source` | 由来ドキュメントや計画書 |

## ツール

```bash
node nodesrc/issues.js migrate-review20260425
node nodesrc/issues.js new --area selfhost --title "短いタイトル" --priority P1 --type architecture
node nodesrc/issues.js index
node nodesrc/issues.js check
```

`index` と `check` はコミット前に実行する。
