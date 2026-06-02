# stdlib/json.n.md

## json_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
    ##: [7] ok
    ##: [8] ok
    ##: [9] ok
    ##: [10] ok
    ##: [11] ok
    ##: [12] ok
```neplg2

#entry main
#indent 4
#target std

#import "alloc/encoding/json" as *
#import "core/option" as *
#import "core/result" as *
#import "alloc/string" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    let mut checks checks_new;

    let jn1 %JsonValue json_null
    set checks checks_push checks check json_is_null jn1
    let jn2 %JsonValue json_null
    let jn2_bool %Option bool json_as_bool jn2
    set checks checks_push checks check is_none jn2_bool
    let jn3 %JsonValue json_null
    let jn3_number %Option i32 json_as_number jn3
    set checks checks_push checks check is_none jn3_number

    let jt1 %JsonValue json_bool true
    match json_as_bool jt1:
        Option::Some v:
            set checks checks_push checks check v
        Option::None:
            set checks checks_push checks Result::Err "json_as_bool true returned None"

    let jf1 %JsonValue json_bool false
    match json_as_bool jf1:
        Option::Some v:
            set checks checks_push checks check_ne true v
        Option::None:
            set checks checks_push checks Result::Err "json_as_bool false returned None"
    let jt2 %JsonValue json_bool true
    let jt2_number %Option i32 json_as_number jt2
    set checks checks_push checks check is_none jt2_number

    let jnum1 %JsonValue json_number 123
    match json_as_number jnum1:
        Option::Some v:
            set checks checks_push checks check_eq_i32 123 v
        Option::None:
            set checks checks_push checks Result::Err "json_as_number returned None"
    let jnum2 %JsonValue json_number 123
    let jnum2_bool %Option bool json_as_bool jnum2
    set checks checks_push checks check is_none jnum2_bool

    let s %str "hello"
    let js1 %JsonValue json_string s
    match json_as_string js1:
        Option::Some p:
            set checks checks_push checks check_str_eq "hello" p
        Option::None:
            set checks checks_push checks Result::Err "json_as_string returned None"
    let js2 %JsonValue json_string s
    let js2_number %Option i32 json_as_number js2
    set checks checks_push checks check is_none js2_number

    let arr1 %JsonArray unwrap_ok json_array_new
    let ja1 %JsonValue json_array arr1
    set checks checks_push checks check_ne true json_is_null ja1
    let arr2 %JsonArray unwrap_ok json_array_new
    let ja2 %JsonValue json_array arr2
    set checks checks_push checks check is_none json_as_string ja2

    let obj1 %JsonObject unwrap_ok json_object_new
    let jo1 %JsonValue json_object obj1
    set checks checks_push checks check is_none json_as_string jo1

    let shown checks_print_report checks
    checks_exit_code shown
```
