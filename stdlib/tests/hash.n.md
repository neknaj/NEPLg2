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
#import "alloc/string/access" as string
#import "alloc/string/byte_index" as string_byte_index
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn sha256_update_str_loop <(Sha256,str,i32,i32)*>Result<Sha256, StdErrorKind>> (ctx, text, idx, n):
    let mut current <Sha256> ctx
    let mut cursor <i32> idx
    let mut failed <bool> false
    let mut failure <StdErrorKind> StdErrorKind::OutOfMemory
    while and lt cursor n not failed:
        do:
            match string_byte_index::checked_string_byte_at text cursor:
                Option::Some b:
                    match sha256_update current b:
                        Result::Err e:
                            set failure sha256_update_error_kind &e
                            set current sha256_update_error_ctx e
                            set failed true
                        Result::Ok next_ctx:
                            set current next_ctx
                            set cursor add cursor 1
                Option::None:
                    set cursor n
    if:
        failed
        then:
            sha256_free current
            Result<Sha256, StdErrorKind>::Err failure
        else:
            Result<Sha256, StdErrorKind>::Ok current

fn sha256_update_str <(Sha256,str)*>Result<Sha256, StdErrorKind>> (ctx, text):
    sha256_update_str_loop ctx text 0 string::len text

fn sha256_digest_for_text <(str)*>Result<Vec<i32>, StdErrorKind>> (text):
    match new_sha256:
        Result::Err e:
            Result<Vec<i32>, StdErrorKind>::Err e
        Result::Ok ctx0:
            match sha256_update_str ctx0 text:
                Result::Err e:
                    Result<Vec<i32>, StdErrorKind>::Err e
                Result::Ok ctx1:
                    sha256_finalize ctx1

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

fn sha256_check_digest_loop <(&Vec<i32>,i32,i32,TestReport)*>TestReport> (digest, kind, idx, checks):
    if:
        ge idx 32
        then:
            checks
        else:
            match get<i32> digest idx:
                Option::None:
                    let next_checks checks_push checks Result<(),str>::Err "sha256 digest missing byte"
                    sha256_check_digest_loop digest kind add idx 1 next_checks
                Option::Some actual:
                    let next_checks checks_push checks check_eq_i32 sha256_expected_byte kind idx actual
                    sha256_check_digest_loop digest kind add idx 1 next_checks

fn sha256_check_result <(Result<Vec<i32>, StdErrorKind>,i32,TestReport)*>TestReport> (digest_result, kind, checks):
    match digest_result:
        Result::Err _e:
            checks_push checks Result<(),str>::Err "sha256 digest returned error"
        Result::Ok digest:
            let digest_len <i32> len<i32> &digest
            let checks1 checks_push checks check_eq_i32 32 digest_len
            let checks2 sha256_check_digest_loop &digest kind 0 checks1
            free<i32> digest
            checks2

fn main <()*>i32> ():
    let h0 new_fnv1a32
    let h1 fnv1a32_update h0 97
    let result fnv1a32_finalize h1

    let empty_digest <Result<Vec<i32>, StdErrorKind>> sha256_digest_for_text ""
    let abc_digest <Result<Vec<i32>, StdErrorKind>> sha256_digest_for_text "abc"
    let multi_digest <Result<Vec<i32>, StdErrorKind>> sha256_digest_for_text "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"

    let checks0:
        checks_new
        |> checks_push check_eq_i32 -468965076 result
        |> checks_push check_eq_i32 hash32_by_trait 123456 hash32_by_trait 123456
        |> checks_push check ne hash32_by_trait 123456 hash32_by_trait 123457
    let checks1 sha256_check_result empty_digest 0 checks0
    let checks2 sha256_check_result abc_digest 1 checks1
    let checks3 sha256_check_result multi_digest 2 checks2
    let shown checks_print_report checks3;
    checks_exit_code shown
```
