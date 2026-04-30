# レビュー妥当性の再レビュー

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 結論

今回の総レビューは、対象 commit、GitHub Actions run、open issue、主要 design doc、Rust compiler、stdlib、selfhost、tools、tests、tutorial、examples を章ごとに確認しており、現時点の進捗確認として妥当である。

ただし、main は Actions failure であり、review は「green な状態の承認」ではない。結論は「どこまで進めてよいか」と「どこをまだ固定してはいけないか」を分けるための判断である。

この文書による最終再レビュー後に remote main へ入った変更は、この `doc/fullreview20260430/` review の追従対象外とする。その後は通常の issue 解決・開発作業へ戻る。

## 進捗状況

| review 領域 | 状況 | 妥当性判断 |
|---|---|---|
| project / Actions / risk | 完了 | `gh` で Actions run と artifact を確認し、local test を review evidence にしていない。 |
| Rust compiler | 完了 | pipeline、diagnostic、parser/typecheck、Resource IR、backend、target gate を分けて確認した。 |
| stdlib | 完了 | core、mem、string、collections、std/fs/stdio、nm/kp/platform を分けて確認した。 |
| selfhost | 完了 | S0-S7 と実ファイルを照合し、開始可能範囲と禁止範囲を分けた。 |
| tools / tests / tutorial / examples | 完了 | CLI、nodesrc、language/LSP、web、`.n.md`、tutorial、examples を確認した。 |
| crosscutting | 完了 | static safety、stdlib readiness、diagnostics/tests/docs の横断判断を追加した。 |
| summary | 完了 | findings と selfhost readiness を最終整理した。 |

## 根拠の確認

test 状況は GitHub Actions を根拠にした。対象 run `25157230630` は `f108cebdf72289251b5d9f90c0fd7de4ca591e6e` の main push run で、conclusion は failure である。

使用した Actions evidence:

- `gh run list --repo neknaj/NEPLg2 --branch main --limit 5`
- `gh run view 25157230630 --json status,conclusion,headSha,headBranch,displayTitle,createdAt,updatedAt,jobs`
- `gh run view 25157230630 --job <job-id> --log-failed`
- artifact: `stdlib-tests.json`
- artifact: `nmd-tests.json`
- artifact: `tutorials-tests.json`
- artifact: `tests-current.json`
- artifact: `tests-dual-tests.json`
- artifact: `tests-dual-stdlib.json`

local test は review evidence にしていない。docs-only commit の事前確認として `node nodesrc/issues.js check`、`git diff --check`、`git diff --cached --check` は使った。

## finding の再確認

| finding | 再確認結果 |
|---|---|
| Actions は green ではない | `gh run list` で最新 main run `25157230630` が failure であることを再確認した。 |
| Resource IR は final authority ではない | design doc と Rust compiler review が一致している。旧 move_check / HIR drop insertion が残る判断は妥当。 |
| stdlib memory model は過渡期 | `core/mem`、`Vec`、string/io builder、collection review、stdlib safety design doc が同じ結論を示す。 |
| selfhost は S1/S2 限定で開始可能 | selfhost S0-S7 review、Actions artifact の timeout/owner failure、risk map が一致している。 |
| diagnostic / `.n.md` / assert は移行中 | diagnostic redesign plan、nmd assert output plan、quality review が一致している。 |
| tutorial / README / examples は追従対象 | Actions tutorial failure と project progress review が一致している。 |

## 見落としリスク

この review は repository 全体を章ごとに確認したが、全ソースの全行に対する形式的証明ではない。巨大 file では、設計上の責務、Actions failure、issue、source policy、representative source を中心に確認した。

残る見落としリスク:

- Actions artifact に出ていない未到達 path の owner/drop bug。
- source policy の対象外にある新しい raw memory / diagnostic string / hash dispatch pattern。
- Rust backend と LLVM backend の差分のうち、dual backend failure に表面化していないもの。
- selfhost stage0 smoke API が実処理へ置き換わる時に出る collection ownership bug。
- docs が増えたことで、将来の設計変更時に古い doc が残るリスク。

これらは「レビューの結論を否定するリスク」ではなく、次の issue 解決と CI policy 拡張で継続監視すべきリスクである。

## 更新追従の確認

レビュー中に remote main は複数回確認した。最終まとめ直前にも `git pull --ff-only origin main` を実行し、`Already up to date` であることを確認した。最新 main run は引き続き `25157230630` / `f108cebd` / failure である。

この最終再レビュー後に入る main 更新は、今回の `doc/fullreview20260430/` の追従対象外とする。通常開発へ戻る際は、新しい issue / commit / Actions run をその時点の作業として扱う。

## 最終判定

今回の review は、開発方針で求められた次の観点を満たしている。

- 技術的負債を隠さず、open issue と Actions failure として明示した。
- 暫定実装と暫定設計を分け、S3 以降の selfhost で固定してはいけない設計を明示した。
- 型安全とメモリ安全を必達として、Resource IR / owner token / stdlib memory model の未完了点を優先課題にした。
- enum / match / stable string boundary を横断観点として確認した。
- review の test 状況を local ではなく GitHub Actions で確認した。

したがって、この総レビューは 2026-04-30 `f108cebd` 時点の NEPLg2 進捗確認として採用してよい。
