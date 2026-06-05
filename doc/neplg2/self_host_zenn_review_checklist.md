# NEPLg2.1 self-host Zenn review checklist

最終更新: 2026-06-06

## 目的

この文書は、`stdlib/neplg2/` セルフホストコンパイラの各実装 slice で、Zenn 記事 `https://zenn.dev/bem130/articles/1b352797de94e7` の方針を subagent review によって継続確認するための checklist である。

`doc/neplg2/self_host_neplg21_compiler_design.md` は設計 authority、`doc/neplg2/self_host_execution_plan.md` は作業手順 authority、この文書は subagent review に渡す確認項目 authority とする。subagent へ渡す標準依頼文と標準 response 形式は `doc/neplg2/self_host_zenn_review_prompt.md` を正とする。

この checklist は、レビューの回数を増やすためだけの文書ではない。レビューごとに同じ観点を確認し、Blocker を同じ branch 内で修正するか、原因、影響、完了条件を持つ issue へ分離するための文書である。

## review の入力

subagent review を依頼するときは、少なくとも次を渡す。

- `nodesrc/selfhost_zenn_review_packet.js` で生成した review packet、または同等の全項目を持つ依頼文。packet は `YYYY-MM-DD` または ISO-like date-time の Zenn 再確認日時、committed / staged / unstaged / untracked の差分区分、accepted / fail-closed、検証欄、既存 warning、今回差分由来 warning を明示する。
- `doc/neplg2/self_host_zenn_review_prompt.md` の request template。
- 対象 branch と対象 commit。
- 対象 issue または実装 slice の目的。
- Zenn 記事 URL。
- `AGENTS.md` の関連方針。
- 今回変更した file list。
- 設計文書、issue、`note.n.md`、source policy の更新箇所。
- 実行した検証、未実行の検証、既存 warning と今回差分由来の warning の区別。
- review の観点が `policy/spec` と `implementation/test` の 2 軸に分かれていること。
- 個別 subagent review response を `nodesrc/selfhost_zenn_review_response_check.js --review-kind individual` で検査し、必須 section / field が欠けた返答を受理しないこと。
- 最終受理時は、2 件以上の個別 subagent review response を集約し、`nodesrc/selfhost_zenn_review_response_check.js --review-kind final --record <note-or-issue.md>` で、response の要約と判断根拠が `note.n.md` または `issues/items/*.md` の関連 issue に残っていることを検査すること。一時ファイルや repo 外ファイルは durable な review 証跡ではない。
- `--record` は最終集約 review だけで使う。個別 subagent review 1 件を durable final acceptance として扱ってはならない。
- review response と durable record には `subagent_review_ids` と `subagent_review_count` を残すこと。`subagent review` という文字列だけでは、どの独立 review を受理したかの証跡として扱わない。
- selfhost Zenn 方針の最終受理には、2 件以上の独立 subagent review が必要である。1 件だけの review、同一 subagent id の重複、または件数と id list が一致しない response は受理しない。
- `source_policy: required` または `source_policy: follow-up` が残る `MERGE_APPROVED` は受理しない。必要な source policy は同じ branch で追加・更新するか、merge 前に `not-needed` の根拠へ落とし込む。
- 今回差分由来 warning が残る `MERGE_APPROVED` は受理しない。既存 warning は既存として記録し、今回差分で増えた warning は修正する。review response と durable record の両方に `existing_warnings` / `new_warnings` または同等の warning 区分を残す。
- durable record 側にも `policy/spec` と `implementation/test` の両方の `source_policy`、`residual_risk`、`unexecuted_verification`、`existing warnings`、`new warnings` または同等の機械可読 field を残す。`MERGE_APPROVED` の durable record に `source_policy: required` / `source_policy: follow-up`、残リスク、未実行検証、今回差分由来 warning が残る場合は受理しない。
- `zenn_check` は対象 file、関数、test、source policy、authority boundary などの具体的な根拠を要求する。`` `Result` ``、`` `enum` ``、`` `match` `` のように抽象語だけを code span にしたものは、Zenn 方針を確認した証跡として扱わない。

