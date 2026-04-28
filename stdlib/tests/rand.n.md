# stdlib/rand.n.md

## rand_main

neplg2:test
```neplg2

#entry test_rand
#indent 4
#target std
#import "core/rand/xorshift32" as *
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *
#import "core/field" as *

fn test_rand <()*>i32> ():
    let mut checks checks_new

    let rng0 new_xorshift32 42

    let rng1 xorshift32_next rng0
    let v1 get rng1 "state"

    let rng2 xorshift32_next rng1
    let v2 get rng2 "state"

    let rng3 xorshift32_next rng2
    let v3 get rng3 "state"

    set checks checks_push checks check_ne eq v1 0 true
    set checks checks_push checks check_ne eq v2 0 true
    set checks checks_push checks check_ne eq v1 v2 true

    let rng_z new_xorshift32 0
    let rng_z1 xorshift32_next rng_z
    let vz1 get rng_z1 "state"
    set checks checks_push checks check_ne eq vz1 0 true

    let shown checks_print_report checks
    checks_exit_code shown
```
