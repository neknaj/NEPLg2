# NEPLg2.1 self-host subagent review prompt

最終更新: 2026-06-06

## 目的

この文書は、`stdlib/neplg2/` セルフホストコンパイラの実装 slice で、subagent に渡す標準 review 依頼文と標準 response 形式を定義する。

review の観点は `doc/neplg2/self_host_zenn_review_checklist.md` を正とする。この文書は、その checklist を毎回の subagent 依頼に漏れなく含めるための prompt template である。

## review request template

subagent へ依頼するときは、`nodesrc/selfhost_zenn_review_packet.js` で現在 branch / base commit / head commit / 変更 file list を入れた review packet を生成し、その本文を依頼文の土台にする。

```bash
node nodesrc/selfhost_zenn_review_packet.js \
  --issue <issue-id-or-note-checkpoint> \
  --slice <implementation-slice-name> \
  --accepted <accepted-scope> \
  --fail-closed <remaining-fail-closed-scope> \
  --zenn-checked-at <YYYY-MM-DD-or-ISO-like-date-time> \
  --executed <command-list> \
  --not-executed <command-and-reason-list-or-none> \
  --existing-warnings <warning-list-or-none> \
  --new-warnings <warning-list-or-none>
```

helper は Zenn 記事 URL、Zenn 再確認日時、`AGENTS.md`、checklist、prompt authority、base / head、committed / staged / unstaged / untracked に分けた差分 file list、accepted / fail-closed、検証欄、既存 warning、今回差分由来 warning を出力する。Zenn 再確認日時は `YYYY-MM-DD` または ISO-like date-time とする。日時付きの場合は `YYYY-MM-DDTHH:mm`、`YYYY-MM-DDTHH:mm:ss`、末尾 `Z`、または `+09:00` のような timezone offset を使える。自然言語、月日だけの値、存在しない暦日、存在しない時刻は helper が拒否する。review owner は helper 実行前に Zenn 記事を再確認し、出力後に空欄や現状とずれた項目が残っていないことを確認してから subagent へ渡す。

手動で補う場合も、次の template の項目を省いてはならない。

```text
Repository:
対象 branch:
base commit:
head commit:
対象 issue / slice:
変更 file list:
変更目的:
今回 accepted にした範囲:
fail-closed に残した範囲:
Zenn policy:
  https://zenn.dev/bem130/articles/1b352797de94e7
  zenn_checked_at: <YYYY-MM-DD-or-ISO-like-date-time>
Repo policy:
  AGENTS.md
Review checklist:
  doc/neplg2/self_host_zenn_review_checklist.md
Design docs:
  doc/neplg2/self_host_neplg21_compiler_design.md
  doc/neplg2/self_host_execution_plan.md
関連 issue / note:
  <issues/items/... または note.n.md の checkpoint>
検証:
  executed:
    - <command>
  not executed:
    - <command and reason>
  existing warnings:
    - <warning known before this slice>
  new warnings:
    - <warning introduced by this slice, or none>

依頼:
  編集しないでレビューのみ行ってください。
  この slice を policy/spec と implementation/test の 2 軸でレビューしてください。
  Zenn policy、AGENTS.md、NEPLg2.1 仕様、設計文書、issue 完了条件、source policy、doc comment、検証結果に照らして確認してください。
  実際に読んだ file list を files_read に列挙してください。
  見ていない範囲は not_reviewed に明記してください。
  review response には、この review を実行した subagent の id を subagent_review_ids に列挙し、件数を subagent_review_count に記録してください。
  `subagent_review_ids` と `subagent_review_count` は、文字列として存在するだけではなく、実際に作業した subagent の id と件数に一致している必要があります。
  この review response は個別 subagent review として `node nodesrc/selfhost_zenn_review_response_check.js --review-kind individual --input <review-response.md>` で検査します。
  個別 subagent review では、この review を実行した subagent id だけを `subagent_review_ids` に記録し、`subagent_review_count: 1` とします。
  最終受理には 2 件以上の独立 subagent review が必要です。この review はそのうちの 1 件として扱われます。最終受理では、agent が 2 件以上の個別 response を集約し、`--review-kind final` または既定の final mode で検査します。
  行数制限、ファイル長制限、doc comment 長制限、コメント削減を理由にしないでください。
  source token 再読、scope lookup 再実行、cursor-only evidence loss、owner/free、pure/impure、authority boundary を重点確認してください。
  Blocker は同じ branch 内で修正が必要なものとして分類してください。
  Non-blocker は次 slice または issue へ残す改善として分類してください。
  Question は仕様判断や優先順位確認が必要なものとして分類してください。
  Approve は Blocker がない場合だけ出してください。
  返答は `nodesrc/selfhost_zenn_review_response_check.js` で検査します。

必ず次の形式で返してください。

## review_scope
- branch:
- base:
- head:
- files_read:
- not_reviewed:
- subagent_review_ids:
- subagent_review_count:

## decision
- MERGE_APPROVED | BLOCKED | QUESTION

## policy/spec
- classification:
- file/function:
- finding:
- root_cause:
- reason:
- recommended_fix:
- source_policy: added | updated | required | not-needed | follow-up
- source_policy_reason:
- doc_issue_note: needed | not-needed
- verify:

## implementation/test
- classification:
- file/function:
- finding:
- root_cause:
- reason:
- recommended_fix:
- source_policy: added | updated | required | not-needed | follow-up
- source_policy_reason:
- doc_issue_note: needed | not-needed
- verify:

## zenn_check
- Result/Option:
- enum error/display separation:
- match exhaustiveness:
- pure/impure boundary:
- authority boundary:
- owner/free:
- zero-cost/performance:
- doc comment:
- prototype/fail-closed:

## evidence_to_record
- note:
- issue:
- source policy:
- tests:

## warnings
- existing_warnings:
- new_warnings:

## summary
- blockers:
- non_blockers:
- questions:
- approve:
- residual_risk:
- unexecuted_verification:
```

