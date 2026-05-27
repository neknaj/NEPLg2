# NEPLg2 ドキュメント

現行 Rust 実装としての NEPLg2 に関する設計・保守ドキュメントを置く。

2026-05-24 以降の大規模構文移行では、現行 NEPLg2 を NEPLg2.1 へ切り替える。NEPLg2.1 は `nepl-core/` と既存 `stdlib/` / `tests/` を発展させる移行であり、NEPLg3 実装ではない。NEPLg3 文書は今後も十分に変わり得る参考資料として扱う。

| ドキュメント | 内容 |
|---|---|
| [neplg21_syntax_migration_plan.md](./neplg21_syntax_migration_plan.md) | NEPLg2.1 表層構文移行計画。`%` 型注釈、prefix 型式、`\` 関数リテラル、generic postfix 撤廃の境界 |
| [self_host_plan.md](./self_host_plan.md) | NEPLg2.0 self-host compiler の詳細実装計画 |
| [self_host_execution_plan.md](./self_host_execution_plan.md) | branch、commit、merge、Rust 側修正合流、Issue 提出規則 |
| [pre_selfhost_audit_20260426.md](./pre_selfhost_audit_20260426.md) | self-host 開始前の Rust compiler / stdlib 監査と追加 Issue |
| [pre_selfhost_performance_audit_20260426.md](./pre_selfhost_performance_audit_20260426.md) | self-host 開始前の計算量・メモリ監査と追加 Issue |
| [compiler_performance_cache_design.md](./compiler_performance_cache_design.md) | NEPLg2.1 compile-time performance、Resource IR pruning、stdlib prechecked artifact、CompilerSession / incremental cache 設計 |
| [static_check_complexity_reduction_plan.md](./static_check_complexity_reduction_plan.md) | 静的検査の不必要な複雑化を Resource IR / owner token / internal effect 境界で解消する仕様と実装計画 |
| [char_stdlib_integration_plan.md](./char_stdlib_integration_plan.md) | char 型と言語 literal 追加後の stdlib API、string / UTF-8 / builder 連携、既存 code 移行計画 |
| [tutorial_rewrite_plan.md](./tutorial_rewrite_plan.md) | 現在の NEPLg2 に合わせて tutorial を章立て・コード例・検証方針から全面改訂する計画 |

NEPLg3 の次世代仕様・実装設計は [../neplg3/](../neplg3/README.md) を参照できる。ただし NEPLg2.1 移行中は、NEPLg3 文書を現在の正仕様として扱わない。
