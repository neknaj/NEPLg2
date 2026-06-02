# platforms/gui/web input boundary

Web Playground backend の input host import を、raw sentinel ではなく `Result` / `Option` で扱えることを確認する。

## empty event queue returns Ok None

neplg2:test
ret: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/option" as *
#import "core/result" as *
#import "core/test" as *
#import "platforms/gui/web" as *

fn main %impure fn unit i32 \unit:
    match gui_web_poll_event_result:
        Result::Ok event:
            assert is_none event
            0
        Result::Err _:
            1
```
