use nepl_core::loader::Loader;
use nepl_core::{compile_module, CompileOptions, CompileTarget};
use std::sync::{Arc, Mutex};
use wasmi::{Caller, Engine, Extern, Linker, Module, Store};

/// Compile source to wasm bytes.
pub fn compile_src(src: &str) -> Vec<u8> {
    let loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline("<test>".into(), src.to_string())
        .expect("load");
    let artifact = compile_module(
        loaded.module,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
        },
    )
    .expect("compile failure");
    artifact.wasm
}

/// Compile source with explicit options (uses Loader to resolve imports).
pub fn compile_src_with_options(src: &str, options: CompileOptions) -> Vec<u8> {
    let loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline("<test>".into(), src.to_string())
        .expect("load");
    let artifact = compile_module(loaded.module, options).expect("compile failure");
    artifact.wasm
}

/// Compile and run `main` returning i32 (or 0 if main is ())->()).
pub fn run_main_i32(src: &str) -> i32 {
    let wasm = compile_src(src);
    let engine = Engine::default();
    let module = Module::new(&engine, &*wasm).expect("module");
    let mut linker = Linker::new(&engine);
    // Minimal env for legacy stdio (if present)
    linker
        .func_wrap("env", "print_i32", |x: i32| {
            println!("{x}");
        })
        .unwrap();
    linker
        .func_wrap(
            "env",
            "print_str",
            |mut caller: Caller<'_, ()>, ptr: i32| {
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    let offset = ptr as usize;
                    if offset + 4 <= data.len() {
                        let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap())
                            as usize;
                        let start = offset + 4;
                        if start + len <= data.len() {
                            let s = std::str::from_utf8(&data[start..start + len])
                                .unwrap_or("<utf8-error>");
                            println!("{s}");
                        }
                    }
                }
            },
        )
        .unwrap();
    // Provide simple host allocator (nepl_alloc) for tests: uses linear memory at 0: heap_ptr, 4: free_head
    linker
        .func_wrap(
            "nepl_alloc",
            "alloc",
            |mut caller: Caller<'_, ()>, size: i32| -> i32 {
                let header = 8u32;
                let size = size as u32;
                let total = ((size + header + 7) / 8) * 8;
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    // traverse free list
                    let mut cur = if data.len() >= 8 {
                        u32::from_le_bytes(data[4..8].try_into().unwrap())
                    } else {
                        0
                    };
                    let mut prev: Option<u32> = None;
                    while cur != 0 {
                        if (cur as usize) + 8 > data.len() {
                            break;
                        }
                        let blk_sz = u32::from_le_bytes(
                            data[cur as usize..cur as usize + 4].try_into().unwrap(),
                        );
                        let next = u32::from_le_bytes(
                            data[cur as usize + 4..cur as usize + 8].try_into().unwrap(),
                        );
                        if blk_sz >= total {
                            // remove
                            if let Some(p) = prev {
                                mem.write(&mut caller, (p + 4) as usize, &next.to_le_bytes())
                                    .ok();
                            } else {
                                mem.write(&mut caller, 4usize, &next.to_le_bytes()).ok();
                            }
                            // possibly split
                            let remain = blk_sz - total;
                            if remain >= 16 {
                                let new_blk = cur + total;
                                mem.write(&mut caller, new_blk as usize, &remain.to_le_bytes())
                                    .ok();
                                mem.write(&mut caller, (new_blk + 4) as usize, &next.to_le_bytes())
                                    .ok();
                                mem.write(&mut caller, cur as usize, &total.to_le_bytes())
                                    .ok();
                            }
                            return (cur + header) as i32;
                        }
                        prev = Some(cur);
                        cur = next;
                    }
                    // bump
                    if data.len() < 4 {
                        return 0;
                    }
                    let heap = u32::from_le_bytes(data[0..4].try_into().unwrap());
                    let alloc_start = ((heap + 7) / 8) * 8;
                    let new_heap = alloc_start.saturating_add(total);
                    if new_heap as usize > data.len() {
                        return 0;
                    }
                    mem.write(&mut caller, alloc_start as usize, &total.to_le_bytes())
                        .ok();
                    mem.write(&mut caller, 0usize, &new_heap.to_le_bytes()).ok();
                    return (alloc_start + header) as i32;
                }
                0
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "nepl_alloc",
            "dealloc",
            |mut caller: Caller<'_, ()>, ptr: i32, size: i32| {
                let header = 8u32;
                let ptr = ptr as u32;
                let _size = size as u32;
                if ptr < header {
                    return;
                }
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let header_ptr = ptr - header;
                    let data = mem.data(&caller);
                    let cur_head = if data.len() >= 8 {
                        u32::from_le_bytes(data[4..8].try_into().unwrap())
                    } else {
                        0
                    };
                    let sz = ((_size + header + 7) / 8 * 8) as u32;
                    mem.write(&mut caller, header_ptr as usize, &sz.to_le_bytes())
                        .ok();
                    mem.write(
                        &mut caller,
                        (header_ptr + 4) as usize,
                        &cur_head.to_le_bytes(),
                    )
                    .ok();
                    mem.write(&mut caller, 4usize, &header_ptr.to_le_bytes())
                        .ok();
                }
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "nepl_alloc",
            "realloc",
            |mut caller: Caller<'_, ()>, ptr: i32, old_size: i32, new_size: i32| -> i32 {
                let header = 8u32;
                let ptr = ptr as u32;
                let old = old_size as u32;
                let new = new_size as u32;
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    if data.len() < 4 {
                        return 0;
                    }
                    let heap = u32::from_le_bytes(data[0..4].try_into().unwrap());
                    let total_new = ((new + header + 7) / 8) * 8;
                    let alloc_start = ((heap + 7) / 8) * 8;
                    let new_heap = alloc_start.saturating_add(total_new);
                    if new_heap as usize > data.len() {
                        return 0;
                    }
                    mem.write(&mut caller, alloc_start as usize, &total_new.to_le_bytes())
                        .ok();
                    mem.write(&mut caller, 0usize, &new_heap.to_le_bytes()).ok();
                    let new_ptr = alloc_start + header;
                    let copy_len = core::cmp::min(old, new) as usize;
                    if copy_len > 0 {
                        let snapshot = mem.data(&caller).to_vec();
                        let src = ptr as usize;
                        let dst = new_ptr as usize;
                        if src + copy_len <= snapshot.len() && dst + copy_len <= snapshot.len() {
                            mem.write(&mut caller, dst, &snapshot[src..src + copy_len])
                                .ok();
                        }
                    }
                    // simplistic dealloc: push old to free list
                    if ptr != 0 {
                        let hdr = ptr - header;
                        let sz = if (hdr as usize) + 4 <= mem.data(&caller).len() {
                            u32::from_le_bytes(
                                mem.data(&caller)[hdr as usize..hdr as usize + 4]
                                    .try_into()
                                    .unwrap(),
                            )
                        } else {
                            0
                        };
                        let cur_head = if mem.data(&caller).len() >= 8 {
                            u32::from_le_bytes(mem.data(&caller)[4..8].try_into().unwrap())
                        } else {
                            0
                        };
                        mem.write(&mut caller, (hdr + 4) as usize, &cur_head.to_le_bytes())
                            .ok();
                        mem.write(&mut caller, hdr as usize, &sz.to_le_bytes()).ok();
                        mem.write(&mut caller, 4usize, &hdr.to_le_bytes()).ok();
                    }
                    return new_ptr as i32;
                }
                0
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .and_then(|pre| pre.start(&mut store))
        .expect("instantiate");
    if let Ok(f) = instance.get_typed_func::<(), i32>(&store, "main") {
        f.call(&mut store, ()).expect("call")
    } else if let Ok(fu) = instance.get_typed_func::<(), ()>(&store, "main") {
        fu.call(&mut store, ()).expect("call");
        0
    } else {
        panic!("main not found")
    }
}

