# NEPLg2 compiler diagnostic redesign plan

作成日: 2026-04-29

## 目的

NEPLg2 の Rust compiler diagnostic は、初期の typecheck / parser エラーを前提にした数値 ID 中心の実装から始まっている。その後、Resource IR、internal effect、owner obligation、borrow lifetime、self-host compiler が増えたことで、現行の診断基盤は現在の設計に合わなくなっている。

この文書は、diagnostic を場当たり的に `D3102` などへ増やし続けるのではなく、現在の NEPLg2 設計に合わせて再設計するための仕様と実装計画を定める。

関連 issue:

- [ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D](../../issues/items/ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D.md): Rust compiler diagnostics が Resource IR / self-host model と揃っていない。
- [ISS-20260425T000000Z-RV-CORE-009-58589A3F](../../issues/items/ISS-20260425T000000Z-RV-CORE-009-58589A3F.md): Resource IR 上の move / borrow / drop 検査。
- [ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04](../../issues/items/ISS-20260427T132406497Z-CORE-MEM-RAW-MEMORY-OPERATIONS-BYPAS-DC67DF04.md): raw memory effect / ownership boundary。
- [ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF](../../issues/items/ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF.md): `MemPtr` / owner token / initialized cell の分離。
- [NEPLg2 静的検査の複雑化解消計画](./static_check_complexity_reduction_plan.md): Stage 4/5 の Resource IR authoritative gate。

## 現状の問題

### 1. 数値 ID が意味体系ではなく履歴になっている

`nepl-core/src/diagnostic_ids.rs` は `D1000` 台を loader / lexer、`D2000` 台を parser、`D3000` 台を type / effect / move / borrow / Resource IR、`D4000` 台を backend に割り当てている。一方で、Resource IR の新しい診断は `D3025`、`D3100`、`D3101` へ押し込まれている。

この状態では、次の区別を ID だけで表せない。

- type error と effect boundary error。
- raw identity escape と ordinary impure call。
- owner obligation leak と initialized cell violation。
- borrow lifetime escape と active borrow conflict。
- Resource IR lowering 欠落と Resource IR checker violation。

`D3100` は特に範囲が広く、raw memory、owner、cell state、storage destructive op の複数問題を同じ bucket で表す。これは test では便利だが、診断設計としては粗すぎる。

### 2. Rust core と self-host の diagnostic model が分岐している

Rust core の `Diagnostic` は `severity`、任意の数値 `id`、任意の `code`、message、primary label、secondary labels を持つ。実際にはほとんどの call site が `Diagnostic::error(...).with_id(...)` を直接呼び、`code` は一部 backend 以外ほぼ使われていない。

self-host 側の `SelfhostDiagnostic` は、stable string `code`、message、primary label、note を中心に設計されている。CLI reporter も human / JSON rendering を分けているため、最終的な self-host parity では string code を軸に比較するほうが自然である。

Rust core が数値 ID だけを主語にしたままでは、self-host compiler へ移す時に再び変換層を作る必要がある。

### 3. Diagnostic construction が各 pass に散らばっている

lexer、parser、typecheck、compiler gate、backend が、それぞれ message と ID を直接組み立てている。Resource IR では `ResourceCheckDiagnostic` / `ResourceOwnerDiagnostic` / `ResourceBorrowDiagnostic` / `ResourceEffectBoundaryDiagnostic` から compiler diagnostic への変換が `compiler.rs` に集まりつつある。

この構造では、Resource IR の意味的な診断分類が call site の ad-hoc mapping に吸収される。たとえば Resource IR が `CellUnavailable` を検出しても、compiler error にするか、どの ID にするか、どの note を出すかが変換関数ごとに散らばる。

### 4. 表示と機械判定の境界が弱い

CLI は `error[Dxxxx][code]: message` を表示し、secondary label を `note:` として出す。だが diagnostic value には dedicated note/help/suggestion/related diagnostic がない。nodesrc doctest は `diag_id` / `diag_ids` で数値だけを確認する。

そのため、将来の migration で message や label を改善すると、test が粗い数値 bucket のままになり、逆に意味的な code の regression を捕まえにくい。

## 目標仕様

### Diagnostic の安定識別子