## 必須確認項目

subagent は、次の項目を確認する。

### review の 2 軸

- `policy/spec`: Zenn 記事、`AGENTS.md`、NEPLg2.1 仕様、設計文書、issue の完了条件に照らして、仕様境界や公開 API が正しいか確認する。
- `implementation/test`: 実装、doc comment、source policy、doctest、focused test、broad regression に照らして、実装が仕様と方針を満たすか確認する。
- 片方だけで approve しない。仕様判断に疑問がある場合は `Question` として返し、実装だけで押し切らない。

### 静的検査と error model

- 失敗経路が `Result`、`Option`、enum diagnostic で表されていること。
- 表示文字列、数値 sentinel、数値 kind、文字列 kind で分岐していないこと。
- error enum と表示、JSON code、diagnostic text の責務が分離されていること。
- `match` の網羅性検査が効く形で variant を扱っていること。
- `_:` catch-all を使う場合は、将来 variant を隠さない fail-closed branch であることが doc comment または source policy から確認できること。

### pure core と platform boundary

- compiler core が filesystem、stdio、argv、環境変数、clock、random、DOM、Canvas などの host detail を直接扱っていないこと。
- host detail は `cli/`、web、native adapter、または明示的な platform boundary に閉じていること。
- pure / impure の境界が型と実装で一致していること。
- private cache、memoization、内部 allocation、private state を pure と見なす場合は、外部観測不能性と escape 禁止が設計または issue に明記されていること。

### authority boundary

- parser が prefix call boundary、overload、generic、trait、expected type を先に決めていないこと。
- checker が HIR allocation や backend detail を直接扱っていないこと。
- HIR lowering が source text や token lexeme を再読して型証拠を作り直していないこと。
- Resource IR が ownership、borrow、initialized state、drop、internal effect の authority を持っていること。
- backend が typecheck や Resource proof の不足を補うための推測をしていないこと。

### documentation comment

- 各 module、public type、public function、重要な private helper に、日本語の `//:` doc comment があること。
- doc comment が目的、契約、戻り値、error variant の条件、計算量、制約、典型例、現状の実装詳細を必要に応じて説明していること。
- 将来も守る契約と、今後変わり得る現状実装の説明が分離されていること。
- `Option`、`Result`、enum を返す場合は、どの条件でどの variant になるかが記載されていること。
- コメントを短くするための行数制限、説明削減、機械的な boilerplate 化が入っていないこと。
- `doc/neplg2/self_host_zenn_review_checklist.md` と `doc/neplg2/self_host_zenn_review_prompt.md` 自体にも、行数、byte数、file size、comment量、doc comment長を理由に説明を削る制限を入れないこと。これらの運用文書は reviewer と agent の authority なので、source policy と同じく制限混入を検査対象にする。

### performance と探索範囲

- 高速化が安全性検査の省略ではなく、探索範囲、依存関係、query key、artifact、cache key、事前検査済み summary、DAG 化によって行われていること。
- cold compile の固定費と warm / incremental compile の再利用境界が混同されていないこと。
- cache に頼る前に、同じ情報の再探索、source 再読、重複 type lookup、不要な owner allocation が減っているか確認していること。
- performance 用の shortcut が diagnostic、type safety、memory safety、pure / impure boundary を弱めていないこと。

### prototype policy

- 試作段階で後方互換を壊す判断は許容しても、雑な設計、隠れた技術的負債、回避実装を残していないこと。
- 暫定実装を置く場合は、識別子または doc comment で検索可能にし、妥協内容、fail-closed 範囲、解除条件、対応 issue を記録していること。
- 仕様変更がある場合は、README、`doc/`、issue、migration tool、source policy、tests の更新要否を確認していること。
- Rust 側 bug や仕様未確定を selfhost 側 workaround で隠していないこと。

## 指摘分類

subagent は、指摘を次の分類で返す。

