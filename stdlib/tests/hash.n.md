# stdlib/hash.n.md

## hash_main

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"hash_main\" count=9 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"fnv1a32 finalize\" expected=\"-468965076\" actual=\"-468965076\" message=\"\"\nassertion index=1 status=ok kind=bool label=\"hash32 trait stable\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=2 status=ok kind=bool label=\"hash32 trait differentiates\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=3 status=ok kind=eq_i32 label=\"sha256 empty length\" expected=\"32\" actual=\"32\" message=\"\"\nassertion index=4 status=ok kind=bool label=\"sha256 empty bytes\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"sha256 abc length\" expected=\"32\" actual=\"32\" message=\"\"\nassertion index=6 status=ok kind=bool label=\"sha256 abc bytes\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"sha256 multi length\" expected=\"32\" actual=\"32\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"sha256 multi bytes\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "alloc/hash/fnv1a32" as *
#import "alloc/hash/hash32" as *
#import "alloc/hash/sha256" as *
#import "core/traits/hash" as *
#import "std/test" as *
#import "alloc/string" as text
#import "alloc/collections/vec" as *
#import "alloc/string/access" as string
#import "alloc/string/byte_index" as string_byte_index
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

fn sha256_update_str_loop %impure fn Sha256 impure fn str impure fn i32 impure fn i32 Result Sha256 StdErrorKind \ctx\text\idx\n:
    let mut current %Sha256 ctx
    let mut cursor %i32 idx
    let mut failed %bool false
    let mut failure %StdErrorKind StdErrorKind::OutOfMemory
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
            Result::Err failure
        else:
            Result::Ok current

fn sha256_update_str %impure fn Sha256 impure fn str Result Sha256 StdErrorKind \ctx\text:
    sha256_update_str_loop ctx text 0 string::len text

fn sha256_digest_for_text %impure fn str Result Vec i32 StdErrorKind \text:
    match new_sha256:
        Result::Err e:
            Result::Err e
        Result::Ok ctx0:
            match sha256_update_str ctx0 text:
                Result::Err e:
                    Result::Err e
                Result::Ok ctx1:
                    sha256_finalize ctx1

fn sha256_expected_empty %fn i32 i32 \idx:
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

fn sha256_expected_abc %fn i32 i32 \idx:
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

fn sha256_expected_multi %fn i32 i32 \idx:
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

fn sha256_expected_byte %fn i32 fn i32 i32 \kind\idx:
    match kind:
        0:
            sha256_expected_empty idx
        1:
            sha256_expected_abc idx
        2:
            sha256_expected_multi idx
        _:
            #intrinsic "unreachable" <> ()

fn sha256_digest_matches_loop %fn &Vec i32 fn i32 fn i32 bool \digest\kind\idx:
    if:
        ge idx 32
        then:
            true
        else:
            match get digest idx:
                Option::None:
                    false
                Option::Some actual:
                    and eq sha256_expected_byte kind idx actual sha256_digest_matches_loop digest kind add idx 1

fn sha256_digest_matches %fn &Vec i32 fn i32 bool \digest\kind:
    sha256_digest_matches_loop digest kind 0

fn sha256_push_digest_checks %impure fn TestReport impure fn str impure fn Result Vec i32 StdErrorKind impure fn i32 TestReport \report\label\digest_result\kind:
    match digest_result:
        Result::Err e:
            test_report_push report test_assertion_fail label std_error_kind_str e
        Result::Ok digest:
            let digest_len %i32 len &digest
            let len_label %str text::concat label " length"
            let bytes_label %str text::concat label " bytes"
            let report1 test_report_push report assert_eq_i32 len_label 32 digest_len
            let report2 test_report_push report1 assert bytes_label sha256_digest_matches &digest kind
            free digest
            report2

fn main %impure fn void i32 \void:
    let h0 new_fnv1a32
    let h1 fnv1a32_update h0 97
    let result fnv1a32_finalize h1
    let hash_same_a %i32 hash32_by_trait 123456
    let hash_same_b %i32 hash32_by_trait 123456
    let hash_other %i32 hash32_by_trait 123457

    let empty_digest %Result Vec i32 StdErrorKind sha256_digest_for_text ""
    let abc_digest %Result Vec i32 StdErrorKind sha256_digest_for_text "abc"
    let multi_digest %Result Vec i32 StdErrorKind sha256_digest_for_text "abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"

    let report0:
        test_report_new "hash_main"
        |> test_report_push assert_eq_i32 "fnv1a32 finalize" -468965076 result
        |> test_report_push assert "hash32 trait stable" eq hash_same_a hash_same_b
        |> test_report_push assert "hash32 trait differentiates" ne hash_same_a hash_other
    let report1 sha256_push_digest_checks report0 "sha256 empty" empty_digest 0
    let report2 sha256_push_digest_checks report1 "sha256 abc" abc_digest 1
    let report3 sha256_push_digest_checks report2 "sha256 multi" multi_digest 2
    let shown test_report_print_stdout report3
    test_report_exit_code shown
```