## response の扱い

review response を受け取った agent は、次を行う。

- 個別 subagent review response は `node nodesrc/selfhost_zenn_review_response_check.js --review-kind individual --input <review-response.md>` または `--stdin` で検査する。
- commit 前の最終受理では、2 件以上の個別 subagent review response を agent が集約し、集約 response の要約を `note.n.md` または `issues/items/*.md` の関連 issue に記録したうえで、`node nodesrc/selfhost_zenn_review_response_check.js --review-kind final --input <aggregate-review-response.md> --record <note-or-issue.md>` を実行し、review 証跡が durable な記録先にも残っていることを検査する。`--record` に一時ファイルや repo 外ファイルを指定してはならない。
- `--record` は最終集約 review にだけ使う。個別 subagent review 1 件を `--record` で最終受理扱いしてはならない。
- response checker が失敗した返答は review 記録として扱わず、subagent に不足 section / field の再提出を依頼する。
- `MERGE_APPROVED` は、`blockers` と `questions` が空で、`approve` が明示的に承認を示し、`files_read`、`not_reviewed`、2 件以上の独立 `subagent_review_ids`、`subagent_review_count`、`existing_warnings`、`new_warnings` が記録されている場合だけ受理する。
- durable record 側にも `policy/spec` と `implementation/test` の両方の `source_policy`、`residual_risk`、`unexecuted_verification`、`existing warnings`、`new warnings` または同等の機械可読 field を残す。`MERGE_APPROVED` の record に `source_policy: required` / `source_policy: follow-up`、残リスク、未実行検証、今回差分由来 warning が残る場合は受理しない。
- `source_policy: required` または `source_policy: follow-up` が残る `MERGE_APPROVED` は受理しない。必要な source policy は同じ branch で追加・更新する。
- `Blocker` は同じ branch 内で修正する。
- 同じ branch 内で修正できない `Blocker` は、原因、影響、完了条件、検証予定を持つ issue へ分離する。
- `Non-blocker` は `note.n.md`、`todo.md`、または対応 issue に残す。
- `Question` は仕様確認として扱い、勝手な回避実装で進めない。
- `Approve` があっても、検証未実行、未説明の residual risk、または response / record のどちらかに今回差分由来 warning が残る場合は merge しない。
- `Approve` があっても、`files_read`、`not_reviewed`、`subagent_review_ids`、`subagent_review_count`、`zenn_check`、`residual_risk`、`unexecuted_verification` が空の場合は review 記録として扱わない。
- `zenn_check` は `yes` や `確認済み` だけでは受理しない。対象 file、関数、test、source policy、authority boundary などの具体的な根拠を各項目に書く。`` `Result` ``、`` `enum` ``、`` `match` `` のように抽象語だけを code span にしたものは具体的な根拠として扱わない。
- `source_policy: not-needed` の場合も、`source_policy_reason` に理由を残す。

## 禁止事項

- Zenn 記事 URL、`AGENTS.md`、checklist、対象 branch / commit / issue を省いた依頼を出してはならない。
- `policy/spec` と `implementation/test` のどちらか片方だけで approve してはならない。
- `files_read`、`not_reviewed`、2 件以上の独立 `subagent_review_ids`、`subagent_review_count` を省いてはならない。
- `existing_warnings` と `new_warnings` を response から省いてはならない。
- 1 件だけの subagent review または同一 subagent id の重複を最終受理してはならない。
- `source_policy: not-needed` の理由を省いてはならない。
- `source_policy: required` または `source_policy: follow-up` を残したまま `MERGE_APPROVED` にしてはならない。
- Blocker を「後で見る」とだけ書いて merge してはならない。
- 行数制限、ファイル長制限、doc comment 長制限を review 条件にしてはならない。
- コメントを短くするために、目的、契約、戻り値条件、error variant、計算量、制約、現状説明を削ってはならない。
- warning を既存か今回差分由来か分けずに扱ってはならない。
