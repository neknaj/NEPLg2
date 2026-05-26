# kp 補助ライブラリと streamio の組み合わせテスト

## stream_scanner_to_stdio_stdout_i32

neplg2:test[normalize_newlines]
stdin: "10 20 30\n"
stdout: "10\n20\n30\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "std/stdio" as *
#import "core/result" as *

fn main %impure fn unit unit \unit:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let a %i32 read &sc;
    let b %i32 read &sc;
    let c %i32 read &sc;
    println_i32 a;
    println_i32 b;
    println_i32 c;
    close sc;
```

## stdio_stdin_to_stream_writer_stdout

neplg2:test[normalize_newlines]
stdin: "hello world\n"
stdout: "hello world"
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *
#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main %impure fn unit unit \unit:
    let line %str read_line;
    unwrap_ok open WriteStream::Stdio
    |> write line
    |> flush
    |> close;
```

## stream_scanner_to_stream_writer_i32

neplg2:test[normalize_newlines]
stdin: "5 3\n1 2 3 4 5\n1 3\n2 5\n1 5\n"
stdout: "6\n14\n15\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "core/result" as *
#import "core/option" as *
#import "alloc/collections/vec" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main %impure fn unit unit \unit:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let n %i32 read &sc;
    let q %i32 read &sc;

    let pref_len %i32 add n 1;
    let pref %Vec i32 unwrap_ok filled pref_len 0;

    let mut i %i32 1;
    while le i n:
        do:
            let a %i32 read &sc;
            let im1 %i32 sub i 1;
            let prev %i32 if and ge im1 0 lt im1 pref_len:
                then:
                    match get &pref im1:
                        Option::Some v:
                            v
                        Option::None:
                            #intrinsic "unreachable" <> ()
                else:
                    #intrinsic "unreachable" <> ()
            let cur %i32 add prev a;
            if and ge i 0 lt i pref_len:
                then:
                    replace &pref i cur
                else:
                    #intrinsic "unreachable" <> ()
            set i add i 1;

    let mut w %StreamWriter unwrap_ok open WriteStream::Stdio;
    let mut k %i32 0;
    while lt k q:
        do:
            let l1 %i32 read &sc;
            let r1 %i32 read &sc;
            let l %i32 sub l1 1;
            let diff %i32 if and and ge l 0 lt l pref_len and ge r1 0 lt r1 pref_len:
                then:
                    match get &pref l:
                        Option::Some left:
                            match get &pref r1:
                                Option::Some right:
                                    sub right left
                                Option::None:
                                    0
                        Option::None:
                            0
                else:
                    0
            set w writeln w diff;
            set k add k 1;

    set w flush w;
    close w;
    close sc;
    free pref;
```

## stream_scanner_to_stream_writer_i64

neplg2:test[normalize_newlines]
stdin: "6\n"
stdout: "13\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/math" as *
#import "core/result" as *
#import "core/cast" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn ways %impure fn i32 i64 \n:
    if le n 1:
        then %i64 cast 1
        else:
            let mut a %i64 cast 1;
            let mut b %i64 cast 1;
            let mut i %i32 2;
            while le i n:
                do:
                    let c %i64 add a b;
                    set a b;
                    set b c;
                    set i add i 1;
            b

fn main %impure fn unit unit \unit:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let n %i32 read &sc;
    let ans %i64 ways n;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln ans
    |> flush
    |> close;
```

## stream_scanner_to_stream_writer_f64

neplg2:test[normalize_newlines]
stdin: "3.5 -2.25 1e2\n"
stdout: "3.500000\n-2.250000\n100.000000\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main %impure fn unit unit \unit:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let a %f64 read &sc;
    let b %f64 read &sc;
    let c %f64 read &sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln a
    |> writeln b
    |> writeln c
    |> flush
    |> close;
```

## stream_scanner_to_stream_writer_f32

neplg2:test[normalize_newlines]
stdin: "1.25\n"
stdout: "1.250000\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main %impure fn unit unit \unit:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let v %f32 read &sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln v
    |> flush
    |> close;
```

## kpsearch_unique_and_count

neplg2:test[normalize_newlines]
stdout: "2 3\n1 2 5\n"
```neplg2
#entry main
#indent 4
#target std

#import "kp/kpsearch" as *
#import "core/result" as *
#import "core/math" as *
#import "core/option" as *
#import "std/stdio" as *
#import "alloc/collections/vec" as *

fn main %impure fn unit unit \unit:
    let len %i32 6;
    let count_data %Vec i32:
        unwrap_ok with_capacity len
        |> push 1 |> unwrap_ok
        |> push 1 |> unwrap_ok
        |> push 2 |> unwrap_ok
        |> push 2 |> unwrap_ok
        |> push 5 |> unwrap_ok
        |> push 5 |> unwrap_ok
    let cnt2 %i32 count_equal_range_vec_i32 &count_data 2;
    let unique_data %Vec i32:
        unwrap_ok with_capacity len
        |> push 1 |> unwrap_ok
        |> push 1 |> unwrap_ok
        |> push 2 |> unwrap_ok
        |> push 2 |> unwrap_ok
        |> push 5 |> unwrap_ok
        |> push 5 |> unwrap_ok
    let unique %UniqueSortedVecI32 unique_sorted_vec_i32 unique_data;
    let new_len %i32 unique_sorted_vec_i32_len &unique;
    print_i32 cnt2;
    print " ";
    println_i32 new_len;
    free count_data;

    let mut i %i32 0;
    while lt i new_len:
        do:
            if gt i 0:
                then print " "
                else unit
            match unique_sorted_vec_i32_get &unique i:
                Option::Some value:
                    print_i32 value
                Option::None:
                    unit
            set i add i 1;
    println "";
    free unique;
```
