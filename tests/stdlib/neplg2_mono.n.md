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
    let invalid_instance <SelfhostMonoInstanceId> selfhost_mono_instance_id_invalid
    let ok <bool>:
        and:
            and selfhost_mono_instance_key_is_valid valid_key selfhost_mono_instance_id_is_valid valid_instance
            and:
                and not selfhost_mono_instance_key_is_valid invalid_key0 not selfhost_mono_instance_key_is_valid invalid_key1
                not selfhost_mono_instance_id_is_valid invalid_instance
    if ok 0 1
```