今後の主識別子は string code とする。形式は次を基本にする。

| 領域 | 例 | 意味 |
|---|---|---|
| `loader.*` | `loader.source.not_found` | source / VFS / target gate |
| `lexer.*` | `lexer.string.unterminated` | byte lexer |
| `parser.*` | `parser.token.expected` | syntax parser |
| `resolve.*` | `resolve.name.undefined` | name / import / scope |
| `type.*` | `type.argument.mismatch` | type inference / overload |
| `effect.*` | `effect.pure.calls_impure` | surface effect |
| `resource.lower.*` | `resource.lower.unknown_place` | Resource IR lowering completeness |
| `resource.cell.*` | `resource.cell.moved` | initialized / moved / dropped state |
| `resource.owner.*` | `resource.owner.leaked` | free obligation owner |
| `resource.borrow.*` | `resource.borrow.unique_conflict` | borrow / lifetime |
| `resource.raw.*` | `resource.raw.identity_escape` | raw identity / unsafe memory boundary |
| `backend.wasm.*` | `backend.wasm.signature.unsupported` | WASM backend |
| `backend.llvm.*` | `backend.llvm.hir.unsupported` | LLVM backend |

数値 `DiagnosticId` は互換層として残す。既存 doctest の `diag_id` は即時削除せず、新しい `diag_code` / `diag_codes` を追加して段階移行する。

### Diagnostic value

Rust core の diagnostic は self-host 側へ移植可能な形へ寄せる。

必要な field:

- `severity`
- `code`: stable string code
- `legacy_id`: optional numeric ID
- `message`
- `primary_label`
- `secondary_labels`
- `notes`
- `help`
- `stage`: loader / lexer / parser / resolve / type / effect / resource / backend
- `category`: type / effect / resource cell / resource owner / resource borrow など

初期実装では既存 `Diagnostic` へ破壊的変更を入れず、builder / registry で `code` と `legacy_id` を同時に付ける。self-host 側の `SelfhostDiagnostic` はこの subset として扱う。

### Registry

diagnostic 定義は単一 registry に寄せる。Rust では最初に static table または macro table として実装し、次を機械的に検査する。

- code が重複しない。
- legacy numeric ID が重複しない。
- legacy ID 付き diagnostic は既存 `DiagnosticId::from_u32` と一致する。
- message fallback が空でない。
- category / stage が必ず設定される。

将来的には self-host 側でも同じ code 一覧を持つ。少なくとも string code は Rust と NEPLg2 self-host の shared contract として扱う。

### Resource IR diagnostic mapping

Resource IR の diagnostic は、compiler.rs の ad-hoc 関数で直接 legacy ID に落とさず、次の順に変換する。

1. Resource IR が typed diagnostic kind を返す。
2. Resource diagnostic kind を stable diagnostic code に写像する。
3. registry が legacy numeric ID と message fallback を補う。
4. CLI / nodesrc / web は code を主、legacy ID を互換表示として使う。

例:

| Resource IR diagnostic | Stable code | Legacy ID |
|---|---|---:|
| lowering unknown place | `resource.lower.unknown_place` | D3101 |
| raw destructive live cell | `resource.cell.raw_destructive_live_cell` | D3100 |
| read moved cell | `resource.cell.moved` | D3053 / D3100 は移行方針で決める |
| owner leak | `resource.owner.leaked` | D3100 |
| owner maybe leak | `resource.owner.maybe_leaked` | D3100 |
| unique borrow read conflict | `resource.borrow.unique_read_conflict` | D3052 |
| borrow return escape | `resource.borrow.return_escape` | D3099 |
| raw identity escape from pure | `resource.raw.identity_escape` | D3025 |

既存 `diag_id` は当面維持するが、Resource IR の内部診断名を失わないように `diag_code` を regression に追加していく。

## 実装計画

### Stage D0: audit と互換境界の固定

目的: 既存の `diag_id` regression を壊さず、移行先の境界を明確にする。

作業:

- `DiagnosticId` の全 variant に stable string code を割り当てる一覧を作る。
- `nodesrc/parser.js` / `nodesrc/tests.js` に `diag_code` / `diag_codes` の metadata を追加する計画を固定する。
- `Diagnostic::with_code` が既存表示・JSON にどう出るかを確認する。

