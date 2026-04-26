# NEPLg2.0 実装レビュー Issue 管理

作成日: 2026-04-25

> このディレクトリは履歴スナップショットです。新しい Issue 管理の正は [`../../issues/`](../../issues/README.md) です。
> 旧 `RV-...` ID は各 Issue の `legacy_id` として移行済みです。

このディレクトリは、NEPLg2.0 の現行実装を `core`、`cli`、`stdlib`、`examples` に分けてレビューし、見つかった問題を Issue として継続管理するための台帳です。

## 対象

| 領域 | 主な対象 |
|---|---|
| core | `nepl-core/src/**` |
| cli | `nepl-cli/src/**`, `nodesrc/**` の CLI / テスト実行系 |
| stdlib | `stdlib/**` |
| examples | `examples/**`, `doc/examples.md` |

`plan.md` は参照専用です。実装との差分や修正方針はこのレビュー文書と `note.n.md` に記録します。

## Issue ID

| 接頭辞 | 領域 | 例 |
|---|---|---|
| `RV-CORE` | Rust コンパイラコア | `RV-CORE-001` |
| `RV-CLI` | Rust CLI / Node CLI / テスト実行系 | `RV-CLI-001` |
| `RV-STDLIB` | NEPL stdlib / self-host 実装 | `RV-STDLIB-001` |
| `RV-EXAMPLE` | 実行可能サンプル | `RV-EXAMPLE-001` |

番号は領域ごとに 3 桁の連番にします。解決時も番号は再利用しません。

## 状態と解決済フラグ

各 Issue は必ず次のフィールドを持ちます。

| フィールド | 値 |
|---|---|
| 解決済 | `false` / `true` |
| 状態 | `open` / `investigating` / `fixed` / `verified` / `wontfix` |
| 優先度 | `P0` / `P1` / `P2` / `P3` |
| 種別 | `bug` / `performance` / `architecture` / `test` / `doc` / `security` |

`解決済: true` にできるのは、修正が入り、必要なテストまたは再現確認が完了した後だけです。

## 優先度

| 優先度 | 意味 |
|---|---|
| P0 | コンパイラクラッシュ、誤った成功、メモリ破壊など、すぐ修正すべき問題 |
| P1 | 実用上の大きなバグ、顕著な性能劣化、設計方針との重大な矛盾 |
| P2 | 仕様不整合、保守性の大きな低下、テスト不足 |
| P3 | 改善余地、整理、将来作業 |

## 記録形式

領域別ファイルでは、次の形式で記録します。

```md
## RV-CORE-000: 短いタイトル

- 解決済: false
- 状態: open
- 優先度: P1
- 種別: bug
- 対象: `path/to/file`

### 根拠

- `path/to/file:line`: コード上の根拠。

### 問題

何が問題かを説明します。

### 影響

利用者、実装、テスト、性能へどう影響するかを説明します。

### 修正方針

間に合わせではなく、原因を取り除く修正方針を書きます。

### 検証

修正後に実行すべきテストや追加すべき回帰テストを書きます。
```

## ファイル

| ファイル | 内容 |
|---|---|
| [issues.md](./issues.md) | 全 Issue の中央台帳 |
| [core.md](./core.md) | `nepl-core` レビュー |
| [cli.md](./cli.md) | `nepl-cli` / `nodesrc` レビュー |
| [stdlib.md](./stdlib.md) | `stdlib` レビュー |
| [examples.md](./examples.md) | `examples` レビュー |

