# nodesrc Discord Webhook

This document explains how to send a message to Discord from `nodesrc/cli.js`.

## Usage

1) Set webhook URL with environment variable

```bash
set NEPL_DISCORD_WEBHOOK_URL=https://discord.com/api/webhooks/...
node nodesrc/cli.js "LLM status update"
```

2) Pass webhook URL explicitly

```bash
node nodesrc/cli.js --discord-webhook-url "https://discord.com/api/webhooks/..." --discord "LLM status update"
```

## Notes

- `--discord` is optional. If no mode flags are used, plain `node nodesrc/cli.js "..."` is treated as a discord message.
- Discord posting mode cannot be mixed with `-i`, `-o`, or `--playground-editor-tests`.
- Message text is split into chunks up to 2000 characters by default.
- Environment variables:
  - `NEPL_DISCORD_WEBHOOK_URL`
  - `DISCORD_WEBHOOK_URL` (fallback)
  - `NEPL_DISCORD_WEBHOOK_MESSAGE_MAX` (message chunk size, default `2000`)
- Posted messages use the webhook display name `NEPLg2 dev report`.
- Timeout/retry controls:
  - `NEPL_DISCORD_WEBHOOK_TIMEOUT_MS` (default `15000`)
  - `NEPL_DISCORD_WEBHOOK_RETRIES` (default `3`)
- Mentions are disabled with `allowed_mentions: { parse: [] }`.

## Success output

On success:

```
discord sent: chunks=1, url=/api/webhooks/...
```

## 進捗レポートの書き方
本機能の報告は `doc/progress_report_template.md` の形式に従うこと。
- タイトルは `# 進捗報告: YYYY-MM-DD — <1行要約>` 形式
- `## 直近の改良` 内で、通常改良と `Issue対応`（Issue番号/原因/対応内容）を分けて記載する

## 最低記載ルール
- タイトルは `# 進捗報告: YYYY-MM-DD — <1行要約>`
- `## 直近の改良` を必ず含める
- `## これからする内容` を必ず含める
- `## 検証` に実行結果（成功/失敗）を添える
