# stdlib/rand.n.md

## rand_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"rand_main\" count=4 failed=0\nassertion index=0 status=ok kind=bool label=\"first generated state is nonzero\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"second generated state is nonzero\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"successive states differ\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"zero seed escapes zero state\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry test_rand
#indent 4
#target std
#import "core/rand/xorshift32" as *
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *
#import "core/field" as *

fn test_rand %impure fn unit i32 \unit:
    let rng0 new_xorshift32 42

    let rng1 xorshift32_next rng0
    let v1 get rng1 "state"

    let rng2 xorshift32_next rng1
    let v2 get rng2 "state"

    let rng_z new_xorshift32 0
    let rng_z1 xorshift32_next rng_z
    let vz1 get rng_z1 "state"

    let report:
        test_report_new "rand_main"
        |> test_report_push assert "first generated state is nonzero" not eq v1 0
        |> test_report_push assert "second generated state is nonzero" not eq v2 0
        |> test_report_push assert "successive states differ" not eq v1 v2
        |> test_report_push assert "zero seed escapes zero state" not eq vz1 0
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
