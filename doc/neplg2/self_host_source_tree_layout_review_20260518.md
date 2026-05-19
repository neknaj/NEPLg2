# NEPLg2 self-host source tree layout review

作成日: 2026-05-18

## 目的

NEPLg2 self-host compiler の実装をさらに進める前に、現行 Rust compiler の分量と構造を再確認し、`stdlib/neplg2/` 側で採用するディレクトリ構造とファイル分割方針を固定する。

この文書は [self_host_plan.md](./self_host_plan.md) の「推奨ディレクトリ構造」を、2026-05-18 時点の Rust 実装と静的検査大規模修正の進捗に合わせて具体化する。目的は Rust 実装の flat さを NEPL 側に移植しないことである。

関連 issue:

- [ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD](../../issues/items/ISS-20260425T000000Z-RV-STDLIB-008-F4BCB5DD.md): NEPLg2 self-host compiler が部分実装に留まっている。
- [ISS-20260518T140937691Z-SELF-HOST-SOURCE-TREE-PLAN-MUST-BE-R-5C746649](../../issues/items/ISS-20260518T140937691Z-SELF-HOST-SOURCE-TREE-PLAN-MUST-BE-R-5C746649.md): self-host source tree plan を現行 Rust compiler 構造に合わせて再検証する。
- [ISS-20260519T204942256Z-SELF-HOST-CHECKER-LACKS-A-GENERIC-PR-35D60062](../../issues/items/ISS-20260519T204942256Z-SELF-HOST-CHECKER-LACKS-A-GENERIC-PR-35D60062.md): self-host checker に汎用 proof entry point を追加し、module item span 検査を typed fact / obligation / solver 境界へ接続する。
- [static_check_complexity_reduction_plan.md](./static_check_complexity_reduction_plan.md): Resource IR / owner / initialized / borrow / effect の複雑化解消計画。
- [compiler_diagnostics_redesign_plan.md](./compiler_diagnostics_redesign_plan.md): diagnostic code を階層 enum にする計画。
- [stdlib_documentation_contract_plan.md](./stdlib_documentation_contract_plan.md): stdlib documentation comment / doctest 整備方針。

## 結論

self-host compiler は `core/` と `cli/` の二層分割だけでは不十分である。`core/` の内部も pipeline stage、typed model、generic proof engine、Resource IR subdomain ごとに階層化する。

特に次の方針を固定する。

- root module は orchestration または public facade に限定し、巨大な実装本体を置かない。
- parser、typecheck、Resource IR、codegen は Rust 側の root flat file をそのまま移植しない。
- 静的検査の証明器は `core/proof/` に汎用基盤として置き、owner / initialized / borrow / effect / abstraction check が個別に証明器を持たない。
- stdlib や module 名の allowlist ではなく、source code、typed AST/HIR、typed capability、Resource IR fact から性質を証明する。
- 新規 self-host 実装は最初から最終階層に置く。旧 path 互換のためだけの wrapper は作らない。facade が必要な場合も実装を持たない public API 境界に限定する。
- ドキュメントコメントや doctest は削らない。ファイルが大きくなる場合は責務で分割し、説明を落とさない。

## 現行 Rust 実装の分量

2026-05-18 時点の `nepl-core/src` は 382 files / 約 79,607 行である。大きい領域は次の通り。

| 領域 | files | lines | 評価 |
|---|---:|---:|---|
| `resource/` | 271 | 37,090 | subdomain は分割されているが、`mod.rs` が巨大な export hub になり、owner / initialized / effect / lower / summary が同一階層に並びすぎている。NEPL 側ではさらに階層を切る。 |
| root 直下 | 36 | 27,732 | `parser.rs`、`codegen_llvm.rs`、`codegen_wasm.rs`、`compiler.rs`、`types.rs`、`loader.rs` などが root にあり、依存方向が見えにくい。NEPL 側では root 直下に実装本体を置かない。 |
| `typecheck/` | 39 | 11,332 | `typecheck.rs` は facade 化済みだが、`prefix_check.rs` や `driver.rs` が大きい。NEPL 側では `check/` と `abstraction/` と `proof/` に責務を分ける。 |
| `source_capability/` | 23 | 1,585 | static proof の一部が compiler source proof として分離された。NEPL 側では `proof/source/` として汎用証明器に接続する。 |

大きい root file は次の通り。

