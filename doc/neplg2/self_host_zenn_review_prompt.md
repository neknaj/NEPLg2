# NEPLg2.1 self-host subagent review prompt

最終更新: 2026-06-05

## 目的

この文書は、`stdlib/neplg2/` セルフホストコンパイラの実装 slice で、subagent に渡す標準 review 依頼文と標準 response 形式を定義する。

review の観点は `doc/neplg2/self_host_zenn_review_checklist.md` を正とする。この文書は、その checklist を毎回の subagent 依頼に漏れなく含めるための prompt template である。

## review request template

subagent へ依頼するときは、次の template を使う。

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
  行数制限、ファイル長制限、doc comment 長制限、コメント削減を理由にしないでください。
  source token 再読、scope lookup 再実行、cursor-only evidence loss、owner/free、pure/impure、authority boundary を重点確認してください。
  Blocker は同じ branch 内で修正が必要なものとして分類してください。
  Non-blocker は次 slice または issue へ残す改善として分類してください。
  Question は仕様判断や優先順位確認が必要なものとして分類してください。
  Approve は Blocker がない場合だけ出してください。

必ず次の形式で返してください。

## review_scope
- branch:
- base:
- head:
- files_read:
- not_reviewed:

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

- `Blocker` は同じ branch 内で修正する。
- 同じ branch 内で修正できない `Blocker` は、原因、影響、完了条件、検証予定を持つ issue へ分離する。
- `Non-blocker` は `note.n.md`、`todo.md`、または対応 issue に残す。
- `Question` は仕様確認として扱い、勝手な回避実装で進めない。
- `Approve` があっても、検証未実行や今回差分由来 warning が残る場合は merge しない。
- `Approve` があっても、`files_read`、`not_reviewed`、`zenn_check`、`residual_risk`、`unexecuted_verification` が空の場合は review 記録として扱わない。
- `source_policy: not-needed` の場合も、`source_policy_reason` に理由を残す。

## 禁止事項

- Zenn 記事 URL、`AGENTS.md`、checklist、対象 branch / commit / issue を省いた依頼を出してはならない。
- `policy/spec` と `implementation/test` のどちらか片方だけで approve してはならない。
- `files_read` と `not_reviewed` を省いてはならない。
- `source_policy: not-needed` の理由を省いてはならない。
- Blocker を「後で見る」とだけ書いて merge してはならない。
- 行数制限、ファイル長制限、doc comment 長制限を review 条件にしてはならない。
- コメントを短くするために、目的、契約、戻り値条件、error variant、計算量、制約、現状説明を削ってはならない。
- warning を既存か今回差分由来か分けずに扱ってはならない。