commit 単位:

1. registry data structure と consistency test。
2. `DiagnosticId -> code` の互換 mapping。
3. nodesrc doctest metadata の code 受け入れ。

### Stage D1: Diagnostic builder の導入

目的: call site が message と ID を直接組み合わせる状態を止める。

作業:

- `DiagnosticBuilder` または `DiagnosticSpec` を導入する。
- `Diagnostic::error(...).with_id(...)` をすぐ消すのではなく、新規 code path は builder を使う。
- note/help/related label を value として保持できる拡張点を作る。

commit 単位:

1. builder API と unit test。
2. lexer/parser の代表診断を builder に移す。
3. typecheck の代表診断を builder に移す。

### Stage D2: Resource IR diagnostic の typed mapping

目的: Stage 4/5 の Resource IR gate が ad-hoc legacy ID mapping に依存しないようにする。

作業:

- `ResourceDiagnosticCode` または registry code を Resource IR diagnostic kind に持たせる。
- `resource_*_diagnostic_to_error` は code を先に決定し、legacy ID は registry から補う。
- owner / cell / borrow / raw effect の code を分ける。

commit 単位:

1. lowering coverage diagnostics。
2. cell diagnostics。
3. owner diagnostics。
4. borrow diagnostics。
5. raw effect diagnostics。

### Stage D3: CLI / JSON / web 表示の整理

目的: 人間向け表示と機械判定を同じ diagnostic value から安定生成する。

作業:

- CLI 表示は `error[resource.owner.leaked][D3100]: ...` のように code を主表示にする。ただし当面は既存表示との互換を検討し、必要なら order は段階的に変える。
- JSON 出力に `code`、`legacy_id`、`stage`、`category`、`primary`、`notes` を出す。
- web / editor / LSP 向け API は code を primary key とする。

commit 単位:

1. CLI rendering compatibility tests。
2. JSON diagnostic output。
3. web/editor diagnostic adapter。

### Stage D4: test migration

目的: regression が legacy bucket だけに依存しないようにする。

作業:

- `diag_code` / `diag_codes` を doctest metadata として追加する。
- 新規 Resource IR / effect / owner / borrow regression は code を必須にする。
- 既存 `diag_id` は削除せず、legacy compatibility test として残す。

commit 単位:

1. nodesrc parser / runner。
2. Resource IR gate tests の code 追加。
3. lexer/parser/typecheck の代表 tests の code 追加。

### Stage D5: self-host parity

目的: Rust core と NEPLg2 self-host compiler が同じ diagnostic contract を使う。

作業:

- `SelfhostDiagnostic` の code 命名を Rust registry と揃える。
- self-host reporter JSON と Rust CLI JSON を比較できる形にする。
- self-host parser / resolver / checker の diagnostic code を registry 方針へ合わせる。

commit 単位:

1. self-host diagnostic code naming audit。
2. reporter JSON parity fixture。
3. parser / loader diagnostic parity tests。

## 静的検査大規模修正との関係

この再設計は `static_check_complexity_reduction_plan.md` の Stage 4/5 を止めるためのものではない。むしろ、Stage 4/5 の Resource IR gate をこれ以上 ad-hoc な `D3100` / `D3025` mapping へ押し込まないための前提である。

当面の方針:

- memory safety / type safety / effect safety の gate は弱めない。
- 既に authoritative 化した Resource IR gate は維持する。
- 新規 gate を追加する時は、可能なら stable code を同時に設計する。
- legacy `diag_id` は既存回帰の互換条件として残す。
- self-host 実装開始前に、少なくとも registry と Resource IR diagnostic code 方針を固定する。

## 完了条件

1. Rust core diagnostic に stable string code が必ず付く。
2. 既存 `diag_id` regression が legacy compatibility として維持される。
3. Resource IR diagnostic が cell / owner / borrow / raw effect / lowering の意味分類を失わない。
4. CLI / JSON / nodesrc test が code を主識別子として扱える。
5. self-host `SelfhostDiagnostic` と Rust core diagnostic が同じ code contract で比較できる。
6. 新しい memory safety / type safety / effect safety issue を、粗い `D3100` bucket だけで追跡しなくてよい。