| file | lines | self-host 側での扱い |
|---|---:|---|
| `nepl-core/src/parser.rs` | 4,036 | `syntax/parser/` 配下へ module / item / expr / type / pattern / directive / raw backend で分割する。 |
| `nepl-core/src/codegen_llvm.rs` | 3,847 | `codegen/llvm/` 配下へ module / type map / aggregate / scalar / intrinsic / text emitter で分割する。 |
| `nepl-core/src/codegen_wasm.rs` | 2,392 | `codegen/wasm/` 配下へ module / section / instr / local / data / leb128 / runtime で分割する。 |
| `nepl-core/src/compiler.rs` | 2,183 | `pipeline/` または `core/pipeline.nepl` は orchestration のみとし、各 stage result を専用 module に置く。 |
| `nepl-core/src/types.rs` | 2,044 | `ty/` と `abstraction/` へ分割し、trait / generic / type constructor / layout を混在させない。 |
| `nepl-core/src/loader.rs` | 2,011 | `module/physical`、`module/syntax_graph`、`module/logical`、`cli/file_io` に分ける。 |
| `nepl-core/src/monomorphize.rs` | 1,358 | `mono/instance`、`mono/subst`、`mono/worklist`、`mono/trait_lookup` に分ける。 |
| `nepl-core/src/diagnostic_codes.rs` | 1,313 | `infra/diag/code/` に category ごとに分け、string 化は exhaustive match に限定する。 |
| `nepl-core/src/lexer.rs` | 1,218 | `syntax/lexer/` 配下へ state / indent / literal / directive / raw mode / tokenize で分割する。 |

## 現行 `stdlib/neplg2/` の分量

2026-05-18 時点の self-host tree は 39 files / 約 10,239 行である。まだ部分実装だが、既に大きい file が出ている。

| file | lines | 対応方針 |
|---|---:|---|
| `core/syntax/lexer.nepl` | 1,329 | `syntax/lexer/` 配下に state、indent、literal、directive、raw mode、tokenize を分ける。 |
| `core/hir/hir.nepl` | 1,036 | `hir/` 配下に id、expr、function、module、arena、range、lower を分ける。 |
| `core/syntax/token.nepl` | 864 | `syntax/token/` 配下に kind、token value、display/name、directive classification を分ける。 |
| `core/ty/ty.nepl` | 769 | `ty/` 配下に id、kind、record、arena、eq、function type を分ける。 |
| `core/syntax/parser/module_parser.nepl` | 603 | `syntax/parser/` 配下に module loop、directive parser、raw block parser、item start classifier を分ける。 |
| `core/infra/diag.nepl` | 361 | `infra/diag/` 配下に code、value、collection、query を分ける。 |

これらは「今すぐ全分割してから実装する」という意味ではない。だが、今後の実装をこれらの巨大 file に直接足さない。関連 issue を解く commit では、必要な責務単位を切り出してから実装する。

## 目標ディレクトリ構造

