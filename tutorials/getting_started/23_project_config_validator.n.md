# Project: config validator

設定値の検証は、失敗理由を `Result` の `Err` に入れて返します。呼び出し側は最初に validation を通し、以後は検証済みの値だけを使います。

neplg2:test
ret: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/test" as *
#import "core/field" as *
#import "core/math" as *

struct ServerConfig:
    port <i32>
    workers <i32>

fn validate_config <(ServerConfig)->Result<ServerConfig,str>> (config):
    if:
        lt get config "port" 1
        then:
            Result<ServerConfig,str>::Err "port too small"
        else:
            if:
                lt get config "workers" 1
                then:
                    Result<ServerConfig,str>::Err "workers too small"
                else:
                    Result<ServerConfig,str>::Ok config

fn expect_valid <(ServerConfig)->Result<(),str>> (config):
    match validate_config config:
        Result::Ok _ok:
            Result<(),str>::Ok ()
        Result::Err msg:
            Result<(),str>::Err msg

fn expect_invalid <(ServerConfig,str)->Result<(),str>> (config, expected):
    match validate_config config:
        Result::Ok _ok:
            Result<(),str>::Err "expected invalid config"
        Result::Err msg:
            check_str_eq expected msg

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push expect_valid ServerConfig 8080 4
        |> checks_push expect_invalid ServerConfig 0 4 "port too small"
        |> checks_push expect_invalid ServerConfig 8080 0 "workers too small"
    let shown checks_print_report checks
    checks_exit_code shown
```

`ServerConfig` のような small struct は、validation を通した後に次の層へ渡すと API 境界が読みやすくなります。
