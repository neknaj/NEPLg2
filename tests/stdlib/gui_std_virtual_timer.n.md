# GUI std virtual timer doctests

このファイルは、headless / offscreen test 用 deterministic timer scheduler の public import surface を固定する。

`GuiVirtualTimerState` は public struct であるため、schedule / advance は public constructor で作られた不正 state を必ず再検査する。repeating timer の catch-up は queue ではなく state の remainder と `advance state 0` により 1 event ずつ取り出す。

source policy labels:

- gui_std_virtual_timer_facade_ok
- gui_std_virtual_timer_state_invariant_ok
- gui_std_virtual_timer_one_shot_clear_before_event_ok
- gui_std_virtual_timer_repeating_remainder_drain_ok
- gui_std_virtual_timer_no_sentinel_no_queue_no_platform_no_fallback

## virtual timer state transition

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"gui_std_virtual_timer\" count=15 failed=0\nassertion index=0 status=ok kind=bool label=\"before interval none\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"one-shot id\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=2 status=ok kind=eq_i32 label=\"one-shot tick\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=3 status=ok kind=bool label=\"one-shot clears state\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=4 status=ok kind=eq_i32 label=\"repeating first tick\" expected=\"1\" actual=\"1\" message=\"\"\nassertion index=5 status=ok kind=eq_i32 label=\"repeating first remainder\" expected=\"15\" actual=\"15\" message=\"\"\nassertion index=6 status=ok kind=eq_i32 label=\"repeating drain tick\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=7 status=ok kind=eq_i32 label=\"repeating drain remainder\" expected=\"5\" actual=\"5\" message=\"\"\nassertion index=8 status=ok kind=bool label=\"clear removes timer\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=9 status=ok kind=bool label=\"invalid request rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=10 status=ok kind=bool label=\"negative delta rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=11 status=ok kind=bool label=\"malformed none state rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=12 status=ok kind=bool label=\"malformed active state rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=13 status=ok kind=bool label=\"elapsed overflow rejected\" expected=\"true\" actual=\"true\" message=\"\"\nassertion index=14 status=ok kind=bool label=\"tick overflow rejected\" expected=\"true\" actual=\"true\" message=\"\"\n"
exit_code: 0
```neplg2
#entry main
#target std
#indent 4

#import "core/option" as *
#import "core/result" as *
#import "core/gui" as *
#import "std/gui/timer" as *
#import "std/gui/virtual_timer" as *
#import "std/gui/window" as *
#import "std/test" as *

// gui_std_virtual_timer_facade_ok
// gui_std_virtual_timer_state_invariant_ok
// gui_std_virtual_timer_one_shot_clear_before_event_ok
// gui_std_virtual_timer_repeating_remainder_drain_ok
// gui_std_virtual_timer_no_sentinel_no_queue_no_platform_no_fallback

fn state_error_is_invalid %fn Result GuiVirtualTimerState GuiError bool \result:
    match result:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _state:
            false

fn advance_error_is_invalid %fn Result GuiVirtualTimerAdvance GuiError bool \result:
    match result:
        Result::Err error:
            match error:
                GuiError::InvalidCommand:
                    true
                _:
                    false
        Result::Ok _advance:
            false

fn option_timer_id %fn Option GuiEvent i32 \event_option:
    match event_option:
        Option::Some event:
            match event:
                GuiEvent::Timer timer:
                    timer_event_timer_id &timer
                _:
                    -1
        Option::None:
            -2

fn option_timer_tick %fn Option GuiEvent i32 \event_option:
    match event_option:
        Option::Some event:
            match event:
                GuiEvent::Timer timer:
                    timer_event_tick &timer
                _:
                    -1
        Option::None:
            -2

fn main %impure fn void i32 \void:
    let window %WindowId unwrap_ok window_id_result 7
    let one_timer %TimerId timer_id 5
    let repeating_timer %TimerId timer_id 6
    let invalid_timer %TimerId timer_id 0
    let one_request %TimerRequest timer_request window one_timer 10 false
    let repeating_request %TimerRequest timer_request window repeating_timer 10 true
    let invalid_request %TimerRequest timer_request window invalid_timer 10 true
    let clear_request %TimerRequest timer_request window one_timer 0 false
    let empty %GuiVirtualTimerState gui_virtual_timer_empty
    let one_state %GuiVirtualTimerState unwrap_ok gui_virtual_timer_schedule empty one_request
    let before %GuiVirtualTimerAdvance unwrap_ok gui_virtual_timer_advance one_state 9
    let before_event %Option GuiEvent gui_virtual_timer_advance_event &before
    let before_state %GuiVirtualTimerState gui_virtual_timer_advance_state &before
    let fired %GuiVirtualTimerAdvance unwrap_ok gui_virtual_timer_advance before_state 1
    let fired_event %Option GuiEvent gui_virtual_timer_advance_event &fired
    let after_one %GuiVirtualTimerState gui_virtual_timer_advance_state &fired
    let repeating_state %GuiVirtualTimerState unwrap_ok gui_virtual_timer_schedule empty repeating_request
    let repeating_first %GuiVirtualTimerAdvance unwrap_ok gui_virtual_timer_advance repeating_state 25
    let repeating_first_event %Option GuiEvent gui_virtual_timer_advance_event &repeating_first
    let repeating_after_first %GuiVirtualTimerState gui_virtual_timer_advance_state &repeating_first
    let repeating_drain %GuiVirtualTimerAdvance unwrap_ok gui_virtual_timer_advance repeating_after_first 0
    let repeating_drain_event %Option GuiEvent gui_virtual_timer_advance_event &repeating_drain
    let repeating_after_drain %GuiVirtualTimerState gui_virtual_timer_advance_state &repeating_drain
    let cleared %GuiVirtualTimerState unwrap_ok gui_virtual_timer_schedule repeating_after_drain clear_request
    let clear_advance %GuiVirtualTimerAdvance unwrap_ok gui_virtual_timer_advance cleared 100
    let clear_event %Option GuiEvent gui_virtual_timer_advance_event &clear_advance
    let invalid_request_result %Result GuiVirtualTimerState GuiError gui_virtual_timer_schedule empty invalid_request
    let negative_delta_result %Result GuiVirtualTimerAdvance GuiError gui_virtual_timer_advance repeating_state -1
    let malformed_none %GuiVirtualTimerState GuiVirtualTimerState none 1 0
    let malformed_none_result %Result GuiVirtualTimerAdvance GuiError gui_virtual_timer_advance malformed_none 0
    let malformed_active %GuiVirtualTimerState GuiVirtualTimerState some clear_request 0 0
    let malformed_active_result %Result GuiVirtualTimerAdvance GuiError gui_virtual_timer_advance malformed_active 0
    let overflow_state %GuiVirtualTimerState GuiVirtualTimerState some repeating_request 2147483647 0
    let overflow_result %Result GuiVirtualTimerAdvance GuiError gui_virtual_timer_advance overflow_state 1
    let tick_max_state %GuiVirtualTimerState GuiVirtualTimerState some repeating_request 10 2147483647
    let tick_overflow_result %Result GuiVirtualTimerAdvance GuiError gui_virtual_timer_advance tick_max_state 0
    let before_check assert "before interval none" is_none before_event
    let one_id_check assert_eq_i32 "one-shot id" 5 option_timer_id fired_event
    let one_tick_check assert_eq_i32 "one-shot tick" 1 option_timer_tick fired_event
    let one_clear_check assert "one-shot clears state" is_none gui_virtual_timer_state_request &after_one
    let repeating_tick_check assert_eq_i32 "repeating first tick" 1 option_timer_tick repeating_first_event
    let repeating_remainder_check assert_eq_i32 "repeating first remainder" 15 gui_virtual_timer_state_elapsed_ms &repeating_after_first
    let repeating_drain_tick_check assert_eq_i32 "repeating drain tick" 2 option_timer_tick repeating_drain_event
    let repeating_drain_remainder_check assert_eq_i32 "repeating drain remainder" 5 gui_virtual_timer_state_elapsed_ms &repeating_after_drain
    let clear_check assert "clear removes timer" is_none clear_event
    let invalid_request_check assert "invalid request rejected" state_error_is_invalid invalid_request_result
    let negative_delta_check assert "negative delta rejected" advance_error_is_invalid negative_delta_result
    let malformed_none_check assert "malformed none state rejected" advance_error_is_invalid malformed_none_result
    let malformed_active_check assert "malformed active state rejected" advance_error_is_invalid malformed_active_result
    let overflow_check assert "elapsed overflow rejected" advance_error_is_invalid overflow_result
    let tick_overflow_check assert "tick overflow rejected" advance_error_is_invalid tick_overflow_result
    let checks:
        test_report_new "gui_std_virtual_timer"
        |> test_report_push before_check
        |> test_report_push one_id_check
        |> test_report_push one_tick_check
        |> test_report_push one_clear_check
        |> test_report_push repeating_tick_check
        |> test_report_push repeating_remainder_check
        |> test_report_push repeating_drain_tick_check
        |> test_report_push repeating_drain_remainder_check
        |> test_report_push clear_check
        |> test_report_push invalid_request_check
        |> test_report_push negative_delta_check
        |> test_report_push malformed_none_check
        |> test_report_push malformed_active_check
        |> test_report_push overflow_check
        |> test_report_push tick_overflow_check
    let shown test_report_print_stdout checks
    test_report_exit_code shown
```