```text
stdlib/neplg2/
    core/
        infra/
            source/
                file_id.nepl
                span.nepl
                text.nepl
                source_map.nepl
            diag/
                code/
                    loader.nepl
                    lexer.nepl
                    parser.nepl
                    resolve.nepl
                    type.nepl
                    trait.nepl
                    effect.nepl
                    resource.nepl
                    backend.nepl
                    cli.nepl
                code.nepl
                value.nepl
                collection.nepl
                outcome.nepl

        syntax/
            token/
                kind.nepl
                token.nepl
                name.nepl
                directive.nepl
            lexer/
                state.nepl
                indent.nepl
                literal.nepl
                directive.nepl
                raw_mode.nepl
                tokenize.nepl
            ast/
                module.nepl
                item.nepl
                expr.nepl
                type_expr.nepl
                pattern.nepl
                directive.nepl
                raw_backend.nepl
            parser/
                module.nepl
                item.nepl
                expr.nepl
                type_expr.nepl
                pattern.nepl
                signature.nepl
                directive.nepl
                raw_backend.nepl

        module/
            physical/
                path.nepl
                vfs.nepl
                stdlib_map.nepl
            syntax_graph/
                import_spec.nepl
                import_scan.nepl
                graph.nepl
            logical/
                module_id.nepl
                visibility.nepl
                export_table.nepl
                facade_merge.nepl
            loader.nepl

        resolve/
            def_id.nepl
            def_table.nepl
            scope.nepl
            hoist.nepl
            import_resolver.nepl
            name_resolver.nepl
            visibility.nepl

        ty/
            id.nepl
            kind.nepl
            record.nepl
            arena.nepl
            unify.nepl
            subst.nepl
            layout.nepl
            type_expr_reduce.nepl

        abstraction/
            generic_param.nepl
            trait_def.nepl
            trait_bound.nepl
            impl_table.nepl
            coherence.nepl
            overload.nepl
            dictionary.nepl

        proof/
            fact.nepl
            obligation.nepl
            evidence.nepl
            lattice.nepl
            solver.nepl
            worklist.nepl
            query.nepl
            source/
                capability.nepl
                exact_use_site.nepl
                raw_memory.nepl
                owner_aggregate.nepl
                compiler_memory.nepl

        check/
            driver.nepl
            module.nepl
            decl.nepl
            expr/
                reduce.nepl
                check.nepl
                call.nepl
                field.nepl
                control.nepl
            pattern.nepl
            trait.nepl
            effect.nepl
            diagnostics.nepl

        hir/
            id.nepl
            range.nepl
            expr.nepl
            function.nepl
            module.nepl
            arena.nepl
            lower.nepl
            walk.nepl

        resource/
            ir/
                id.nepl
                place.nepl
                op.nepl
                block.nepl
                function.nepl
                module.nepl
            lower/
                lower.nepl
                call.nepl
                aggregate.nepl
                match.nepl
                raw_memory.nepl
                condition.nepl
            owner/
                state.nepl
                transfer.nepl
                summary.nepl
                extent.nepl
                variant.nepl
                check.nepl
            init/
                cell.nepl
                alias.nepl
                range.nepl
                summary.nepl
                check.nepl
            borrow/
                state.nepl
                scope.nepl
                usage.nepl
                check.nepl
            effect/
                op.nepl
                summary.nepl
                boundary.nepl
                check.nepl
            drop/
                requirement.nepl
                plan.nepl
                elaboration.nepl
            host/
                memory_contract.nepl
                dependent_length.nepl
                external_io.nepl
            report.nepl

        mono/
            instance.nepl
            subst.nepl
            worklist.nepl
            trait_lookup.nepl
            mono.nepl

        codegen/
            wasm/
                module.nepl
                section.nepl
                instr.nepl
                local.nepl
                data.nepl
                leb128.nepl
                layout.nepl
                intrinsic.nepl
                runtime.nepl
            llvm/
                module.nepl
                type_map.nepl
                expr.nepl
                aggregate.nepl
                scalar.nepl
                intrinsic.nepl
                text.nepl

        pipeline/
            request.nepl
            stage.nepl
            artifact.nepl
            compile.nepl
        pipeline.nepl
        options.nepl

    cli/
        args/
            types.nepl
            classify.nepl
            parse.nepl
            emit.nepl
            options.nepl
        reporter/
            human.nepl
            json.nepl
            collection.nepl
        file_io/
            read.nepl
            write.nepl
            vfs_build.nepl
        driver.nepl
        main.nepl
```

## 分割原則

### 1. stage と data model を混ぜない

`ty/record.nepl` は型表現だけを持つ。`check/expr/check.nepl` は式の検査だけを持つ。`resource/ir/place.nepl` は Resource IR の place 表現だけを持つ。表現と検査を同じ file に詰め込まない。

### 2. root file は実装置き場にしない

`core/pipeline.nepl`、`core/check/driver.nepl`、`core/resource/report.nepl` のような root 寄り file は、依存方向を示す orchestration と public API 境界に限定する。

### 3. 証明器は汎用基盤にする

owner、initialized、borrow、effect、source capability、abstraction coherence がそれぞれ別々の ad hoc 証明器を持つ設計は禁止する。各 stage は typed fact と obligation を `core/proof/` に渡し、`proof/solver.nepl` と `proof/query.nepl` を通して検査する。

個別 module は「証明器」ではなく「fact producer」または「obligation producer」である。たとえば `resource/owner/check.nepl` は owner obligation を生成するが、source code を再走査して独自に raw helper 名を推測しない。

### 4. enum / match で静的検査が効く構造にする

diagnostic code、Resource IR op、effect op、type kind、trait obligation、proof fact、proof obligation は enum で表す。stable string は reporter 境界でだけ作る。wildcard arm は原則使わず、新 variant 追加時に compile error または source policy で未対応が露出するようにする。

### 5. stdlib allowlist を性質の証明にしない

`core/mem/internal` や特定関数名を列挙して安全とみなす設計は禁止する。特権的な操作は、source capability、typed definition identity、use-site proof、Resource IR fact、owner extent proof のいずれかで証明する。

### 6. ファイルの大きさは責務で制御する

行数だけを理由にドキュメントコメントを削らない。目安は次の通り。

