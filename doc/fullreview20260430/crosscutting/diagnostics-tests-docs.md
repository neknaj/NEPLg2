# 診断・テスト・ドキュメント横断レビュー

## レビュー範囲

確認対象:

- 診断 ID: `nepl-core/src/diagnostic.rs`, `nepl-core/src/diagnostic_codes.rs`
- CI と test runner: `.github/workflows/ci.yml`, `nodesrc/tests.js`, `nodesrc/run_source_policy_regressions.js`
- stdlib test: `stdlib/std/test/**`
- tutorial/examples/doc の検査導線
- issue registry と GitHub Actions 実行状況

このレビューでは、ローカルでテストを新規実行して合否を作るのではなく、CI 状態は `gh run list` で確認した。

## 診断 ID

Rust compiler 側の診断 ID は enum registry 化されている。`DiagnosticCode` が compiler-owned な安定 ID であり、領域別の子 enum に分かれる。`DiagnosticSpec` は構築時点で `DiagnosticCode` を持ち、`Diagnostic` も code を保持するため、診断の根拠を文字列に後付けする設計ではない。

この方向性は、selfhost でも同じ ID を実装する前提として妥当である。文字列表現は表示、CLI、JSON、テスト snapshot などの外部境界に限定し、内部の分岐は enum と `match` に寄せるべきである。

残る問題は、Rust compiler の診断再設計が進行中で、selfhost 側の診断 model が完全には追従していない点である。selfhost の diagnostic ID は、Rust 側の `DiagnosticCode` 設計を写すのではなく、同じ分類と安定 ID の原則を持つ selfhost-native な enum registry として実装する必要がある。

関連 open issue:

- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`

## stdlib assert と `.n.md` テスト

`stdlib/std/test` は、assertion の評価、report aggregation、stdout 表示、exit code 変換が分離された構造へ進んでいる。`AssertionStatus`, `AssertionKind`, `TestAssertion`, `TestReport` が定義され、`assert_*` は直接終了や即時出力ではなく `TestAssertion` を返す。

`test_report_print_stdout` と `test_report_exit_code` が canonical な出力と終了コード変換の API になっているため、`.n.md` test を Rust/selfhost 共通に運用する基盤としては妥当である。

ただし旧運用の互換 helper と、main の返り値を test result として扱う経路が残っている。`finish_checks` や `result_exit_code` は移行補助として存在しており、最終形では stdout に構造化された assertion report を出し、exit code は可否だけにする方針へ統一するべきである。

関連 open issue:

- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD`

## CI と regression policy

GitHub Actions は `CI (NEPL-g2)` に集約され、build、compile test、Rust test、nm compile、WASI test、`.n.md` doctest、tutorials test、stdlib test、LLVM test、Pages まで広く実行している。concurrency は branch 単位で `cancel-in-progress: true` のため、main への連続 push では古い run が cancelled になる。

source policy regression は `nodesrc/run_source_policy_regressions.js --warn-only` として CI build に入っている。現在は warning gate なので、設計違反を検出しても CI fail にはならない。静的検査や enum/match 方針を必達にするには、policy の粒度を整理し、false positive を潰したうえで fail gate へ移す必要がある。

確認時点の Actions:

- `25509045688`: `docs(review): add quality tools review`, `in_progress`
- それ以前の selfhost/refactor run は後続 push により `cancelled`

この `cancelled` は workflow concurrency によるものなので、単独で失敗とは扱わない。ただし latest run が完了するまでは、review 文書では「CI 最終結果確認前」と明記する必要がある。

## tutorials/examples/docs

tutorial は現行仕様への追従が進んでいるが、`getting_started` の doctest failure が open issue として残っている。tutorial は言語仕様の利用者向け入口であり、selfhost や stdlib の API 変更に追従できていない場合、実装の到達点を誤って伝える。tutorial の compile/doctest は仕様文書の検査として扱う必要がある。

examples は一部 smoke compile が CI に存在するが、`examples/*.nepl` 全体の doctest ではない。CLI や stdlib の利用例は regression として重要なので、examples も `.n.md` と同様に stdout/assertion report へ寄せて CI に入れる必要がある。

関連 open issue:

- `ISS-20260507T161156205Z-GETTING-STARTED-TUTORIAL-DOCTESTS-FA-A0324153`
- `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895`

## 進捗状況

| 領域 | 状態 | 根拠 |
| --- | --- | --- |
| Rust diagnostic enum registry | 実装済み寄り | `DiagnosticCode` と `ALL_DIAGNOSTIC_CODES` がある |
| selfhost diagnostic ID | 設計追従が必要 | Rust 側設計に合わせる指示と open issue が残る |
| stdlib assert/report API | 実装済み寄り、移行中 | `TestReport` と stdout/exit code API はある |
| `.n.md` stdout assertion 運用 | 未完了 P1 | 旧 return-value 運用 issue が残る |
| source policy regression | CI 組込済み、warn-only | fail gate ではない |
| tutorials doctest | 一部未完了 P1 | getting_started failure issue が残る |
| examples doctest | 未完了 P2 | 全体 CI gate がない |
| Actions latest status | 確認中 | latest run は in_progress |

## 判断

診断 ID と stdlib test report の方向性は正しい。残る課題は「仕組みがある」段階から「必ず検査される」段階へ移すことにある。

優先して進めるべきこと:

1. selfhost diagnostic ID を Rust 側再設計後の分類、安定 ID、enum registry 原則に合わせる。
2. `.n.md` test の main-return 依存を廃止し、stdout assertion report と exit code に統一する。
3. source policy regression を warn-only から必須 gate に移せるように、既知例外を issue 化して潰す。
4. tutorials と examples を CI の仕様検査として扱い、サンプル drift を早期に検出する。
