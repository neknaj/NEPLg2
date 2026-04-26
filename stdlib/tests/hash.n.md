# stdlib/hash.n.md

## hash_main

neplg2:test
```neplg2

#entry main
#indent 4
#target std
#import "alloc/hash/fnv1a32" as *
#import "alloc/hash/hash32" as *
#import "alloc/hash/sha256" as *
#import "core/traits/hash" as *
#import "std/test" as *
#import "alloc/collections/vec" as *
#import "alloc/string" as string
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn sha256_update_str_loop <(Sha256,str,i32,i32)*>Sha256> (ctx, text, idx, n):
    if:
        ge idx n
        then:
            ctx
        else:
            let b <i32> string::string_byte_at_unchecked text idx
            let next_ctx <Sha256> sha256_update ctx b
            sha256_update_str_loop next_ctx text add idx 1 n

fn sha256_update_str <(Sha256,str)*>Sha256> (ctx, text):
    sha256_update_str_loop ctx text 0 string::len text

fn sha256_expected_empty <(i32)->i32> (idx):
    match idx:
        0:
            227
        1:
            176
        2:
            196
        3:
            66
        4:
            152
        5:
            252
        6:
            28
        7:
            20
        8:
            154
        9:
            251
        10:
            244
        11:
            200
        12:
            153
        13:
            111
        14:
            185
        15:
            36
        16:
            39
        17:
            174
        18:
            65
        19:
            228
        20:
            100
        21:
            155
        22:
            147
        23:
            76
        24:
            164
        25:
            149
        26:
            153
        27:
            27
        28:
            120
        29:
            82
        30:
            184
        31:
            85
        _:
            #intrinsic "unreachable" <> ()

fn sha256_expected_abc <(i32)->i32> (idx):
    match idx:
        0:
            186
        1:
            120
        2:
            22
        3:
            191
        4:
            143
        5:
            1
        6:
            207
        7:
            234
        8:
            65
        9:
            65
        10:
            64
        11:
            222
        12:
            93
        13:
            174
        14:
            34
        15:
            35
        16:
            176
        17:
            3
        18:
            97
        19:
            163
        20:
            150
        21:
            23
        22:
            122
        23:
            156
        24:
            180
        25:
            16
        26:
            255
        27:
            97
        28:
            242
        29:
            0
        30:
            21
        31:
            173
        _:
            #intrinsic "unreachable" <> ()

fn sha256_expected_multi <(i32)->i32> (idx):
    match idx:
        0:
            36
        1:
            141
        2:
            106
        3:
            97
        4:
            210
        5:
            6
        6:
            56
        7:
            184
        8:
            229
        9:
            192
        10:
            38
        11:
            147
        12:
            12
        13:
            62
        14:
            96
        15:
            57
        16:
            163
        17:
            60
        18:
            228
        19:
            89
        20:
            100
        21:
            255
        22:
            33
        23:
            103
        24:
            246
        25:
            236
        26:
            237
        27:
            212
        28:
            25
        29:
            219
        30:
            6
        31:
            193
        _:
            #intrinsic "unreachable" <> ()

fn sha256_expected_byte <(i32,i32)->i32> (kind, idx):
    match kind:
        0:
            sha256_expected_empty idx
        1:
            sha256_expected_abc idx
        2:
            sha256_expected_multi idx
        _:
            #intrinsic "unreachable" <> ()

fn sha256_check_digest_loop <(&Vec<i32>,i32,i32,Vec<Result<(),str>>)*>Vec<Result<(),str>>> (digest, kind, idx, checks):
    if:
        ge idx 32
        then:
            checks
        else:
            let actual <i32> unwrap<i32> get_ref<i32> digest idx
            let next_checks <Vec<Result<(),str>>> checks_push checks check_eq_i32 sha256_expected_byte kind idx actual
            sha256_check_digest_loop digest kind add idx 1 next_checks

fn main <()*>i32> ():
    let h0 new_fnv1a32
    let h1 fnv1a32_update h0 97
    let result fnv1a32_finalize h1

    let empty0 <Sha256> new_sha256
    let empty_digest <Vec<i32>> sha256_finalize empty0

    let abc0 <Sha256> new_sha256
    let abc1 <Sha256> sha256_update_str abc0 "abc"
    let abc_digest <Vec<i32>> sha256_finalize abc1

    let multi0 <Sha256> new_sha256
    let multi1 <Sha256> sha256_update_str multi0 "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"
    let multi_digest <Vec<i32>> sha256_finalize multi1
    let empty_len <i32> len_ref<i32> &empty_digest
    let abc_len <i32> len_ref<i32> &abc_digest
    let multi_len <i32> len_ref<i32> &multi_digest

    let checks0 <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 -468965076 result
        |> checks_push check_eq_i32 hash32_by_trait 123456 hash32_by_trait 123456
        |> checks_push check ne hash32_by_trait 123456 hash32_by_trait 123457
        |> checks_push check_eq_i32 32 empty_len
        |> checks_push check_eq_i32 32 abc_len
        |> checks_push check_eq_i32 32 multi_len
    let checks1 <Vec<Result<(),str>>> sha256_check_digest_loop &empty_digest 0 0 checks0
    let checks2 <Vec<Result<(),str>>> sha256_check_digest_loop &abc_digest 1 0 checks1
    let checks3 <Vec<Result<(),str>>> sha256_check_digest_loop &multi_digest 2 0 checks2
    free<i32> empty_digest
    free<i32> abc_digest
    free<i32> multi_digest
    let shown <Vec<Result<(),str>>> checks_print_report checks3;
    checks_exit_code shown
```