/// Compile and run `main`, capturing stdout via WASI fd_write.
pub fn run_main_capture_stdout(src: &str) -> String {
    let wasm = compile_src_with_options(
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasi),
        },
    );
    let engine = Engine::default();
    let module = Module::new(&engine, &*wasm).expect("module");
    let output = Arc::new(Mutex::new(String::new()));
    let mut linker = Linker::new(&engine);
    let output_buf = output.clone();
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "fd_read",
            move |mut caller: Caller<'_, ()>,
                  fd: i32,
                  _iovs_ptr: i32,
                  _iovs_len: i32,
                  nread_ptr: i32|
                  -> i32 {
                if fd != 0 {
                    return 8;
                }
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    if nread_ptr != 0 {
                        mem.write(&mut caller, nread_ptr as usize, &0u32.to_le_bytes())
                            .ok();
                    }
                }
                0
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            move |mut caller: Caller<'_, ()>,
                  fd: i32,
                  iovs_ptr: i32,
                  iovs_len: i32,
                  nwritten_ptr: i32|
                  -> i32 {
                if fd != 1 && fd != 2 {
                    return 8;
                }
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    let mut written = 0u32;
                    let count = if iovs_len > 0 { iovs_len as usize } else { 0 };
                    let base = if iovs_ptr > 0 { iovs_ptr as usize } else { 0 };
                    let mut out = output_buf.lock().unwrap();
                    for idx in 0..count {
                        let off = base.saturating_add(idx.saturating_mul(8));
                        if off + 8 > data.len() {
                            break;
                        }
                        let ptr =
                            u32::from_le_bytes(data[off..off + 4].try_into().unwrap()) as usize;
                        let len =
                            u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap()) as usize;
                        if ptr + len > data.len() {
                            break;
                        }
                        let bytes = &data[ptr..ptr + len];
                        match std::str::from_utf8(bytes) {
                            Ok(s) => out.push_str(s),
                            Err(_) => out.push_str("<utf8-error>"),
                        }
                        written = written.saturating_add(len as u32);
                    }
                    if nwritten_ptr != 0 {
                        mem.write(&mut caller, nwritten_ptr as usize, &written.to_le_bytes())
                            .ok();
                    }
                }
                0
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "nepl_alloc",
            "alloc",
            |mut caller: Caller<'_, ()>, size: i32| -> i32 {
                let header = 8u32;
                let size = size as u32;
                let total = ((size + header + 7) / 8) * 8;
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    let mut cur = if data.len() >= 8 {
                        u32::from_le_bytes(data[4..8].try_into().unwrap())
                    } else {
                        0
                    };
                    let mut prev: Option<u32> = None;
                    while cur != 0 {
                        if (cur as usize) + 8 > data.len() {
                            break;
                        }
                        let blk_sz = u32::from_le_bytes(
                            data[cur as usize..cur as usize + 4].try_into().unwrap(),
                        );
                        let next = u32::from_le_bytes(
                            data[cur as usize + 4..cur as usize + 8].try_into().unwrap(),
                        );
                        if blk_sz >= total {
                            if let Some(p) = prev {
                                mem.write(&mut caller, (p + 4) as usize, &next.to_le_bytes())
                                    .ok();
                            } else {
                                mem.write(&mut caller, 4usize, &next.to_le_bytes()).ok();
                            }
                            let remain = blk_sz - total;
                            if remain >= 16 {
                                let new_blk = cur + total;
                                mem.write(&mut caller, new_blk as usize, &remain.to_le_bytes())
                                    .ok();
                                mem.write(&mut caller, (new_blk + 4) as usize, &next.to_le_bytes())
                                    .ok();
                                mem.write(&mut caller, cur as usize, &total.to_le_bytes())
                                    .ok();
                            }
                            return (cur + header) as i32;
                        }
                        prev = Some(cur);
                        cur = next;
                    }
                    if data.len() < 4 {
                        return 0;
                    }
                    let heap = u32::from_le_bytes(data[0..4].try_into().unwrap());
                    let alloc_start = ((heap + 7) / 8) * 8;
                    let new_heap = alloc_start.saturating_add(total);
                    if new_heap as usize > data.len() {
                        return 0;
                    }
                    mem.write(&mut caller, alloc_start as usize, &total.to_le_bytes())
                        .ok();
                    mem.write(&mut caller, 0usize, &new_heap.to_le_bytes()).ok();
                    return (alloc_start + header) as i32;
                }
                0
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "nepl_alloc",
            "dealloc",
            |mut caller: Caller<'_, ()>, ptr: i32, size: i32| {
                let header = 8u32;
                let ptr = ptr as u32;
                let _size = size as u32;
                if ptr < header {
                    return;
                }
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let header_ptr = ptr - header;
                    let data = mem.data(&caller);
                    let cur_head = if data.len() >= 8 {
                        u32::from_le_bytes(data[4..8].try_into().unwrap())
                    } else {
                        0
                    };
                    let sz = ((_size + header + 7) / 8 * 8) as u32;
                    mem.write(&mut caller, header_ptr as usize, &sz.to_le_bytes())
                        .ok();
                    mem.write(
                        &mut caller,
                        (header_ptr + 4) as usize,
                        &cur_head.to_le_bytes(),
                    )
                    .ok();
                    mem.write(&mut caller, 4usize, &header_ptr.to_le_bytes())
                        .ok();
                }
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "nepl_alloc",
            "realloc",
            |mut caller: Caller<'_, ()>, ptr: i32, old_size: i32, new_size: i32| -> i32 {
                let header = 8u32;
                let ptr = ptr as u32;
                let old = old_size as u32;
                let new = new_size as u32;
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    if data.len() < 4 {
                        return 0;
                    }
                    let heap = u32::from_le_bytes(data[0..4].try_into().unwrap());
                    let total_new = ((new + header + 7) / 8) * 8;
                    let alloc_start = ((heap + 7) / 8) * 8;
                    let new_heap = alloc_start.saturating_add(total_new);
                    if new_heap as usize > data.len() {
                        return 0;
                    }
                    mem.write(&mut caller, alloc_start as usize, &total_new.to_le_bytes())
                        .ok();
                    mem.write(&mut caller, 0usize, &new_heap.to_le_bytes()).ok();
                    let new_ptr = alloc_start + header;
                    let copy_len = core::cmp::min(old, new) as usize;
                    if copy_len > 0 {
                        let snapshot = mem.data(&caller).to_vec();
                        let src = ptr as usize;
                        let dst = new_ptr as usize;
                        if src + copy_len <= snapshot.len() && dst + copy_len <= snapshot.len() {
                            mem.write(&mut caller, dst, &snapshot[src..src + copy_len])
                                .ok();
                        }
                    }
                    if ptr != 0 {
                        let hdr = ptr - header;
                        let sz = if (hdr as usize) + 4 <= mem.data(&caller).len() {
                            u32::from_le_bytes(
                                mem.data(&caller)[hdr as usize..hdr as usize + 4]
                                    .try_into()
                                    .unwrap(),
                            )
                        } else {
                            0
                        };
                        let cur_head = if mem.data(&caller).len() >= 8 {
                            u32::from_le_bytes(mem.data(&caller)[4..8].try_into().unwrap())
                        } else {
                            0
                        };
                        mem.write(&mut caller, (hdr + 4) as usize, &cur_head.to_le_bytes())
                            .ok();
                        mem.write(&mut caller, hdr as usize, &sz.to_le_bytes()).ok();
                        mem.write(&mut caller, 4usize, &hdr.to_le_bytes()).ok();
                    }
                    return new_ptr as i32;
                }
                0
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .and_then(|pre| pre.start(&mut store))
        .expect("instantiate");
    if let Ok(f) = instance.get_typed_func::<(), i32>(&store, "main") {
        let _ = f.call(&mut store, ()).expect("call");
    } else if let Ok(fu) = instance.get_typed_func::<(), ()>(&store, "main") {
        fu.call(&mut store, ()).expect("call");
    } else {
        panic!("main not found")
    }
    let captured = output.lock().unwrap().clone();
    captured
}

