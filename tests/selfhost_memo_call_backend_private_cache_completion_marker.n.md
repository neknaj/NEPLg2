# Self-host memo call backend traversal completion marker

Production resource-lowering境界がcompletion未発行、under-emit、over-emitのsplit ownerをtyped errorで拒否し、walker inputとobservation ownerを回収することを実行時に確認します。

neplg2:test
```neplg2
#entry main
#indent 4
#target std

#import "neplg2/core/codegen/memo_call_backend_private_cache_proof_gate" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    match selfhost_memo_call_backend_private_cache_actual_traversal_completion_boundary_stage0:
        Result::Ok accepted:
            test_assertion_exit_code assert_ne_bool "completion cases rejected" false accepted
        Result::Err _error:
            test_assertion_exit_code assert_ne_bool "completion setup succeeded" false false
```
