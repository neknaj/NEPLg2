# NEPLg2.0 ドキュメント

現行実装としての NEPLg2.0 に関する設計・保守ドキュメントを置く。

| ドキュメント | 内容 |
|---|---|
| [self_host_plan.md](./self_host_plan.md) | NEPLg2.0 self-host compiler の詳細実装計画 |
| [self_host_execution_plan.md](./self_host_execution_plan.md) | branch、commit、merge、Rust 側修正合流、Issue 提出規則 |
| [pre_selfhost_audit_20260426.md](./pre_selfhost_audit_20260426.md) | self-host 開始前の Rust compiler / stdlib 監査と追加 Issue |
| [pre_selfhost_performance_audit_20260426.md](./pre_selfhost_performance_audit_20260426.md) | self-host 開始前の計算量・メモリ監査と追加 Issue |
| [static_check_complexity_reduction_plan.md](./static_check_complexity_reduction_plan.md) | 静的検査の不必要な複雑化を Resource IR / owner token / internal effect 境界で解消する仕様と実装計画 |
| [char_stdlib_integration_plan.md](./char_stdlib_integration_plan.md) | char 型と言語 literal 追加後の stdlib API、string / UTF-8 / builder 連携、既存 code 移行計画 |

NEPLg3 の次世代仕様・実装設計は [../neplg3/](../neplg3/README.md) を参照する。
