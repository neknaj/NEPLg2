# Self-host Resource IR Place projection

Rust PlaceProjection / ResourceOffsetの全variant構造と主要なinvalid payload rejectionをruntimeで確認します。

neplg2:test
```neplg2
#entry main
#indent 4
#target std
#import "neplg2/core/codegen/resource_ir_place_projection" as *
#import "std/test" as *
fn main %fn void i32 \void:
    test_assertion_exit_code assert_ne_bool "projection model validates all variants" false selfhost_resource_ir_place_projection_stage0
```
