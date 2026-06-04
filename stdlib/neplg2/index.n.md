# NEPLg2 Self-Host Compiler Index

NEPLg2.1 self-host compiler の入口です。Stage 0 では、NEPLg3 用 placeholder とは別の場所に NEPLg2.1 用の source tree と focused doctest 経路が存在することを固定します。

neplg2:test
```neplg2
#entry main
#target core
#indent 4

#import "neplg2/core/pipeline" as *

fn main %fn void i32 \void:
    selfhost_pipeline_stage0
```