- `Blocker`: 方針違反、静的検査の穴、ownership / effect safety の穴、誤った public API、doc comment 契約不足、または既存 warning ではない今回差分由来の regression。
- `Non-blocker`: 同じ slice で直す必要はないが、次 slice で扱うべき改善、source policy 強化、追加 doctest、性能測定の追加。
- `Question`: 仕様判断や優先順位が必要な確認事項。
- `Approve`: Blocker がなく、merge 可能と判断できる場合。

`Blocker` は同じ branch 内で修正する。修正できない場合は、原因、影響、完了条件、検証予定を持つ issue へ分離する。`Non-blocker` は `note.n.md`、`todo.md`、または対応 issue に残す。

指摘ごとに、次の field を残す。

```text
classification: Blocker | Non-blocker | Question | Approve
decision: fixed | issue | open | not-applicable
source_policy: added | updated | not-needed | follow-up
verify: <実行した検証、または未実行理由>
```

`MERGE_APPROVED` は、`blockers` と `questions` が空で、`approve` が明示的に承認を示し、`files_read`、`not_reviewed`、2 件以上の `subagent_review_ids`、`subagent_review_count`、`existing_warnings`、`new_warnings` が記録され、source policy 不足、今回差分由来 warning、未実行検証、未説明 residual risk が残っていない場合だけ受理する。`nodesrc/selfhost_zenn_review_response_check.js` は、この最小条件と必須 section / field を検査する。

## note checkpoint 形式

commit 前に `note.n.md` へ次を記録する。

最新の `note.n.md` selfhost checkpoint は `nodesrc/test_selfhost_zenn_review_gate_contract.js` で検査する。これは review 証跡が人間の記憶や口頭報告にだけ残ることを防ぐための検査であり、行数、ファイル長、doc comment 長さ、コメント量を制限する検査ではない。

```text
## YYYY-MM-DD Agent selfhost <topic> checkpoint

- Zenn 記事を再確認した。
- AGENTS.md の関連方針を確認した。
- 対象 branch / commit / issue。
- 今回の設計判断と、Zenn 方針のどの項目に対応するか。
- `policy/spec` と `implementation/test` の review 観点。
- subagent review の件数、`subagent_review_ids`、Blocker、Non-blocker、Question、Approve の要約。
- 2 件以上の独立 subagent review を受けたこと。1 件だけの場合は受理せず、追加 review を依頼する。
- Blocker の修正内容、または issue 化した場合の issue ID。
- 指摘別の `classification`、`decision`、`source_policy`、`verify`。
- source policy / doctest / focused test / broad regression の検証結果。
- 既存 warning と今回差分由来 warning の区別。
- 次 slice へ残す作業。
```

## source policy 化の基準

同じ種類の方針違反が戻り得る場合は、source policy regression を追加する。対象は行数やコメント量ではなく、構造と契約である。

- typed error enum が残っていること。
- owner recovery が失われていないこと。
- fallback API や sentinel が再導入されていないこと。
- facade が実装を持ちすぎず、責務 module へ委譲していること。
- parser / checker / HIR / Resource IR / backend の authority が逆流していないこと。
- doc comment が目的、契約、戻り値、error variant、計算量、制約、現状説明を保持していること。
- Zenn review gate と subagent review の証跡が実行計画、設計文書、`note.n.md` に残ること。
- 個別 subagent review response の必須 section / field を `nodesrc/selfhost_zenn_review_response_check.js --review-kind individual` で検査していること。
- 最終集約 response と durable record を `nodesrc/selfhost_zenn_review_response_check.js --review-kind final --record <note-or-issue.md>` で検査していること。
- selfhost Zenn review の最終受理で 2 件以上の独立 subagent review、具体的な `zenn_check` 根拠、source policy 不足なし、今回差分由来 warning なしを検査していること。
- 新規 source policy を追加した場合に `nodesrc/run_source_policy_regressions.js` へ登録されていること。

行数制限、ファイル長制限、doc comment 長制限は source policy に入れない。大きさの問題は、責務混在、依存方向、facade への実装流入、テスト不能な単位、review 不能な境界として検査する。
