# docs review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `doc/README.md`
- `doc/testing.md`
- `doc/cli.md`
- `doc/neplg2/README.md`
- `doc/neplg2/*.md`
- `doc/neplg3/**`
- `doc/compare/**`
- `doc/migration/**`

## 良い点

`doc/README.md` は NEPLg2.0 と NEPLg3 を分け、現行実装向けドキュメントと次世代仕様を区別している。NEPLg2.0 の保守文書は `doc/neplg2/`、NEPLg3 の仕様と実装設計は `doc/neplg3/` に置かれている。

`doc/neplg2/static_check_complexity_reduction_plan.md`、`static_check_design_verification_20260430.md`、`static_check_soundness_review_20260430.md` は、型安全・メモリ安全を ResourceIR / owner token / effect boundary に寄せる判断の根拠として重要である。selfhost の静的検査設計でも参照すべき一次情報である。

`doc/neplg2/compiler_diagnostics_redesign_plan.md` は diagnostic code を code-first / enum-first に寄せる計画を持つ。selfhost diagnostic id もこの方針に合わせるべきで、自由文字列や数値 id へ戻さない基準として有効である。

`doc/neplg2/shared_nmd_test_plan.md` と `nmd_assert_output_plan.md` は、Rust/selfhost 共通 `.n.md` 運用、stdout assertion report、`exit_code` と `ret` の分離を設計済みである。現行実装の進捗と残件が文書に追記されている点は良い。

## 問題とリスク

`doc/testing.md` の `std/test` 節は、まだ `assert` / `assert_eq_i32` / `test_checked` / `test_fail` など旧説明が残っている。`nmd_assert_output_plan.md` の structured report API 実装状況とずれており、読者が古い assertion style を採用する危険がある。

NEPLg3 仕様には `diag_id` という旧表現が残る箇所がある。現行 NEPLg2 では diagnostic code redesign が進んでいるため、NEPLg3 仕様側も `diag_code` / typed diagnostic enum へ用語を揃える必要がある。

`doc/neplg2/README.md` は主要計画文書への入口としては機能しているが、`shared_nmd_test_plan.md`、`nmd_assert_output_plan.md`、`compiler_diagnostics_redesign_plan.md`、`stdlib_collection_mem_string_static_safety_design.md` が一覧に入っていない。実装判断で参照される重要文書が入口から辿りにくい。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `doc/README.md` | NEPLg2/NEPLg3 の入口。 | 良い。 |
| `doc/testing.md` | runner と test 配置を説明。 | `std/test` 節が古い。 |
| `doc/cli.md` | NEPLg2 CLI output 仕様。 | 現行対象を明示。 |
| `doc/neplg2` | selfhost/static check/diagnostics/std safety 計画。 | 重要文書の README 露出不足。 |
| `doc/neplg3` | 次世代仕様と実装設計。 | 実装 placeholder との差分が大きい。 |
| `doc/compare`, `doc/migration` | 移行導線。 | 今回は構成確認のみ。 |

## 推奨対応

- `doc/testing.md` の `std/test` 節を structured `TestReport` / stdout report / `exit_code` 方針へ更新する。
- `doc/neplg2/README.md` に共通 `.n.md`、assert report、diagnostics redesign、stdlib static safety design を追加する。
- NEPLg3 仕様の diagnostic 用語を `diag_code` / typed diagnostic enum へ揃える issue を必要に応じて追加する。
