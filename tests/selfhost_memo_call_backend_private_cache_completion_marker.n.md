# Self-host memo call backend traversal completion marker

Production resource-lowering境界がcompletion未発行、under-emit、over-emitのsplit ownerをtyped errorで拒否し、walker inputとobservation ownerを回収することを実行時に確認します。

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "neplg2/core/codegen/memo_call_backend_private_cache_proof_gate" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    match selfhost_memo_call_backend_private_cache_actual_traversal_completion_boundary_stage0:
        Result::Ok accepted:
            let coverage_transport_ok %bool selfhost_memo_call_backend_private_cache_actual_traversal_private_effect_coverage_event_shape_stage0;
            let traversal_scope_ok %bool selfhost_memo_call_backend_private_cache_resource_lowering_traversal_scope_stage0;
            let resource_ir_inventory_scope_ok %bool selfhost_memo_call_backend_private_cache_resource_ir_inventory_scope_stage0;
            test_assertion_exit_code assert_ne_bool "completion cases rejected and typed scopes transported" false and accepted and coverage_transport_ok and traversal_scope_ok resource_ir_inventory_scope_ok
        Result::Err _error:
            test_assertion_exit_code assert_ne_bool "completion setup succeeded" false false
```