| 種別 | 目安 | 超過時の対応 |
|---|---:|---|
| pure model / enum file | 400-700 lines | enum category や payload type で分割する。 |
| pass implementation | 500-900 lines | walker、obligation producer、diagnostic builder、helper を分ける。 |
| facade / orchestration | 200-500 lines | 実装が混ざっていないか確認する。 |
| doctest-heavy file | 行数のみで問題視しない | ドキュメントを削らず、API group で分ける。 |

1 file が 900 行を超える場合は、次の commit で同じ file に新機能を足す前に分割 issue を立てる。ただし exhaustive enum mapping のように責務が単一であり、分割すると網羅性が弱くなる場合は例外として doc に理由を書く。

### 7. doctest と source policy を階層に合わせる

各 directory は最低 1 つの `.n.md` integration doctest または module doctest を持つ。静的検査・抽象化・Resource IR については、正常系だけでなく compile_fail / diagnostic code / stdout report を固定する。

source policy は implementation detail の文字列検索だけにしない。どうしても Node.js source policy で監視する場合は、再導入してはいけない構造を明示する。

## self-host 実装へ戻る前の適用方針

今後の self-host 実装はこの順で進める。

1. 新しい実装を既存の巨大 file に足す前に、置き場所がこの文書の最終階層に一致しているか確認する。
2. checker 入口を作る場合、`core/check/checker.nepl` に実装本体を置かず、`core/check/module.nepl` または `core/check/expr/*` に置く。
3. module item stream の検査は typed AST/HIR に進むための入口であり、後続の type/effect/resource proof を代替しない。
4. Resource IR 周辺は `core/proof/` の fact/obligation/solver を前提にする。owner / initialized / borrow / effect の各 checker は同じ proof query を使う。
5. Rust 側の flat file を読みながら移植する場合、Rust file 単位ではなく責務単位で NEPL module に落とす。
6. 移植中に Rust 側の設計不備を見つけた場合は self-host 側で workaround せず、Rust core issue として切り出す。

## 直近の実装優先順位

### P0: layout plan の確定

この文書を self-host 実装前の gate とする。`self_host_plan.md` と self-host parent issue から参照する。

### P1: check / proof の入口を最終階層で作る

次の self-host checker 作業は `core/check/checker.nepl` を巨大化させない。module-level checker summary を作る場合も、実装は `core/check/module.nepl` に置き、`checker.nepl` は orchestration API に限定する。

同時に `core/proof/` の最小 model を先に置くか、少なくとも checker issue に「汎用 proof engine へ接続する未完了点」を明記する。個別 checker を証明器として増やさない。

2026-05-20 時点で、`ISS-20260519T204942256Z-SELF-HOST-CHECKER-LACKS-A-GENERIC-PR-35D60062` により `core/proof.nepl` と `core/proof/{fact,obligation,query,solver}.nepl` を追加した。初期 solver は source span validity のみを扱うが、fact / obligation / result kind は enum payload として定義し、`check/module.nepl` は `source_span_is_valid` を直接呼ばず proof query を通す。これは module checker を proof engine にするためではなく、今後の type / trait / effect / resource obligation を同じ `core/proof/` 境界へ載せるための入口である。

残る P1 作業は、raw backend block 状態、declaration well-formedness、trait coherence、effect boundary、Resource IR owner/initialized/borrow obligation を順次 `SelfhostProofFact` / `SelfhostProofObligation` の domain として追加し、`check/*` / `resource/*` は fact producer または obligation producer に限定することである。

### P2: 既存巨大 self-host file の分割 issue を順に処理する

`lexer.nepl`、`hir.nepl`、`token.nepl`、`ty.nepl`、`module_parser.nepl` は今後の実装追加前に分割候補として扱う。全分割を一度に行わず、次に触る責務から final path へ移す。

### P3: abstraction / generics / trait 設計を `abstraction/` と `ty/` に分ける

generic parameter、trait definition、trait bound、impl table、coherence、dictionary/instance model は `types.rs` 相当の一枚岩にしない。抽象化機能にも static verification policy を適用し、trait coherence や bound satisfaction は `proof/` に obligation として渡す。

## 完了条件

- self-host 追加実装の前に、この文書の target tree と異なる場所へ新規 pass logic を置かない。
- `core/proof/` を通さない ad hoc proof path を新規に作らない。
- `checker.nepl`、`pipeline.nepl`、`resource/mod` 相当の file を巨大な実装置き場にしない。
- file split は doctest と documentation comment を保ったまま行う。
- issue には、この文書のどの directory / stage に対応するかを記録する。