/// Compile and run `main`, capturing stdout and providing stdin bytes via WASI fd_read.
pub fn run_main_capture_stdout_with_stdin(src: &str, stdin: &[u8]) -> String {
    let wasm = compile_src_with_options(
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasi),
        },
    );
    let engine = Engine::default();
    let module = Module::new(&engine, &*wasm).expect("module");
    let output = Arc::new(Mutex::new(String::new()));
    let stdin_state = Arc::new(Mutex::new((stdin.to_vec(), 0usize)));
    let mut linker = Linker::new(&engine);
    let output_buf = output.clone();
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "fd_write",
            move |mut caller: Caller<'_, ()>,
                  fd: i32,
                  iovs_ptr: i32,
                  iovs_len: i32,
                  nwritten_ptr: i32|
                  -> i32 {
                if fd != 1 && fd != 2 {
                    return 8;
                }
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    let mut written = 0u32;
                    let count = if iovs_len > 0 { iovs_len as usize } else { 0 };
                    let base = if iovs_ptr > 0 { iovs_ptr as usize } else { 0 };
                    let mut out = output_buf.lock().unwrap();
                    for idx in 0..count {
                        let off = base.saturating_add(idx.saturating_mul(8));
                        if off + 8 > data.len() {
                            break;
                        }
                        let ptr =
                            u32::from_le_bytes(data[off..off + 4].try_into().unwrap()) as usize;
                        let len =
                            u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap()) as usize;
                        if ptr + len > data.len() {
                            break;
                        }
                        let bytes = &data[ptr..ptr + len];
                        match std::str::from_utf8(bytes) {
                            Ok(s) => out.push_str(s),
                            Err(_) => out.push_str("<utf8-error>"),
                        }
                        written = written.saturating_add(len as u32);
                    }
                    if nwritten_ptr != 0 {
                        mem.write(&mut caller, nwritten_ptr as usize, &written.to_le_bytes())
                            .ok();
                    }
                }
                0
            },
        )
        .unwrap();
    let stdin_buf = stdin_state.clone();
    linker
        .func_wrap(
            "wasi_snapshot_preview1",
            "fd_read",
            move |mut caller: Caller<'_, ()>,
                  fd: i32,
                  iovs_ptr: i32,
                  iovs_len: i32,
                  nread_ptr: i32|
                  -> i32 {
                if fd != 0 {
                    return 8;
                }
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data_snapshot = mem.data(&caller).to_vec();
                    let mut total = 0u32;
                    let count = if iovs_len > 0 { iovs_len as usize } else { 0 };
                    let base = if iovs_ptr > 0 { iovs_ptr as usize } else { 0 };
                    let mut state = stdin_buf.lock().unwrap();
                    for idx in 0..count {
                        let off = base.saturating_add(idx.saturating_mul(8));
                        if off + 8 > data_snapshot.len() {
                            break;
                        }
                        let ptr = u32::from_le_bytes(
                            data_snapshot[off..off + 4].try_into().unwrap(),
                        ) as usize;
                        let len = u32::from_le_bytes(
                            data_snapshot[off + 4..off + 8].try_into().unwrap(),
                        ) as usize;
                        if ptr + len > data_snapshot.len() {
                            break;
                        }
                        if state.1 >= state.0.len() {
                            break;
                        }
                        let avail = state.0.len() - state.1;
                        let take = if len < avail { len } else { avail };
                        if take == 0 {
                            break;
                        }
                        mem.write(&mut caller, ptr, &state.0[state.1..state.1 + take])
                            .ok();
                        state.1 += take;
                        total = total.saturating_add(take as u32);
                    }
                    if nread_ptr != 0 {
                        mem.write(&mut caller, nread_ptr as usize, &total.to_le_bytes())
                            .ok();
                    }
                }
                0
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "nepl_alloc",
            "alloc",
            |mut caller: Caller<'_, ()>, size: i32| -> i32 {
                let header = 8u32;
                let size = size as u32;
                let total = ((size + header + 7) / 8) * 8;
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    let mut cur = if data.len() >= 8 {
                        u32::from_le_bytes(data[4..8].try_into().unwrap())
                    } else {
                        0
                    };
                    let mut prev: Option<u32> = None;
                    while cur != 0 {
                        if (cur as usize) + 8 > data.len() {
                            break;
                        }
                        let blk_sz = u32::from_le_bytes(
                            data[cur as usize..cur as usize + 4].try_into().unwrap(),
                        );
                        let next = u32::from_le_bytes(
                            data[cur as usize + 4..cur as usize + 8].try_into().unwrap(),
                        );
                        if blk_sz >= total {
                            if let Some(p) = prev {
                                mem.write(&mut caller, (p + 4) as usize, &next.to_le_bytes())
                                    .ok();
                            } else {
                                mem.write(&mut caller, 4usize, &next.to_le_bytes()).ok();
                            }
                            let remain = blk_sz - total;
                            if remain >= 16 {
                                let new_blk = cur + total;
                                mem.write(&mut caller, new_blk as usize, &remain.to_le_bytes())
                                    .ok();
                                mem.write(&mut caller, (new_blk + 4) as usize, &next.to_le_bytes())
                                    .ok();
                                mem.write(&mut caller, cur as usize, &total.to_le_bytes())
                                    .ok();
                            }
                            return (cur + header) as i32;
                        }
                        prev = Some(cur);
                        cur = next;
                    }
                    if data.len() < 4 {
                        return 0;
                    }
                    let heap = u32::from_le_bytes(data[0..4].try_into().unwrap());
                    let alloc_start = ((heap + 7) / 8) * 8;
                    let new_heap = alloc_start.saturating_add(total);
                    if new_heap as usize > data.len() {
                        return 0;
                    }
                    mem.write(&mut caller, alloc_start as usize, &total.to_le_bytes())
                        .ok();
                    mem.write(&mut caller, 0usize, &new_heap.to_le_bytes()).ok();
                    return (alloc_start + header) as i32;
                }
                0
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "nepl_alloc",
            "dealloc",
            |mut caller: Caller<'_, ()>, ptr: i32, size: i32| {
                let header = 8u32;
                let ptr = ptr as u32;
                let _size = size as u32;
                if ptr < header {
                    return;
                }
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let header_ptr = ptr - header;
                    let data = mem.data(&caller);
                    let cur_head = if data.len() >= 8 {
                        u32::from_le_bytes(data[4..8].try_into().unwrap())
                    } else {
                        0
                    };
                    let sz = ((_size + header + 7) / 8 * 8) as u32;
                    mem.write(&mut caller, header_ptr as usize, &sz.to_le_bytes())
                        .ok();
                    mem.write(
                        &mut caller,
                        (header_ptr + 4) as usize,
                        &cur_head.to_le_bytes(),
                    )
                    .ok();
                    mem.write(&mut caller, 4usize, &header_ptr.to_le_bytes())
                        .ok();
                }
            },
        )
        .unwrap();
    linker
        .func_wrap(
            "nepl_alloc",
            "realloc",
            |mut caller: Caller<'_, ()>, ptr: i32, old_size: i32, new_size: i32| -> i32 {
                let header = 8u32;
                let ptr = ptr as u32;
                let old = old_size as u32;
                let new = new_size as u32;
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    if data.len() < 4 {
                        return 0;
                    }
                    let heap = u32::from_le_bytes(data[0..4].try_into().unwrap());
                    let total_new = ((new + header + 7) / 8) * 8;
                    let alloc_start = ((heap + 7) / 8) * 8;
                    let new_heap = alloc_start.saturating_add(total_new);
                    if new_heap as usize > data.len() {
                        return 0;
                    }
                    mem.write(&mut caller, alloc_start as usize, &total_new.to_le_bytes())
                        .ok();
                    mem.write(&mut caller, 0usize, &new_heap.to_le_bytes()).ok();
                    let new_ptr = alloc_start + header;
                    let copy_len = core::cmp::min(old, new) as usize;
                    if copy_len > 0 {
                        let snapshot = mem.data(&caller).to_vec();
                        let src = ptr as usize;
                        let dst = new_ptr as usize;
                        if src + copy_len <= snapshot.len() && dst + copy_len <= snapshot.len() {
                            mem.write(&mut caller, dst, &snapshot[src..src + copy_len])
                                .ok();
                        }
                    }
                    if ptr != 0 {
                        let hdr = ptr - header;
                        let sz = if (hdr as usize) + 4 <= mem.data(&caller).len() {
                            u32::from_le_bytes(
                                mem.data(&caller)[hdr as usize..hdr as usize + 4]
                                    .try_into()
                                    .unwrap(),
                            )
                        } else {
                            0
                        };
                        let cur_head = if mem.data(&caller).len() >= 8 {
                            u32::from_le_bytes(mem.data(&caller)[4..8].try_into().unwrap())
                        } else {
                            0
                        };
                        mem.write(&mut caller, hdr as usize, &sz.to_le_bytes())
                            .ok();
                        mem.write(&mut caller, (hdr + 4) as usize, &cur_head.to_le_bytes())
                            .ok();
                        mem.write(&mut caller, 4usize, &hdr.to_le_bytes()).ok();
                    }
                    return new_ptr as i32;
                }
                0
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("instantiate")
        .start(&mut store)
        .expect("start");
    if let Ok(f) = instance.get_typed_func::<(), i32>(&store, "main") {
        f.call(&mut store, ()).expect("call");
    } else if let Ok(fu) = instance.get_typed_func::<(), ()>(&store, "main") {
        fu.call(&mut store, ()).expect("call");
    } else {
        panic!("main not found");
    }
    let captured = output.lock().unwrap().clone();
    captured
}

fn stdlib_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}
