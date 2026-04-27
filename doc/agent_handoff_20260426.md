# 2026-04-26 Agent 引継ぎ

この資料は、次の push 後に別環境へ移動して作業を継続するための引継ぎである。作業開始時は必ず remote と同期し、ここに書いた手順より新しいユーザー指示があればそちらを優先する。

## 最初に行うこと

1. `git status -sb` で作業ツリーが clean であることを確認する。
2. `git pull --rebase origin main` で remote を取り込む。
3. `node nodesrc/issues.js check` を実行し、Issue 台帳が壊れていないことを確認する。
4. `issues/index.md` の Open Issues を見て、core の pipe / typechecker / borrowchecker / move checker 関連を優先する。
5. `plan.md` は読むだけにし、変更が必要な内容は `note.n.md` または `doc/` に書く。

## 現在の運用ルール

- 1 issue につき 1 commit にする。
- commit 後は Discord report を送る。Markdown 本文を直接送信し、ファイルパス参照だけの報告にしない。
- commit 後は push し、その後 `git pull --rebase origin main` で他 agent の変更を取り込む。
- commit 前には必要なテストを通し、`node nodesrc/issues.js index` と `node nodesrc/issues.js check` を実行する。
- 新たな問題を発見したら `issues/items/*.md` に Issue を追加し、`issues/index.*` を再生成する。Issue を追加した時点でも、その Issue ID・原因・影響・次の対応を Discord report として送る。
- 旧 `doc/review20260425/` は履歴スナップショットであり、通常は更新しない。
- 問題は workaround で隠さず、原因を特定して根本修正する。
- `note.n.md` には実装状況、原因、修正、検証、`plan.md` との差異を書く。

## Discord report

Webhook URL:

```text
https://discord.com/api/webhooks/1484526657946648577/ftq4WlgJuJbh4CPCp41C1AdefAlw4Hihhbh_V1_W4zKWL92JNwCEofBXvPBMGxpZgBIq?thread_id=1497536803236872313
```

送信コマンド例:

```powershell
node nodesrc/cli.js --discord-webhook-url "https://discord.com/api/webhooks/1484526657946648577/ftq4WlgJuJbh4CPCp41C1AdefAlw4Hihhbh_V1_W4zKWL92JNwCEofBXvPBMGxpZgBIq?thread_id=1497536803236872313" --discord "# 進捗報告: 2026-04-26 - <要約>

## 直近の改良
- Issue対応: <issue id>
- commit: <hash>

## これからする内容
- core の pipe / typechecker / borrowchecker 関連 issue を優先して継続

## 検証
- <実行した検証コマンドと結果>"
```

report 形式は `doc/progress_report_template.md` と `doc/nodesrc_discord_webhook.md` を確認する。
Issue 追加のみの報告でも同じ webhook を使い、タイトルは進捗報告として、本文に「追加 Issue」「根拠」「次の対応」を含める。

## 検証方針

Rust 側を変更した場合は原則として次を実行する。

```powershell
cargo fmt --all --check
cargo test -p nepl-core --test <関連テスト>
trunk build
node nodesrc/tests.js -i <関連 n.md> --no-tree -o tmp/<説明的な名前>.json -j 1
node nodesrc/issues.js index
node nodesrc/issues.js check
git diff --check
```

`trunk build` 後に `nodesrc/cli.js` または `nodesrc/tests.js` の JSON 出力を確認する。`tmp/` に作った検証用 JSON や再現用ファイルは commit しない。

## core 優先順位

次の順で処理する。

1. CI / GH Actions で落ちている core issue。
2. pipe / typechecker / borrowchecker / move checker に関する P0/P1 issue。
3. self-host 前提で Rust 参照 compiler の制約になっている core issue。
4. stdlib / cli / examples は、core 修正の検証に必要な場合を除き後回しにする。

作業候補は次のコマンドで確認する。

```powershell
node nodesrc/issues.js index
rg -n "pipe|typecheck|borrow|move|Resource IR|HashKey|self-host" issues/items
```

現時点で特に見るべき core issue は `issues/index.md` の Open Issues にある `RV-CORE-009` と `SELFHOST-REQ-HASHKEY` である。pipe 系 issue は解決済みでも、修正中に parser / typechecker の nested call や pipeline 境界の別問題を見つけた場合は新規 Issue を作る。

## 既知の注意点

- `nepl-core` には既存 warning が多数残っている。これは既存の warning debt issue で追跡されており、別 issue として扱う。
- Wasix doctest runner の Wasmer 1.x `--volume` 非対応は別 Issue で追跡されている。core/typechecker 修正と混ぜない。
- remote では他 agent が同時に `issues/index.*` と `note.n.md` を更新することがある。pull conflict が起きた場合、`issues/index.*` は `node nodesrc/issues.js index` で再生成し、`note.n.md` は両方の作業メモを残して解消する。
- `git stash` に autostash が残っていないか、push 前に `git stash list` で確認する。
