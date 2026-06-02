# streamio 64-bit 入出力テスト

## stream_scanner_stream_writer_i64_roundtrip

neplg2:test[normalize_newlines]
stdin: "-9223372036854775808 0 9223372036854775807 18446744073709551615\n"
stdout: "-9223372036854775808\n0\n9223372036854775807\n18446744073709551615\n"
```neplg2
#entry main
#indent 4
#target std

#import "core/result" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main %impure fn void unit \void:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let a %i64 read &sc;
    let b %i64 read &sc;
    let c %i64 read &sc;
    let d %u64 read &sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln a
    |> writeln b
    |> writeln c
    |> writeln d
    |> flush
    |> close;
```

## stream_scanner_i64_sign_and_plus

neplg2:test[normalize_newlines]
stdin: "+42 -17 +0\n"
stdout: "42\n-17\n0\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main %impure fn void unit \void:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let a %i64 read &sc;
    let b %i64 read &sc;
    let c %i64 read &sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln a
    |> writeln b
    |> writeln c
    |> flush
    |> close;
```

## stream_scanner_stream_writer_i64_near_bounds

neplg2:test[normalize_newlines]
stdin: "-9223372036854775807 9223372036854775806 1000000000000000000 -1000000000000000000\n"
stdout: "-9223372036854775807\n9223372036854775806\n1000000000000000000\n-1000000000000000000\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main %impure fn void unit \void:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let a %i64 read &sc;
    let b %i64 read &sc;
    let c %i64 read &sc;
    let d %i64 read &sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln a
    |> writeln b
    |> writeln c
    |> writeln d
    |> flush
    |> close;
```

## stream_scanner_stream_writer_u32_roundtrip

neplg2:test[normalize_newlines]
stdin: "0 42 4294967295\n"
stdout: "0\n42\n4294967295\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/streamio" as *
#import "std/iotarget" as *
#import "core/result" as *

fn main %impure fn void unit \void:
    let sc %StreamScanner unwrap_ok open ReadStream::Stdio;
    let a %u32 read &sc;
    let b %u32 read &sc;
    let c %u32 read &sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln a
    |> writeln b
    |> writeln c
    |> flush
    |> close;
```
