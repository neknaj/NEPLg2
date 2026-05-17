extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use wasm_encoder::ValType;

#[derive(Debug)]
pub(super) struct LocalMap {
    map: BTreeMap<String, Vec<u32>>,
    scopes: Vec<Vec<String>>,
    next_idx: u32,
    decls: Vec<ValType>,
    alloc_helper_idx: Option<u32>,
}

impl LocalMap {
    pub(super) fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            scopes: vec![Vec::new()],
            next_idx: 0,
            decls: Vec::new(),
            alloc_helper_idx: None,
        }
    }

    pub(super) fn register_param(&mut self, name: String, wasm_type: Option<ValType>) {
        let idx = if wasm_type.is_some() {
            let idx = self.next_idx;
            self.next_idx += 1;
            idx
        } else {
            0
        };
        self.bind_name(name, idx);
    }

    pub(super) fn ensure_local(&mut self, name: String, wasm_type: Option<ValType>) -> u32 {
        if let Some(idx) = self.lookup_current(&name) {
            idx
        } else {
            let idx = if let Some(wasm_type) = wasm_type {
                let idx = self.next_idx;
                self.next_idx += 1;
                self.decls.push(wasm_type);
                idx
            } else {
                // Zero-sized/unit locals do not need a wasm local slot.
                0
            };
            self.bind_name(name, idx);
            idx
        }
    }

    pub(super) fn set_alloc_helper_idx(&mut self, idx: Option<u32>) {
        self.alloc_helper_idx = idx;
    }

    pub(super) fn alloc_helper_idx(&self) -> Option<u32> {
        self.alloc_helper_idx
    }

    pub(super) fn alloc_temp(&mut self, vt: ValType) -> u32 {
        let idx = self.next_idx;
        self.next_idx += 1;
        self.decls.push(vt);
        idx
    }

    pub(super) fn lookup(&self, name: &str) -> Option<u32> {
        self.map.get(name).and_then(|stack| stack.last().copied())
    }

    pub(super) fn local_decls(&self) -> Vec<(u32, ValType)> {
        self.decls.iter().map(|v| (1u32, *v)).collect()
    }

    pub(super) fn begin_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    pub(super) fn end_scope(&mut self) {
        if let Some(names) = self.scopes.pop() {
            for name in names {
                let remove_entry = if let Some(stack) = self.map.get_mut(&name) {
                    stack.pop();
                    stack.is_empty()
                } else {
                    false
                };
                if remove_entry {
                    self.map.remove(&name);
                }
            }
        }
    }

    fn lookup_current(&self, name: &str) -> Option<u32> {
        let current = self.scopes.last()?;
        if current.iter().any(|n| n == name) {
            self.lookup(name)
        } else {
            None
        }
    }

    fn bind_name(&mut self, name: String, idx: u32) {
        self.map.entry(name.clone()).or_default().push(idx);
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(name);
        }
    }
}
