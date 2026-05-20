# NEPLg2 self-host mono

## instance_key_identity_and_seed_are_stable

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/math" as *
#import "neplg2/core/mono/mono" as *

fn main <()->i32> ():
    let def0 <SelfhostMonoDefId> selfhost_mono_def_id_new 4 12
    let def1 <SelfhostMonoDefId> selfhost_mono_def_id_new 4 13
    let args0 <SelfhostMonoTypeArgRange> selfhost_mono_type_arg_range_new 20 2
    let key0 <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new def0 args0
    let key1 <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new (selfhost_mono_def_id_new 4 12) (selfhost_mono_type_arg_range_new 20 2)
    let key2 <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new def1 args0
    let seed0 <i32> selfhost_mono_instance_key_seed key0
    let seed1 <i32> selfhost_mono_instance_key_seed key1
    let seed2 <i32> selfhost_mono_instance_key_seed key2
    let ok <bool>:
        and:
            and selfhost_mono_instance_key_eq key0 key1 eq seed0 seed1
            and not selfhost_mono_instance_key_eq key0 key2 ne seed0 seed2
    if ok 0 1
```

## invalid_ids_and_ranges_are_rejected

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/math" as *
#import "core/option" as *
#import "neplg2/core/mono/mono" as *

fn main <()->i32> ():
    let valid_def <SelfhostMonoDefId> selfhost_mono_def_id_new 0 0
    let invalid_def <SelfhostMonoDefId> selfhost_mono_def_id_new -1 0
    let valid_range <SelfhostMonoTypeArgRange> selfhost_mono_type_arg_range_empty
    let invalid_range <SelfhostMonoTypeArgRange> selfhost_mono_type_arg_range_new 0 -1
    let valid_key <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new valid_def valid_range
    let invalid_key0 <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new invalid_def valid_range
    let invalid_key1 <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new valid_def invalid_range
    let valid_instance <SelfhostMonoInstanceId> selfhost_mono_instance_id_new 0
    let pending <Option<SelfhostMonoInstanceId>> selfhost_mono_instance_id_pending
    let assigned <Option<SelfhostMonoInstanceId>> selfhost_mono_instance_id_assigned valid_instance
    let assigned_ok <bool>:
        match assigned:
            Option::Some id:
                eq 0 selfhost_mono_instance_id_index id
            Option::None:
                false
    let ok <bool>:
        and:
            and selfhost_mono_instance_key_is_valid valid_key assigned_ok
            and:
                and not selfhost_mono_instance_key_is_valid invalid_key0 not selfhost_mono_instance_key_is_valid invalid_key1
                is_none<SelfhostMonoInstanceId> pending
    if ok 0 1
```

## instance_record_keeps_key_and_assigned_id_together

neplg2:test
ret: 0
```neplg2
#entry main
#target core
#indent 4

#import "core/math" as *
#import "neplg2/core/mono/mono" as *

fn main <()->i32> ():
    let def <SelfhostMonoDefId> selfhost_mono_def_id_new 1 2
    let args <SelfhostMonoTypeArgRange> selfhost_mono_type_arg_range_new 3 4
    let key <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new def args
    let same_key <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new (selfhost_mono_def_id_new 1 2) (selfhost_mono_type_arg_range_new 3 4)
    let other_key <SelfhostMonoInstanceKey> selfhost_mono_instance_key_new (selfhost_mono_def_id_new 1 3) args
    let record <SelfhostMonoInstanceRecord> selfhost_mono_instance_record_new key selfhost_mono_instance_id_new 9
    let record_key <SelfhostMonoInstanceKey> selfhost_mono_instance_record_key record
    let record_id <SelfhostMonoInstanceId> selfhost_mono_instance_record_id record
    let ok <bool>:
        and:
            and selfhost_mono_instance_key_eq key record_key selfhost_mono_instance_record_matches_key record same_key
            and not selfhost_mono_instance_record_matches_key record other_key eq 9 selfhost_mono_instance_id_index record_id
    if ok 0 1
```
