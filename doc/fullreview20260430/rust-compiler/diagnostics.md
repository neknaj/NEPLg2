# Rust コンパイラ diagnostics レビュー

確認対象 commit: `e8a4e399 docs(review): add check ResourceIR gate issue`

## 確認範囲

- `nepl-core/src/diagnostic.rs`
- `nepl-core/src/diagnostic_codes.rs`
- `nepl-cli/src/main.rs`
- `nodesrc/test_diagnostic_code_first_boundary.js`
- `nodesrc/test_selfhost_diag_code_enum.js`
- `doc/neplg2/compiler_diagnostics_redesign_plan.md`

このレビューでは、diagnostic id が文字列後付けではなく enum-first になっているか、Rust compiler と selfhost の将来設計が同じ taxonomy に向かっているかを確認した。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `Diagnostic.code` | mandatory `DiagnosticCode`。`Option<DiagnosticCode>` ではない。 | 方針に合う。code-less diagnostic は source policy で拒否される。 |
| code taxonomy | Loader / Lexer / Parser / Resolve / Type / Effect / Resource / Backend に階層化。Resource は Move / Borrow / Cell / Owner / Raw / Lower に分かれる。 | 良い。ResourceIR gate の原因分類と対応している。 |
| enum conversion | `as_str` と `message` は wildcard なしの exhaustive match。 | 良い。新 code 追加時に網羅性が効く。 |
| registry | `ALL_DIAGNOSTIC_CODES` が leaf code を列挙する。 | 良い。source policy が registry 漏れを検出する。 |
| CLI renderer | mandatory diagnostic code を前提に render する方向へ修正済み。 | 良い。過去の code optional 前提へ戻さないこと。 |
| selfhost | `nodesrc/test_selfhost_diag_code_enum.js` が selfhost 側の raw string diagnostic code を拒否する。 | 方向は良い。selfhost S3+ の Type/Effect/Resource code 拡張は未完。 |

## 良い点

- `Diagnostic::error(...)` / `warning(...)` の code-less constructor を禁止し、`error_with_code` / typed helper へ寄せている。
- ResourceIR の cell/owner/borrow/lower diagnostic が stable dotted code と enum の両方で保持されている。
- `diagnostic_codes.rs` の match に wildcard を使わないため、診断種別追加時に `as_str` / `message` の漏れが compile-time に見えやすい。
- selfhost 側も diagnostic code enum と renderer boundary を source policy で監視している。

## 問題

### selfhost S3 以降の taxonomy がまだ十分ではない

現行 selfhost の診断 infrastructure は S1/S2 の loader/lexer/parser/resolve 相当を中心に進んでいる。今後 typecheck、effect、ResourceIR、backend へ進むとき、Rust 側の `DiagnosticCode` 設計に従い、文字列 code を直接持たせない必要がある。

### diagnostic と `--check` の意味がずれる

Rust compiler diagnostics は code-first へかなり進んでいるが、`--check` が ResourceIR gate を通らないため、`resource.*` diagnostic を check-only UX で確認できない。このため diagnostic taxonomy の品質とは別に、検査 pipeline の入口をそろえる必要がある。

## issue 連携

- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`: open。Rust compiler diagnostics と ResourceIR/selfhost model の整合。
- `ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF`: open。`--check` が `resource.*` diagnostic gate を通らない。

## 次に確認すること

- selfhost diagnostics file が Rust 側 taxonomy と同じ enum-first / match-exhaustive policy に従えるか。
- `.n.md` doctest の expected diagnostic metadata が Rust/selfhost で共有できる形式になっているか。
- CLI / web / LSP / editor の diagnostic renderer が mandatory code を前提に揃っているか。
