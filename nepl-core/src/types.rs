extern crate alloc;
#[cfg(not(target_os = "none"))]
extern crate std;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::ast::Effect;
use crate::backend_scalar_type::BackendScalarType;
use crate::source_map::CompilerMemoryType;

macro_rules! type_log {
    ($($arg:tt)*) => {{
        #[cfg(target_os = "none")]
        {
            let _ = core::format_args!($($arg)*);
        }
        #[cfg(not(target_os = "none"))]
        {
            std::eprintln!($($arg)*);
        }
    }};
}

/// Identifier for a type stored in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeId(pub usize);

/// 長寿命 cache で名義型定義を区別するための型種別。
///
/// `TypeKind` の arena slot は compile session ごとに変わるため、Resource summary などの
/// cache key には直接入れない。この enum は stable identity に含める公開境界であり、
/// `Struct` と `Enum` を同名でも別の定義として扱う。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NominalStableTypeKind {
    Enum,
    Struct,
}

/// compile session をまたいで名義型定義を対応付ける identity。
///
/// `source_path` は loader / `SourceMap` から得た正規化済み path 文字列、`definition_hash`
/// は field / variant / type parameter の型境界から作る fingerprint である。`TypeId` や
/// `Span` を保存せず、同じ path/name でも定義形状が変わった場合は別 identity にする。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NominalStableTypeIdentity {
    kind: NominalStableTypeKind,
    source_path: String,
    name: String,
    arity: usize,
    definition_hash: u64,
}

impl NominalStableTypeIdentity {
    pub fn new(
        kind: NominalStableTypeKind,
        source_path: String,
        name: String,
        arity: usize,
        definition_hash: u64,
    ) -> Self {
        Self {
            kind,
            source_path,
            name,
            arity,
            definition_hash,
        }
    }

    pub fn stable_key_component(&self) -> String {
        format!(
            "nominal(kind={},path={},name={},arity={},hash={:016x})",
            self.kind.tag(),
            stable_text_component(&self.source_path),
            stable_text_component(&self.name),
            self.arity,
            self.definition_hash
        )
    }

    pub fn kind(&self) -> NominalStableTypeKind {
        self.kind
    }

    pub fn source_path(&self) -> &str {
        self.source_path.as_str()
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn definition_hash(&self) -> u64 {
        self.definition_hash
    }
}

impl NominalStableTypeKind {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Enum => "enum",
            Self::Struct => "struct",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Unit,
    I32,
    U8,
    F32,
    Bool,
    Char,
    Str,
    Never,
    Named(String),
    Enum {
        name: String,
        type_params: Vec<TypeId>, // TypeId(Var)
        variants: Vec<EnumVariantInfo>,
    },
    Struct {
        name: String,
        type_params: Vec<TypeId>, // TypeId(Var)
        fields: Vec<TypeId>,
        field_names: Vec<String>,
    },
    Tuple {
        items: Vec<TypeId>,
    },
    Function {
        type_params: Vec<TypeId>, // new
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
    },
    Var(TypeVar),
    Apply {
        base: TypeId,
        args: Vec<TypeId>,
    },
    Box(TypeId),
    Reference(TypeId, bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NominalApplyKind {
    Enum,
    Struct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeVar {
    pub label: Option<alloc::string::String>,
    pub binding: Option<TypeId>,
    pub copy_cap: bool,
    pub clone_cap: bool,
    pub drop_cap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumVariantInfo {
    pub name: alloc::string::String,
    pub payload: Option<TypeId>,
}

/// Arena-based type context with simple unification.
#[derive(Debug)]
pub struct TypeCtx {
    arena: Vec<TypeKind>,
    unit: TypeId,
    i32_ty: TypeId,
    u8_ty: TypeId,
    f32_ty: TypeId,
    bool_ty: TypeId,
    char_ty: TypeId,
    str_ty: TypeId,
    never_ty: TypeId,
    named: alloc::collections::BTreeMap<alloc::string::String, TypeId>,
    nominal_identities: BTreeMap<TypeId, NominalStableTypeIdentity>,
    copy_impl_targets: Vec<TypeId>,
    clone_impl_targets: Vec<TypeId>,
    copy_trait_enabled: bool,
    drop_impl_targets: Vec<TypeId>,
    compiler_memory_types: Vec<(TypeId, CompilerMemoryType)>,
    undo_log: Vec<TypeCtxUndo>,
    active_snapshots: usize,
}

#[derive(Debug, Clone)]
enum TypeCtxUndo {
    Arena {
        id: TypeId,
        previous: TypeKind,
    },
    Named {
        name: String,
        previous: Option<TypeId>,
    },
    NominalIdentity {
        id: TypeId,
        previous: Option<NominalStableTypeIdentity>,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct TypeCtxCheckpoint {
    arena_len: usize,
    undo_len: usize,
    copy_impl_targets_len: usize,
    clone_impl_targets_len: usize,
    drop_impl_targets_len: usize,
    compiler_memory_types_len: usize,
    copy_trait_enabled: bool,
    active_snapshots: usize,
}

#[derive(Default)]
struct CapabilityQueryStack {
    copy: BTreeSet<TypeId>,
    clone: BTreeSet<TypeId>,
    drop: BTreeSet<TypeId>,
}

impl Clone for TypeCtx {
    fn clone(&self) -> Self {
        Self {
            arena: self.arena.clone(),
            unit: self.unit,
            i32_ty: self.i32_ty,
            u8_ty: self.u8_ty,
            f32_ty: self.f32_ty,
            bool_ty: self.bool_ty,
            char_ty: self.char_ty,
            str_ty: self.str_ty,
            never_ty: self.never_ty,
            named: self.named.clone(),
            nominal_identities: self.nominal_identities.clone(),
            copy_impl_targets: self.copy_impl_targets.clone(),
            clone_impl_targets: self.clone_impl_targets.clone(),
            copy_trait_enabled: self.copy_trait_enabled,
            drop_impl_targets: self.drop_impl_targets.clone(),
            compiler_memory_types: self.compiler_memory_types.clone(),
            undo_log: Vec::new(),
            active_snapshots: 0,
        }
    }
}

static GLOBAL_UNIFY_DEPTH: AtomicUsize = AtomicUsize::new(0);

struct UnifyDepthGuard;
impl Drop for UnifyDepthGuard {
    fn drop(&mut self) {
        GLOBAL_UNIFY_DEPTH.fetch_sub(1, Ordering::SeqCst);
    }
}

impl TypeCtx {
    pub fn new() -> Self {
        let mut arena = Vec::new();
        let unit = TypeId(arena.len());
        arena.push(TypeKind::Unit);
        let i32_ty = TypeId(arena.len());
        arena.push(TypeKind::I32);
        let u8_ty = TypeId(arena.len());
        arena.push(TypeKind::U8);
        let f32_ty = TypeId(arena.len());
        arena.push(TypeKind::F32);
        let bool_ty = TypeId(arena.len());
        arena.push(TypeKind::Bool);
        let char_ty = TypeId(arena.len());
        arena.push(TypeKind::Char);
        let str_ty = TypeId(arena.len());
        arena.push(TypeKind::Str);
        let never_ty = TypeId(arena.len());
        arena.push(TypeKind::Never);

        Self {
            arena,
            unit,
            i32_ty,
            u8_ty,
            f32_ty,
            bool_ty,
            char_ty,
            str_ty,
            never_ty,
            named: alloc::collections::BTreeMap::new(),
            nominal_identities: BTreeMap::new(),
            copy_impl_targets: Vec::new(),
            clone_impl_targets: Vec::new(),
            copy_trait_enabled: false,
            drop_impl_targets: Vec::new(),
            compiler_memory_types: Vec::new(),
            undo_log: Vec::new(),
            active_snapshots: 0,
        }
    }

    pub fn checkpoint(&mut self) -> TypeCtxCheckpoint {
        let checkpoint = TypeCtxCheckpoint {
            arena_len: self.arena.len(),
            undo_len: self.undo_log.len(),
            copy_impl_targets_len: self.copy_impl_targets.len(),
            clone_impl_targets_len: self.clone_impl_targets.len(),
            drop_impl_targets_len: self.drop_impl_targets.len(),
            compiler_memory_types_len: self.compiler_memory_types.len(),
            copy_trait_enabled: self.copy_trait_enabled,
            active_snapshots: self.active_snapshots,
        };
        self.active_snapshots += 1;
        checkpoint
    }

    pub fn rollback(&mut self, checkpoint: TypeCtxCheckpoint) {
        while self.undo_log.len() > checkpoint.undo_len {
            match self.undo_log.pop().unwrap() {
                TypeCtxUndo::Arena { id, previous } => {
                    if id.0 < self.arena.len() {
                        self.arena[id.0] = previous;
                    }
                }
                TypeCtxUndo::Named { name, previous } => {
                    if let Some(previous) = previous {
                        self.named.insert(name, previous);
                    } else {
                        self.named.remove(&name);
                    }
                }
                TypeCtxUndo::NominalIdentity { id, previous } => {
                    if let Some(previous) = previous {
                        self.nominal_identities.insert(id, previous);
                    } else {
                        self.nominal_identities.remove(&id);
                    }
                }
            }
        }
        self.arena.truncate(checkpoint.arena_len);
        self.nominal_identities
            .retain(|id, _| id.0 < checkpoint.arena_len);
        self.copy_impl_targets
            .truncate(checkpoint.copy_impl_targets_len);
        self.clone_impl_targets
            .truncate(checkpoint.clone_impl_targets_len);
        self.drop_impl_targets
            .truncate(checkpoint.drop_impl_targets_len);
        self.compiler_memory_types
            .truncate(checkpoint.compiler_memory_types_len);
        self.copy_trait_enabled = checkpoint.copy_trait_enabled;
        self.active_snapshots = checkpoint.active_snapshots;
    }

    /// Keep the changes made after a checkpoint and close that checkpoint.
    ///
    /// The undo log is still retained when an outer checkpoint exists, because
    /// the outer rollback must be able to undo the committed inner changes. If
    /// there is no outer checkpoint, the speculative undo entries can be
    /// discarded while the arena and binding changes remain committed.
    pub fn commit(&mut self, checkpoint: TypeCtxCheckpoint) {
        if checkpoint.active_snapshots == 0 {
            self.undo_log.truncate(checkpoint.undo_len);
        }
        self.active_snapshots = checkpoint.active_snapshots;
    }

    fn record_arena_update(&mut self, id: TypeId) {
        if self.active_snapshots == 0 || id.0 >= self.arena.len() {
            return;
        }
        self.undo_log.push(TypeCtxUndo::Arena {
            id,
            previous: self.arena[id.0].clone(),
        });
    }

    fn record_named_update(&mut self, name: &String) {
        if self.active_snapshots == 0 {
            return;
        }
        self.undo_log.push(TypeCtxUndo::Named {
            name: name.clone(),
            previous: self.named.get(name).copied(),
        });
    }

    fn record_nominal_identity_update(&mut self, id: TypeId) {
        if self.active_snapshots == 0 {
            return;
        }
        self.undo_log.push(TypeCtxUndo::NominalIdentity {
            id,
            previous: self.nominal_identities.get(&id).cloned(),
        });
    }

    pub fn unit(&self) -> TypeId {
        self.unit
    }
    pub fn i32(&self) -> TypeId {
        self.i32_ty
    }
    pub fn u8(&self) -> TypeId {
        self.u8_ty
    }
    pub fn f32(&self) -> TypeId {
        self.f32_ty
    }
    pub fn bool(&self) -> TypeId {
        self.bool_ty
    }
    pub fn char(&self) -> TypeId {
        self.char_ty
    }
    pub fn str(&self) -> TypeId {
        self.str_ty
    }
    pub fn never(&self) -> TypeId {
        self.never_ty
    }

    /// 現在の `TypeCtx` に存在する型 arena slot を決定的な順序で走査する。
    ///
    /// Resource summary cache のように stable key から現在 session の `TypeId` を探す処理は、
    /// `TypeId` そのものを cache key に入れず、arena 内の型を stable 表現へ変換して照合する。
    pub(crate) fn type_ids(&self) -> impl Iterator<Item = TypeId> + '_ {
        (0..self.arena.len()).map(TypeId)
    }

    pub fn fresh_var(&mut self, label: Option<alloc::string::String>) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Var(TypeVar {
            label,
            binding: None,
            copy_cap: false,
            clone_cap: false,
            drop_cap: false,
        }));
        id
    }

    pub fn set_var_capabilities(
        &mut self,
        var: TypeId,
        copy_cap: bool,
        clone_cap: bool,
        drop_cap: bool,
    ) {
        self.record_arena_update(var);
        if let TypeKind::Var(tv) = &mut self.arena[var.0] {
            tv.copy_cap = copy_cap;
            tv.clone_cap = clone_cap;
            tv.drop_cap = drop_cap;
        }
    }

    pub fn snapshot_type_var_bindings(
        &self,
        ty: TypeId,
    ) -> alloc::collections::BTreeMap<TypeId, Option<TypeId>> {
        let mut out = alloc::collections::BTreeMap::new();
        let mut seen = BTreeSet::new();
        self.collect_type_var_bindings(ty, &mut seen, &mut out);
        out
    }

    pub fn restore_type_var_bindings(
        &mut self,
        snapshot: &alloc::collections::BTreeMap<TypeId, Option<TypeId>>,
    ) {
        for (var, binding) in snapshot {
            self.record_arena_update(*var);
            if let TypeKind::Var(tv) = &mut self.arena[var.0] {
                tv.binding = *binding;
            }
        }
    }

    fn collect_type_var_bindings(
        &self,
        ty: TypeId,
        seen: &mut BTreeSet<TypeId>,
        out: &mut alloc::collections::BTreeMap<TypeId, Option<TypeId>>,
    ) {
        if !seen.insert(ty) {
            return;
        }
        match &self.arena[ty.0] {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str
            | TypeKind::Never
            | TypeKind::Named(_) => {}
            TypeKind::Var(tv) => {
                out.insert(ty, tv.binding);
                if let Some(binding) = tv.binding {
                    self.collect_type_var_bindings(binding, seen, out);
                }
            }
            TypeKind::Enum {
                type_params,
                variants,
                ..
            } => {
                for tp in type_params {
                    self.collect_type_var_bindings(*tp, seen, out);
                }
                for variant in variants {
                    if let Some(payload) = variant.payload {
                        self.collect_type_var_bindings(payload, seen, out);
                    }
                }
            }
            TypeKind::Struct {
                type_params,
                fields,
                ..
            } => {
                for tp in type_params {
                    self.collect_type_var_bindings(*tp, seen, out);
                }
                for field in fields {
                    self.collect_type_var_bindings(*field, seen, out);
                }
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                ..
            } => {
                for tp in type_params {
                    self.collect_type_var_bindings(*tp, seen, out);
                }
                for param in params {
                    self.collect_type_var_bindings(*param, seen, out);
                }
                self.collect_type_var_bindings(*result, seen, out);
            }
            TypeKind::Tuple { items } => {
                for item in items {
                    self.collect_type_var_bindings(*item, seen, out);
                }
            }
            TypeKind::Apply { base, args } => {
                self.collect_type_var_bindings(*base, seen, out);
                for arg in args {
                    self.collect_type_var_bindings(*arg, seen, out);
                }
            }
            TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
                self.collect_type_var_bindings(*inner, seen, out);
            }
        }
    }

    pub fn register_named(&mut self, name: alloc::string::String, kind: TypeKind) -> TypeId {
        if let Some(existing) = self.named.get(&name) {
            // upgrade placeholder Named to concrete kind
            let eid = *existing;
            match &self.arena[eid.0] {
                TypeKind::Named(_) => {
                    self.record_arena_update(eid);
                    self.arena[eid.0] = kind;
                }
                _ => {}
            }
            eid
        } else {
            let id = TypeId(self.arena.len());
            self.arena.push(kind);
            self.record_named_update(&name);
            self.named.insert(name, id);
            id
        }
    }

    pub fn register_named_with_stable_identity(
        &mut self,
        name: alloc::string::String,
        kind: TypeKind,
        identity: NominalStableTypeIdentity,
    ) -> TypeId {
        let id = self.register_named(name, kind);
        self.record_nominal_identity_update(id);
        self.nominal_identities.insert(id, identity);
        id
    }

    pub fn nominal_stable_identity(&self, id: TypeId) -> Option<&NominalStableTypeIdentity> {
        let resolved = self.resolve_named_type_id(id);
        if let Some(identity) = self.nominal_identities.get(&resolved) {
            return Some(identity);
        }
        let name = match self.get_ref(resolved) {
            TypeKind::Enum { name, .. } | TypeKind::Struct { name, .. } => name,
            _ => return None,
        };
        let named_id = self.named.get(name).copied()?;
        let named_id = self.resolve_named_type_id(named_id);
        self.nominal_identities.get(&named_id)
    }

    pub fn nominal_definition_hash(&self, kind: &TypeKind) -> Option<u64> {
        let mut hash = NominalDefinitionHasher::new("neplg2-nominal-definition-surface-v1");
        match kind {
            TypeKind::Enum {
                type_params,
                variants,
                ..
            } => {
                hash.write_str("enum");
                self.hash_nominal_type_list(&mut hash, type_params)?;
                hash.write_usize(variants.len());
                for variant in variants {
                    hash.write_str(&variant.name);
                    match variant.payload {
                        Some(payload) => {
                            hash.write_bool(true);
                            self.hash_nominal_type_surface(
                                &mut hash,
                                payload,
                                &mut BTreeSet::new(),
                            )?;
                        }
                        None => hash.write_bool(false),
                    }
                }
            }
            TypeKind::Struct {
                type_params,
                fields,
                field_names,
                ..
            } => {
                if fields.len() != field_names.len() {
                    return None;
                }
                hash.write_str("struct");
                self.hash_nominal_type_list(&mut hash, type_params)?;
                hash.write_usize(fields.len());
                for (field_name, field_ty) in field_names.iter().zip(fields.iter()) {
                    hash.write_str(field_name);
                    self.hash_nominal_type_surface(&mut hash, *field_ty, &mut BTreeSet::new())?;
                }
            }
            _ => return None,
        }
        Some(hash.finish())
    }

    fn hash_nominal_type_list(
        &self,
        hash: &mut NominalDefinitionHasher,
        items: &[TypeId],
    ) -> Option<()> {
        hash.write_usize(items.len());
        for item in items {
            self.hash_nominal_type_surface(hash, *item, &mut BTreeSet::new())?;
        }
        Some(())
    }

    fn hash_nominal_type_surface(
        &self,
        hash: &mut NominalDefinitionHasher,
        ty: TypeId,
        seen: &mut BTreeSet<TypeId>,
    ) -> Option<()> {
        let resolved = self.resolve_named_type_id(ty);
        if !seen.insert(resolved) {
            return None;
        }
        let result = match self.get_ref(resolved) {
            TypeKind::Unit => {
                hash.write_str("unit");
                Some(())
            }
            TypeKind::I32 => {
                hash.write_str("i32");
                Some(())
            }
            TypeKind::U8 => {
                hash.write_str("u8");
                Some(())
            }
            TypeKind::F32 => {
                hash.write_str("f32");
                Some(())
            }
            TypeKind::Bool => {
                hash.write_str("bool");
                Some(())
            }
            TypeKind::Char => {
                hash.write_str("char");
                Some(())
            }
            TypeKind::Str => {
                hash.write_str("str");
                Some(())
            }
            TypeKind::Never => {
                hash.write_str("never");
                Some(())
            }
            TypeKind::Named(name) => {
                if let Some(identity) = self.nominal_stable_identity(resolved) {
                    hash.write_str(identity.stable_key_component().as_str());
                    Some(())
                } else {
                    let scalar = BackendScalarType::from_name(name.as_str())?;
                    hash.write_str("backend-scalar");
                    hash.write_str(scalar.source_name());
                    Some(())
                }
            }
            TypeKind::Enum { .. } | TypeKind::Struct { .. } => {
                let identity = self.nominal_stable_identity(resolved)?;
                hash.write_str(identity.stable_key_component().as_str());
                Some(())
            }
            TypeKind::Tuple { items } => {
                hash.write_str("tuple");
                self.hash_nominal_type_list(hash, items)
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } => {
                hash.write_str("fn");
                hash.write_str(match effect {
                    Effect::Pure => "pure",
                    Effect::Impure => "impure",
                });
                self.hash_nominal_type_list(hash, type_params)?;
                self.hash_nominal_type_list(hash, params)?;
                self.hash_nominal_type_surface(hash, *result, seen)
            }
            TypeKind::Var(var) => match var.binding {
                Some(binding) => self.hash_nominal_type_surface(hash, binding, seen),
                None => {
                    hash.write_str("var");
                    hash.write_str(var.label.as_deref()?);
                    hash.write_bool(var.copy_cap);
                    hash.write_bool(var.clone_cap);
                    hash.write_bool(var.drop_cap);
                    Some(())
                }
            },
            TypeKind::Apply { base, args } => {
                hash.write_str("apply");
                self.hash_nominal_type_surface(hash, *base, seen)?;
                self.hash_nominal_type_list(hash, args)
            }
            TypeKind::Box(inner) => {
                hash.write_str("box");
                self.hash_nominal_type_surface(hash, *inner, seen)
            }
            TypeKind::Reference(inner, is_mut) => {
                hash.write_str("ref");
                hash.write_bool(*is_mut);
                self.hash_nominal_type_surface(hash, *inner, seen)
            }
        };
        seen.remove(&resolved);
        result
    }

    pub fn lookup_named(&self, name: &str) -> Option<TypeId> {
        self.named.get(name).copied()
    }

    pub fn function(
        &mut self,
        type_params: Vec<TypeId>,
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
    ) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        });
        id
    }

    pub fn tuple(&mut self, items: Vec<TypeId>) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Tuple { items });
        id
    }

    pub fn resolve_id(&self, id: TypeId) -> TypeId {
        let mut cur = id;
        let mut i = 0;
        loop {
            if i > 5000 {
                return cur;
            }
            match &self.arena[cur.0] {
                TypeKind::Var(tv) => {
                    if let Some(next) = tv.binding {
                        cur = next;
                    } else {
                        return cur;
                    }
                }
                _ => return cur,
            }
            i += 1;
        }
    }

    /// 型変数の束縛と名前付き型の実体をたどり、名義型判定やレイアウト計算で使う ID を返す。
    pub fn resolve_named_type_id(&self, id: TypeId) -> TypeId {
        let mut cur = self.resolve_id(id);
        let mut i = 0;
        loop {
            if i > 5000 {
                return cur;
            }
            match &self.arena[cur.0] {
                TypeKind::Named(name) => {
                    let Some(next) = self.named.get(name).copied() else {
                        return cur;
                    };
                    let next = self.resolve_id(next);
                    if next == cur {
                        return cur;
                    }
                    cur = next;
                }
                _ => return cur,
            }
            i += 1;
        }
    }

    fn nominal_apply_base(
        &self,
        base: TypeId,
    ) -> Option<(NominalApplyKind, String, usize, TypeId)> {
        let base = self.resolve_named_type_id(base);
        match &self.arena[base.0] {
            TypeKind::Enum {
                name, type_params, ..
            } => Some((
                NominalApplyKind::Enum,
                name.clone(),
                type_params.len(),
                base,
            )),
            TypeKind::Struct {
                name, type_params, ..
            } => Some((
                NominalApplyKind::Struct,
                name.clone(),
                type_params.len(),
                base,
            )),
            _ => None,
        }
    }

    fn nominal_apply_bases_match(&self, left: TypeId, right: TypeId) -> bool {
        match (
            self.nominal_apply_base(left),
            self.nominal_apply_base(right),
        ) {
            (
                Some((left_kind, left_name, left_arity, _)),
                Some((right_kind, right_name, right_arity, _)),
            ) => left_kind == right_kind && left_name == right_name && left_arity == right_arity,
            _ => false,
        }
    }

    fn is_nominal_definition_id(&self, id: TypeId, kind: NominalApplyKind, name: &str) -> bool {
        let Some(named_id) = self.named.get(name).copied() else {
            return false;
        };
        let id = self.resolve_named_type_id(id);
        let named_id = self.resolve_named_type_id(named_id);
        if id != named_id {
            return false;
        }
        matches!(
            (kind, self.arena.get(id.0)),
            (NominalApplyKind::Enum, Some(TypeKind::Enum { .. }))
                | (NominalApplyKind::Struct, Some(TypeKind::Struct { .. }))
        )
    }

    fn unify_apply_bases(&mut self, left: TypeId, right: TypeId) -> Result<(), UnifyError> {
        match (
            self.nominal_apply_base(left),
            self.nominal_apply_base(right),
        ) {
            (
                Some((left_kind, left_name, left_arity, _)),
                Some((right_kind, right_name, right_arity, _)),
            ) if left_kind == right_kind
                && left_name == right_name
                && left_arity == right_arity =>
            {
                Ok(())
            }
            (Some(_), Some(_)) => Err(UnifyError::Mismatch),
            _ => self.unify(left, right).map(|_| ()),
        }
    }

    pub fn register_copy_impl_target(&mut self, id: TypeId) {
        let resolved = self.resolve_id(id);
        if self
            .copy_impl_targets
            .iter()
            .any(|t| self.same_type(*t, resolved))
        {
            return;
        }
        self.copy_impl_targets.push(resolved);
    }

    pub fn has_copy_impl_target(&self, id: TypeId) -> bool {
        let mut stack = CapabilityQueryStack::default();
        self.has_copy_impl_target_inner(id, &mut stack)
    }

    fn has_copy_impl_target_inner(&self, id: TypeId, stack: &mut CapabilityQueryStack) -> bool {
        let resolved = self.resolve_id(id);
        self.copy_impl_targets
            .iter()
            .any(|t| self.type_pattern_matches_with_capability_stack(*t, resolved, stack))
    }

    pub fn register_clone_impl_target(&mut self, id: TypeId) {
        let resolved = self.resolve_id(id);
        if self
            .clone_impl_targets
            .iter()
            .any(|t| self.same_type(*t, resolved))
        {
            return;
        }
        self.clone_impl_targets.push(resolved);
    }

    pub fn has_clone_impl_target(&self, id: TypeId) -> bool {
        let mut stack = CapabilityQueryStack::default();
        self.has_clone_impl_target_inner(id, &mut stack)
    }

    fn has_clone_impl_target_inner(&self, id: TypeId, stack: &mut CapabilityQueryStack) -> bool {
        let resolved = self.resolve_id(id);
        self.clone_impl_targets
            .iter()
            .any(|t| self.type_pattern_matches_with_capability_stack(*t, resolved, stack))
    }

    pub fn register_drop_impl_target(&mut self, id: TypeId) {
        let resolved = self.resolve_id(id);
        if self
            .drop_impl_targets
            .iter()
            .any(|t| self.same_type(*t, resolved))
        {
            return;
        }
        self.drop_impl_targets.push(resolved);
    }

    pub fn mark_compiler_memory_type(&mut self, id: TypeId, memory_type: CompilerMemoryType) {
        let resolved = self.resolve_named_type_id(self.resolve_id(id));
        if self
            .compiler_memory_types
            .iter()
            .any(|(ty, registered)| *ty == resolved && *registered == memory_type)
        {
            return;
        }
        self.compiler_memory_types.push((resolved, memory_type));
    }

    pub fn compiler_memory_type(&self, id: TypeId) -> Option<CompilerMemoryType> {
        let resolved = self.resolve_named_type_id(self.resolve_id(id));
        self.compiler_memory_types
            .iter()
            .find_map(|(ty, memory_type)| (*ty == resolved).then_some(*memory_type))
    }

    pub fn has_drop_impl_target(&self, id: TypeId) -> bool {
        let mut stack = CapabilityQueryStack::default();
        self.has_drop_impl_target_inner(id, &mut stack)
    }

    fn has_drop_impl_target_inner(&self, id: TypeId, stack: &mut CapabilityQueryStack) -> bool {
        let resolved = self.resolve_id(id);
        self.drop_impl_targets
            .iter()
            .any(|t| self.type_pattern_matches_with_capability_stack(*t, resolved, stack))
    }

    pub fn type_pattern_matches(&self, pattern: TypeId, actual: TypeId) -> bool {
        let mut stack = CapabilityQueryStack::default();
        self.type_pattern_matches_with_capability_stack(pattern, actual, &mut stack)
    }

    fn type_pattern_matches_with_capability_stack(
        &self,
        pattern: TypeId,
        actual: TypeId,
        stack: &mut CapabilityQueryStack,
    ) -> bool {
        let mut seen = BTreeSet::new();
        let mut mapping = BTreeMap::new();
        self.type_pattern_matches_inner(
            self.resolve_id(pattern),
            self.resolve_id(actual),
            &mut mapping,
            &mut seen,
            stack,
        )
    }

    fn type_pattern_matches_inner(
        &self,
        pattern: TypeId,
        actual: TypeId,
        mapping: &mut BTreeMap<TypeId, TypeId>,
        seen: &mut BTreeSet<(TypeId, TypeId)>,
        stack: &mut CapabilityQueryStack,
    ) -> bool {
        let pattern = self.resolve_id(pattern);
        let actual = self.resolve_id(actual);
        if !seen.insert((pattern, actual)) {
            return true;
        }
        match (self.get_ref(pattern), self.get_ref(actual)) {
            (TypeKind::Var(v), _) => {
                if let Some(bound) = v.binding {
                    return self.type_pattern_matches_inner(bound, actual, mapping, seen, stack);
                }
                if !self.pattern_var_capabilities_match(v, actual, stack) {
                    return false;
                }
                match mapping.get(&pattern).copied() {
                    Some(prev) => self.same_type(prev, actual),
                    None => {
                        mapping.insert(pattern, actual);
                        true
                    }
                }
            }
            (TypeKind::Unit, TypeKind::Unit)
            | (TypeKind::I32, TypeKind::I32)
            | (TypeKind::U8, TypeKind::U8)
            | (TypeKind::F32, TypeKind::F32)
            | (TypeKind::Bool, TypeKind::Bool)
            | (TypeKind::Char, TypeKind::Char)
            | (TypeKind::Str, TypeKind::Str)
            | (TypeKind::Never, TypeKind::Never) => true,
            (TypeKind::Named(a), TypeKind::Named(b)) => a == b,
            (TypeKind::Reference(ai, am), TypeKind::Reference(bi, bm)) => {
                am == bm && self.type_pattern_matches_inner(*ai, *bi, mapping, seen, stack)
            }
            (TypeKind::Box(ai), TypeKind::Box(bi)) => {
                self.type_pattern_matches_inner(*ai, *bi, mapping, seen, stack)
            }
            (TypeKind::Tuple { items: a }, TypeKind::Tuple { items: b }) => {
                a.len() == b.len()
                    && a.iter()
                        .zip(b.iter())
                        .all(|(x, y)| self.type_pattern_matches_inner(*x, *y, mapping, seen, stack))
            }
            (
                TypeKind::Function {
                    type_params: a_tps,
                    params: a_ps,
                    result: a_r,
                    ..
                },
                TypeKind::Function {
                    type_params: b_tps,
                    params: b_ps,
                    result: b_r,
                    ..
                },
            ) => {
                a_tps.len() == b_tps.len()
                    && a_ps.len() == b_ps.len()
                    && a_ps
                        .iter()
                        .zip(b_ps.iter())
                        .all(|(x, y)| self.type_pattern_matches_inner(*x, *y, mapping, seen, stack))
                    && self.type_pattern_matches_inner(*a_r, *b_r, mapping, seen, stack)
            }
            (
                TypeKind::Apply {
                    base: a_base,
                    args: a_args,
                },
                TypeKind::Apply {
                    base: b_base,
                    args: b_args,
                },
            ) => {
                let bases_match = if self.nominal_apply_base(*a_base).is_some()
                    || self.nominal_apply_base(*b_base).is_some()
                {
                    self.nominal_apply_bases_match(*a_base, *b_base)
                } else {
                    self.type_pattern_matches_inner(*a_base, *b_base, mapping, seen, stack)
                };
                bases_match
                    && a_args.len() == b_args.len()
                    && a_args
                        .iter()
                        .zip(b_args.iter())
                        .all(|(x, y)| self.type_pattern_matches_inner(*x, *y, mapping, seen, stack))
            }
            (
                TypeKind::Struct {
                    name: a_name,
                    type_params: a_tps,
                    fields: a_fields,
                    ..
                },
                TypeKind::Struct {
                    name: b_name,
                    type_params: b_tps,
                    fields: b_fields,
                    ..
                },
            ) => {
                a_name == b_name
                    && a_tps.len() == b_tps.len()
                    && a_fields.len() == b_fields.len()
                    && a_tps
                        .iter()
                        .zip(b_tps.iter())
                        .all(|(x, y)| self.type_pattern_matches_inner(*x, *y, mapping, seen, stack))
                    && a_fields
                        .iter()
                        .zip(b_fields.iter())
                        .all(|(x, y)| self.type_pattern_matches_inner(*x, *y, mapping, seen, stack))
            }
            (
                TypeKind::Enum {
                    name: a_name,
                    type_params: a_tps,
                    variants: a_vs,
                    ..
                },
                TypeKind::Enum {
                    name: b_name,
                    type_params: b_tps,
                    variants: b_vs,
                    ..
                },
            ) => {
                a_name == b_name
                    && a_tps.len() == b_tps.len()
                    && a_vs.len() == b_vs.len()
                    && a_tps
                        .iter()
                        .zip(b_tps.iter())
                        .all(|(x, y)| self.type_pattern_matches_inner(*x, *y, mapping, seen, stack))
                    && a_vs.iter().zip(b_vs.iter()).all(|(x, y)| {
                        x.name == y.name
                            && match (x.payload, y.payload) {
                                (Some(px), Some(py)) => {
                                    self.type_pattern_matches_inner(px, py, mapping, seen, stack)
                                }
                                (None, None) => true,
                                _ => false,
                            }
                    })
            }
            _ => false,
        }
    }

    fn pattern_var_capabilities_match(
        &self,
        var: &TypeVar,
        actual: TypeId,
        stack: &mut CapabilityQueryStack,
    ) -> bool {
        (!var.copy_cap || self.is_copy_inner(actual, stack))
            && (!var.clone_cap || self.has_clone_inner(actual, stack))
            && (!var.drop_cap || self.has_drop_inner(actual, stack))
    }

    pub fn set_copy_trait_enabled(&mut self, enabled: bool) {
        self.copy_trait_enabled = enabled;
    }

    pub fn is_copy_eligible(&self, id: TypeId) -> bool {
        let mut visiting = BTreeSet::new();
        let mapping = BTreeMap::new();
        self.is_copy_eligible_inner(id, &mut visiting, &mapping, false)
    }

    pub fn is_copy_impl_eligible(&self, id: TypeId) -> bool {
        let mut visiting = BTreeSet::new();
        let mapping = BTreeMap::new();
        self.is_copy_eligible_inner(id, &mut visiting, &mapping, true)
    }

    pub fn is_copy(&self, id: TypeId) -> bool {
        let mut stack = CapabilityQueryStack::default();
        self.is_copy_inner(id, &mut stack)
    }

    fn is_copy_inner(&self, id: TypeId, stack: &mut CapabilityQueryStack) -> bool {
        if self.copy_trait_enabled {
            return self.is_copy_with_trait_model_inner(id, stack);
        }
        let resolved = self.resolve_id(id);
        match self.get_ref(resolved) {
            TypeKind::Never => true,
            TypeKind::Reference(_, is_mut) => !*is_mut,
            TypeKind::Var(v) => v
                .binding
                .map(|b| self.is_copy_inner(b, stack))
                .unwrap_or(v.copy_cap),
            _ => false,
        }
    }

    fn is_copy_with_trait_model_inner(&self, id: TypeId, stack: &mut CapabilityQueryStack) -> bool {
        let resolved = self.resolve_id(id);
        if !stack.copy.insert(resolved) {
            return false;
        }
        let result = match self.get_ref(resolved) {
            TypeKind::Never => true,
            TypeKind::Reference(_, is_mut) => !*is_mut,
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str => self.has_copy_impl_target_inner(resolved, stack),
            TypeKind::Named(_) => self.has_copy_impl_target_inner(resolved, stack),
            TypeKind::Tuple { items } => items.iter().all(|t| self.is_copy_inner(*t, stack)),
            TypeKind::Struct { .. } | TypeKind::Enum { .. } => {
                self.has_copy_impl_target_inner(resolved, stack)
            }
            TypeKind::Apply { base, .. } => match self.get_ref(self.resolve_id(*base)) {
                TypeKind::Struct { .. } | TypeKind::Enum { .. } => {
                    self.has_copy_impl_target_inner(resolved, stack)
                }
                _ => self.has_copy_impl_target_inner(resolved, stack),
            },
            TypeKind::Var(v) => v
                .binding
                .map(|b| self.is_copy_inner(b, stack))
                .unwrap_or(v.copy_cap),
            TypeKind::Function { .. } => true,
            TypeKind::Box(_) => false,
        };
        stack.copy.remove(&resolved);
        result
    }

    pub fn has_clone(&self, id: TypeId) -> bool {
        let mut stack = CapabilityQueryStack::default();
        self.has_clone_inner(id, &mut stack)
    }

    fn has_clone_inner(&self, id: TypeId, stack: &mut CapabilityQueryStack) -> bool {
        let resolved = self.resolve_id(id);
        if !stack.clone.insert(resolved) {
            return false;
        }
        let result = match self.get_ref(resolved) {
            TypeKind::Never => true,
            TypeKind::Reference(_, is_mut) => !*is_mut,
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str => self.has_clone_impl_target_inner(resolved, stack),
            TypeKind::Named(_) => self.has_clone_impl_target_inner(resolved, stack),
            TypeKind::Tuple { items } => {
                items.iter().all(|t| self.has_clone_inner(*t, stack))
                    || self.has_clone_impl_target_inner(resolved, stack)
            }
            TypeKind::Struct { .. } | TypeKind::Enum { .. } => {
                self.has_clone_impl_target_inner(resolved, stack)
            }
            TypeKind::Apply { .. } => self.has_clone_impl_target_inner(resolved, stack),
            TypeKind::Box(_) => false,
            TypeKind::Function { .. } => true,
            TypeKind::Var(v) => v
                .binding
                .map(|b| self.has_clone_inner(b, stack))
                .unwrap_or(v.clone_cap || v.copy_cap),
        };
        stack.clone.remove(&resolved);
        result
    }

    pub fn has_drop(&self, id: TypeId) -> bool {
        let mut stack = CapabilityQueryStack::default();
        self.has_drop_inner(id, &mut stack)
    }

    fn has_drop_inner(&self, id: TypeId, stack: &mut CapabilityQueryStack) -> bool {
        let resolved = self.resolve_id(id);
        if !stack.drop.insert(resolved) {
            return false;
        }
        let result = match self.get_ref(resolved) {
            TypeKind::Never => false,
            TypeKind::Reference(_, _) => false,
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str => self.has_drop_impl_target_inner(resolved, stack),
            TypeKind::Named(_) => self.has_drop_impl_target_inner(resolved, stack),
            TypeKind::Tuple { items } => {
                items.iter().any(|t| self.has_drop_inner(*t, stack))
                    || self.has_drop_impl_target_inner(resolved, stack)
            }
            TypeKind::Struct { .. } | TypeKind::Enum { .. } => {
                self.has_drop_impl_target_inner(resolved, stack)
            }
            TypeKind::Apply { .. } | TypeKind::Box(_) => {
                self.has_drop_impl_target_inner(resolved, stack)
            }
            TypeKind::Function { .. } => false,
            TypeKind::Var(v) => v
                .binding
                .map(|b| self.has_drop_inner(b, stack))
                .unwrap_or(v.drop_cap),
        };
        stack.drop.remove(&resolved);
        result
    }

    fn is_copy_eligible_inner(
        &self,
        id: TypeId,
        visiting: &mut BTreeSet<TypeId>,
        mapping: &BTreeMap<TypeId, TypeId>,
        allow_opaque_named: bool,
    ) -> bool {
        let resolved = mapping
            .get(&self.resolve_id(id))
            .copied()
            .unwrap_or_else(|| self.resolve_id(id));
        if !visiting.insert(resolved) {
            return false;
        }
        let result = match self.get_ref(resolved) {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str
            | TypeKind::Never => true,
            TypeKind::Reference(_, is_mut) => !*is_mut,
            TypeKind::Box(_) => false,
            TypeKind::Enum { variants, .. } => variants.iter().all(|v| {
                v.payload
                    .map(|p| self.is_copy_eligible_inner(p, visiting, mapping, allow_opaque_named))
                    .unwrap_or(true)
            }),
            TypeKind::Struct { fields, .. } => fields
                .iter()
                .all(|f| self.is_copy_eligible_inner(*f, visiting, mapping, allow_opaque_named)),
            TypeKind::Tuple { items } => items
                .iter()
                .all(|t| self.is_copy_eligible_inner(*t, visiting, mapping, allow_opaque_named)),
            TypeKind::Apply { base, args } => {
                let resolved_base = mapping
                    .get(&self.resolve_id(*base))
                    .copied()
                    .unwrap_or_else(|| self.resolve_id(*base));
                match self.get_ref(resolved_base) {
                    TypeKind::Struct {
                        type_params,
                        fields,
                        ..
                    } => {
                        if type_params.len() != args.len() {
                            false
                        } else {
                            let mut nested = mapping.clone();
                            for (tp, arg) in type_params.iter().zip(args.iter()) {
                                let rhs = mapping
                                    .get(&self.resolve_id(*arg))
                                    .copied()
                                    .unwrap_or_else(|| self.resolve_id(*arg));
                                nested.insert(self.resolve_id(*tp), rhs);
                            }
                            fields.iter().all(|f| {
                                self.is_copy_eligible_inner(
                                    *f,
                                    visiting,
                                    &nested,
                                    allow_opaque_named,
                                )
                            })
                        }
                    }
                    TypeKind::Enum {
                        type_params,
                        variants,
                        ..
                    } => {
                        if type_params.len() != args.len() {
                            false
                        } else {
                            let mut nested = mapping.clone();
                            for (tp, arg) in type_params.iter().zip(args.iter()) {
                                let rhs = mapping
                                    .get(&self.resolve_id(*arg))
                                    .copied()
                                    .unwrap_or_else(|| self.resolve_id(*arg));
                                nested.insert(self.resolve_id(*tp), rhs);
                            }
                            variants.iter().all(|v| {
                                v.payload
                                    .map(|p| {
                                        self.is_copy_eligible_inner(
                                            p,
                                            visiting,
                                            &nested,
                                            allow_opaque_named,
                                        )
                                    })
                                    .unwrap_or(true)
                            })
                        }
                    }
                    _ => false,
                }
            }
            TypeKind::Function { .. } => true,
            TypeKind::Var(v) => {
                if let Some(b) = v.binding {
                    self.is_copy_eligible_inner(b, visiting, mapping, allow_opaque_named)
                } else {
                    v.copy_cap
                }
            }
            TypeKind::Named(name) => {
                if allow_opaque_named {
                    true
                } else {
                    BackendScalarType::from_name(name.as_str()).is_some()
                }
            }
        };
        visiting.remove(&resolved);
        result
    }

    pub fn same_type(&self, a: TypeId, b: TypeId) -> bool {
        let mut seen = BTreeSet::new();
        self.same_type_inner(self.resolve_id(a), self.resolve_id(b), &mut seen)
    }

    fn same_type_inner(&self, a: TypeId, b: TypeId, seen: &mut BTreeSet<(TypeId, TypeId)>) -> bool {
        let ra = self.resolve_id(a);
        let rb = self.resolve_id(b);
        if ra == rb {
            return true;
        }
        let key = if ra <= rb { (ra, rb) } else { (rb, ra) };
        if !seen.insert(key) {
            return true;
        }
        let result = match (self.get_ref(ra), self.get_ref(rb)) {
            (TypeKind::Unit, TypeKind::Unit)
            | (TypeKind::I32, TypeKind::I32)
            | (TypeKind::U8, TypeKind::U8)
            | (TypeKind::F32, TypeKind::F32)
            | (TypeKind::Bool, TypeKind::Bool)
            | (TypeKind::Char, TypeKind::Char)
            | (TypeKind::Str, TypeKind::Str)
            | (TypeKind::Never, TypeKind::Never) => true,
            (TypeKind::Named(na), TypeKind::Named(nb)) => na == nb,
            (TypeKind::Box(ia), TypeKind::Box(ib)) => self.same_type_inner(*ia, *ib, seen),
            (TypeKind::Reference(ia, ma), TypeKind::Reference(ib, mb)) => {
                ma == mb && self.same_type_inner(*ia, *ib, seen)
            }
            (TypeKind::Tuple { items: ia }, TypeKind::Tuple { items: ib }) => {
                ia.len() == ib.len()
                    && ia
                        .iter()
                        .zip(ib.iter())
                        .all(|(ta, tb)| self.same_type_inner(*ta, *tb, seen))
            }
            (
                TypeKind::Function {
                    type_params: tpa,
                    params: pa,
                    result: ra,
                    effect: ea,
                },
                TypeKind::Function {
                    type_params: tpb,
                    params: pb,
                    result: rb,
                    effect: eb,
                },
            ) => {
                ea == eb
                    && tpa.len() == tpb.len()
                    && pa.len() == pb.len()
                    && tpa
                        .iter()
                        .zip(tpb.iter())
                        .all(|(ta, tb)| self.same_type_inner(*ta, *tb, seen))
                    && pa
                        .iter()
                        .zip(pb.iter())
                        .all(|(ta, tb)| self.same_type_inner(*ta, *tb, seen))
                    && self.same_type_inner(*ra, *rb, seen)
            }
            (
                TypeKind::Struct {
                    name: na,
                    type_params: tpa,
                    fields: fa,
                    field_names: fna,
                    ..
                },
                TypeKind::Struct {
                    name: nb,
                    type_params: tpb,
                    fields: fb,
                    field_names: fnb,
                    ..
                },
            ) => {
                na == nb
                    && fna == fnb
                    && tpa.len() == tpb.len()
                    && fa.len() == fb.len()
                    && tpa
                        .iter()
                        .zip(tpb.iter())
                        .all(|(ta, tb)| self.same_type_inner(*ta, *tb, seen))
                    && fa
                        .iter()
                        .zip(fb.iter())
                        .all(|(ta, tb)| self.same_type_inner(*ta, *tb, seen))
            }
            (
                TypeKind::Enum {
                    name: na,
                    type_params: tpa,
                    variants: va,
                    ..
                },
                TypeKind::Enum {
                    name: nb,
                    type_params: tpb,
                    variants: vb,
                    ..
                },
            ) => {
                na == nb
                    && tpa.len() == tpb.len()
                    && va.len() == vb.len()
                    && tpa
                        .iter()
                        .zip(tpb.iter())
                        .all(|(ta, tb)| self.same_type_inner(*ta, *tb, seen))
                    && va.iter().zip(vb.iter()).all(|(a, b)| {
                        a.name == b.name
                            && match (a.payload, b.payload) {
                                (Some(pa), Some(pb)) => self.same_type_inner(pa, pb, seen),
                                (None, None) => true,
                                _ => false,
                            }
                    })
            }
            (TypeKind::Apply { base: ba, args: aa }, TypeKind::Apply { base: bb, args: ab }) => {
                let bases_match = if self.nominal_apply_base(*ba).is_some()
                    || self.nominal_apply_base(*bb).is_some()
                {
                    self.nominal_apply_bases_match(*ba, *bb)
                } else {
                    self.same_type_inner(*ba, *bb, seen)
                };
                aa.len() == ab.len()
                    && bases_match
                    && aa
                        .iter()
                        .zip(ab.iter())
                        .all(|(ta, tb)| self.same_type_inner(*ta, *tb, seen))
            }
            (TypeKind::Var(va), TypeKind::Var(vb)) => match (va.binding, vb.binding) {
                (Some(ba), Some(bb)) => self.same_type_inner(ba, bb, seen),
                (None, None) => va.label == vb.label,
                (Some(ba), None) => self.same_type_inner(ba, rb, seen),
                (None, Some(bb)) => self.same_type_inner(ra, bb, seen),
            },
            (TypeKind::Var(va), _) => va
                .binding
                .map(|ba| self.same_type_inner(ba, rb, seen))
                .unwrap_or(false),
            (_, TypeKind::Var(vb)) => vb
                .binding
                .map(|bb| self.same_type_inner(ra, bb, seen))
                .unwrap_or(false),
            _ => false,
        };
        seen.remove(&key);
        result
    }

    pub fn get_ref(&self, id: TypeId) -> &TypeKind {
        &self.arena[id.0]
    }

    pub fn get(&self, id: TypeId) -> TypeKind {
        let resolved = self.resolve_id(id);
        match &self.arena[resolved.0] {
            TypeKind::Var(tv) => TypeKind::Var(tv.clone()),
            other => other.clone(),
        }
    }

    pub fn unify(&mut self, a: TypeId, b: TypeId) -> Result<TypeId, UnifyError> {
        // recursion guard to avoid native stack overflow in pathological cases
        let depth = GLOBAL_UNIFY_DEPTH.fetch_add(1, Ordering::SeqCst) + 1;
        let _guard = UnifyDepthGuard;
        if depth > 5000 {
            return Err(UnifyError::Mismatch);
        }

        let ra = self.resolve_id(a);
        let rb = self.resolve_id(b);
        if ra != a || rb != b {
            return self.unify(ra, rb);
        }
        if crate::log::is_verbose() {
            type_log!("unify: {:?} with {:?}", self.get(ra), self.get(rb));
        }
        let ra = self.resolve(ra);
        let rb = self.resolve(rb);
        if ra != a || rb != b {
            return self.unify(ra, rb);
        }
        if ra == rb {
            return Ok(ra);
        }
        if self.apply_arity_mismatch(a) || self.apply_arity_mismatch(b) {
            return Err(UnifyError::Mismatch);
        }
        let ak = self.arena[ra.0].clone();
        let bk = self.arena[rb.0].clone();

        match (ak, bk) {
            (TypeKind::Var(_), TypeKind::Never) => Ok(a),
            (TypeKind::Never, TypeKind::Var(_)) => Ok(b),
            (TypeKind::Var(va), TypeKind::Var(vb)) => {
                if let (Some(la), Some(lb)) = (&va.label, &vb.label) {
                    if la != lb && la != "Self" && lb != "Self" {
                        return Err(UnifyError::Mismatch);
                    }
                }
                match (va.label.is_some(), vb.label.is_some()) {
                    (true, false) => {
                        self.bind_var(b, a);
                        Ok(a)
                    }
                    (false, true) => {
                        self.bind_var(a, b);
                        Ok(b)
                    }
                    _ => {
                        self.bind_var(b, a);
                        Ok(a)
                    }
                }
            }
            (TypeKind::Var(va), other) => {
                if self.occurs_in(ra, rb, &mut BTreeSet::new()) {
                    return Err(UnifyError::Mismatch);
                }
                if let Some(label) = &va.label {
                    if !label_matches(label, &other) {
                        return Err(UnifyError::Mismatch);
                    }
                }
                let other_cloned = other.clone();
                self.bind_var_value(ra, &other_cloned);
                Ok(rb)
            }
            (other, TypeKind::Var(vb)) => {
                if self.occurs_in(rb, ra, &mut BTreeSet::new()) {
                    return Err(UnifyError::Mismatch);
                }
                if let Some(label) = &vb.label {
                    if !label_matches(label, &other) {
                        return Err(UnifyError::Mismatch);
                    }
                }
                let other_cloned = other.clone();
                self.bind_var_value(rb, &other_cloned);
                Ok(ra)
            }
            (TypeKind::Unit, TypeKind::Unit) => Ok(self.unit),
            (TypeKind::I32, TypeKind::I32) => Ok(self.i32_ty),
            (TypeKind::U8, TypeKind::U8) => Ok(self.u8_ty),
            (TypeKind::F32, TypeKind::F32) => Ok(self.f32_ty),
            (TypeKind::Bool, TypeKind::Bool) => Ok(self.bool_ty),
            (TypeKind::Char, TypeKind::Char) => Ok(self.char_ty),
            (TypeKind::Str, TypeKind::Str) => Ok(self.str_ty),
            (TypeKind::Never, _) => Ok(b),
            (_, TypeKind::Never) => Ok(a),
            (TypeKind::Named(na), TypeKind::Named(nb)) => {
                if na == nb {
                    Ok(a)
                } else {
                    Err(UnifyError::Mismatch)
                }
            }
            (
                TypeKind::Enum {
                    name: na,
                    type_params: _,
                    variants: va,
                },
                TypeKind::Enum {
                    name: nb,
                    type_params: _,
                    variants: vb,
                },
            ) => {
                if na != nb || va.len() != vb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (a_var, b_var) in va.iter().zip(vb.iter()) {
                    if a_var.name != b_var.name {
                        return Err(UnifyError::Mismatch);
                    }
                    if let (Some(pa), Some(pb)) = (a_var.payload, b_var.payload) {
                        if let Err(e) = self.unify(pa, pb) {
                            if crate::log::is_verbose() {
                                type_log!("unify: variant {} payload mismatch", a_var.name);
                            }
                            return Err(e);
                        }
                    } else if a_var.payload.is_some() || b_var.payload.is_some() {
                        if crate::log::is_verbose() {
                            type_log!("unify: variant {} payload presence mismatch", a_var.name);
                        }
                        return Err(UnifyError::Mismatch);
                    }
                }
                Ok(a)
            }
            (
                TypeKind::Struct {
                    name: na,
                    fields: fa,
                    type_params: _,
                    field_names: _,
                },
                TypeKind::Struct {
                    name: nb,
                    fields: fb,
                    type_params: _,
                    field_names: _,
                },
            ) => {
                if na != nb || fa.len() != fb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (ta, tb) in fa.iter().zip(fb.iter()) {
                    self.unify(*ta, *tb)?;
                }
                Ok(a)
            }
            (TypeKind::Tuple { items: ta }, TypeKind::Tuple { items: tb }) => {
                if ta.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (xa, xb) in ta.iter().zip(tb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                Ok(a)
            }
            (
                TypeKind::Function {
                    type_params: ta,
                    params: pa,
                    result: ra,
                    effect: ea,
                },
                TypeKind::Function {
                    type_params: tb,
                    params: pb,
                    result: rb,
                    effect: eb,
                },
            ) => {
                if ea != eb || pa.len() != pb.len() || ta.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                for (xa, xb) in ta.iter().zip(tb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                for (xa, xb) in pa.iter().zip(pb.iter()) {
                    self.unify(*xa, *xb)?;
                }
                self.unify(ra, rb)?;
                Ok(self.function(ta.clone(), pa.clone(), ra, ea))
            }
            (TypeKind::Named(na), TypeKind::Enum { name: nb, .. })
            | (TypeKind::Enum { name: na, .. }, TypeKind::Named(nb)) => {
                if na == nb {
                    Ok(a)
                } else {
                    Err(UnifyError::Mismatch)
                }
            }
            (TypeKind::Named(na), TypeKind::Struct { name: nb, .. })
            | (TypeKind::Struct { name: na, .. }, TypeKind::Named(nb)) => {
                if na == nb {
                    Ok(a)
                } else {
                    Err(UnifyError::Mismatch)
                }
            }
            (TypeKind::Box(inner_a), TypeKind::Box(inner_b)) => {
                self.unify(inner_a, inner_b)?;
                Ok(a)
            }
            (TypeKind::Reference(inner_a, mut_a), TypeKind::Reference(inner_b, mut_b)) => {
                if mut_a != mut_b {
                    return Err(UnifyError::Mismatch);
                }
                self.unify(inner_a, inner_b)?;
                Ok(a)
            }
            (TypeKind::Apply { base: ba, args: aa }, TypeKind::Apply { base: bb, args: ab }) => {
                if aa.len() != ab.len() {
                    return Err(UnifyError::Mismatch);
                }
                self.unify_apply_bases(ba, bb)?;
                for (xa, xb) in aa.iter().zip(ab.iter()) {
                    self.unify(*xa, *xb)?;
                }
                Ok(a)
            }
            (
                TypeKind::Enum {
                    name: na,
                    type_params: ta,
                    ..
                },
                TypeKind::Apply { base: bb, args: ab },
            ) => {
                if ta.len() != ab.len() {
                    return Err(UnifyError::Mismatch);
                }
                let resolved_base = self.resolve_id(bb); // Use resolve_id for simple lookup
                match &self.arena[resolved_base.0] {
                    TypeKind::Enum { name: nb, .. } => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    TypeKind::Named(nb) => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    _ => return Err(UnifyError::Mismatch),
                }
                if !self.is_nominal_definition_id(ra, NominalApplyKind::Enum, &na) {
                    for (xa, xb) in ta.iter().zip(ab.iter()) {
                        self.unify(*xa, *xb)?;
                    }
                }
                Ok(a)
            }
            (
                TypeKind::Apply { base: ba, args: aa },
                TypeKind::Enum {
                    name: nb,
                    type_params: tb,
                    ..
                },
            ) => {
                if aa.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                let resolved_base = self.resolve_id(ba);
                match &self.arena[resolved_base.0] {
                    TypeKind::Enum { name: na, .. } => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    TypeKind::Named(na) => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    _ => return Err(UnifyError::Mismatch),
                }
                if !self.is_nominal_definition_id(rb, NominalApplyKind::Enum, &nb) {
                    for (xa, xb) in aa.iter().zip(tb.iter()) {
                        self.unify(*xa, *xb)?;
                    }
                }
                Ok(a)
            }
            (
                TypeKind::Struct {
                    name: na,
                    type_params: ta,
                    ..
                },
                TypeKind::Apply { base: bb, args: ab },
            ) => {
                if ta.len() != ab.len() {
                    return Err(UnifyError::Mismatch);
                }
                let resolved_base = self.resolve_id(bb);
                match &self.arena[resolved_base.0] {
                    TypeKind::Struct { name: nb, .. } => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    TypeKind::Named(nb) => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    _ => return Err(UnifyError::Mismatch),
                }
                if !self.is_nominal_definition_id(ra, NominalApplyKind::Struct, &na) {
                    for (xa, xb) in ta.iter().zip(ab.iter()) {
                        self.unify(*xa, *xb)?;
                    }
                }
                Ok(a)
            }
            (
                TypeKind::Apply { base: ba, args: aa },
                TypeKind::Struct {
                    name: nb,
                    type_params: tb,
                    ..
                },
            ) => {
                if aa.len() != tb.len() {
                    return Err(UnifyError::Mismatch);
                }
                let resolved_base = self.resolve_id(ba);
                match &self.arena[resolved_base.0] {
                    TypeKind::Struct { name: na, .. } => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    TypeKind::Named(na) => {
                        if *na != *nb {
                            return Err(UnifyError::Mismatch);
                        }
                    }
                    _ => return Err(UnifyError::Mismatch),
                }
                if !self.is_nominal_definition_id(rb, NominalApplyKind::Struct, &nb) {
                    for (xa, xb) in aa.iter().zip(tb.iter()) {
                        self.unify(*xa, *xb)?;
                    }
                }
                Ok(a)
            }
            _ => Err(UnifyError::Mismatch),
        }
    }

    fn bind_var(&mut self, var: TypeId, target: TypeId) {
        let target = self.resolve_id(target);
        if target == var {
            return;
        }
        self.record_arena_update(var);
        if let TypeKind::Var(tv) = &mut self.arena[var.0] {
            tv.binding = Some(target);
        }
    }

    fn bind_var_value(&mut self, var: TypeId, value: &TypeKind) {
        self.record_arena_update(var);
        self.arena[var.0] = TypeKind::Var(TypeVar {
            label: match value {
                TypeKind::Var(tv) => tv.label.clone(),
                _ => None,
            },
            binding: Some(self.store(value.clone())),
            copy_cap: match value {
                TypeKind::Var(tv) => tv.copy_cap,
                _ => false,
            },
            clone_cap: match value {
                TypeKind::Var(tv) => tv.clone_cap,
                _ => false,
            },
            drop_cap: match value {
                TypeKind::Var(tv) => tv.drop_cap,
                _ => false,
            },
        });
    }

    pub fn apply(&mut self, base: TypeId, args: Vec<TypeId>) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Apply { base, args });
        id
    }

    pub fn reference(&mut self, inner: TypeId, is_mut: bool) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Reference(inner, is_mut));
        id
    }

    pub fn box_ty(&mut self, inner: TypeId) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(TypeKind::Box(inner));
        id
    }

    pub fn substitute(
        &mut self,
        ty: TypeId,
        mapping: &alloc::collections::BTreeMap<TypeId, TypeId>,
    ) -> TypeId {
        let mut seen = BTreeSet::new();
        self.substitute_inner(ty, mapping, &mut seen)
    }

    fn substitute_inner(
        &mut self,
        ty: TypeId,
        mapping: &alloc::collections::BTreeMap<TypeId, TypeId>,
        seen: &mut BTreeSet<TypeId>,
    ) -> TypeId {
        if let Some(target) = mapping.get(&ty) {
            return *target;
        }
        let ty = self.resolve_id(ty);
        if let Some(target) = mapping.get(&ty) {
            return *target;
        }
        if !seen.insert(ty) {
            return ty;
        }
        let kind = self.arena[ty.0].clone();
        match kind {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str
            | TypeKind::Never => ty,
            TypeKind::Named(_) => ty,
            TypeKind::Var(_) => ty,
            TypeKind::Enum {
                name,
                type_params,
                variants,
            } => {
                let mut new_tps = Vec::new();
                let mut changed = false;
                for tp in type_params {
                    let nt = self.substitute_inner(tp, mapping, seen);
                    if nt != tp {
                        changed = true;
                    }
                    new_tps.push(nt);
                }
                let mut new_vars = Vec::new();
                for v in variants {
                    let new_payload = v.payload.map(|p| {
                        let np = self.substitute_inner(p, mapping, seen);
                        if np != p {
                            changed = true;
                        }
                        np
                    });
                    new_vars.push(EnumVariantInfo {
                        name: v.name.clone(),
                        payload: new_payload,
                    });
                }
                if changed {
                    self.store(TypeKind::Enum {
                        name: name.clone(),
                        type_params: new_tps,
                        variants: new_vars,
                    })
                } else {
                    ty
                }
            }
            TypeKind::Struct {
                name,
                type_params,
                fields,
                field_names,
            } => {
                let mut new_tps = Vec::new();
                let mut changed = false;
                for tp in type_params {
                    let nt = self.substitute_inner(tp, mapping, seen);
                    if nt != tp {
                        changed = true;
                    }
                    new_tps.push(nt);
                }
                let mut new_fs = Vec::new();
                for f in fields {
                    let nf = self.substitute_inner(f, mapping, seen);
                    if nf != f {
                        changed = true;
                    }
                    new_fs.push(nf);
                }
                if changed {
                    self.store(TypeKind::Struct {
                        name: name.clone(),
                        type_params: new_tps,
                        fields: new_fs,
                        field_names: field_names.clone(),
                    })
                } else {
                    ty
                }
            }
            TypeKind::Tuple { items } => {
                let mut new_items = Vec::new();
                let mut changed = false;
                for item in items {
                    let ni = self.substitute_inner(item, mapping, seen);
                    if ni != item {
                        changed = true;
                    }
                    new_items.push(ni);
                }
                if changed {
                    self.store(TypeKind::Tuple { items: new_items })
                } else {
                    ty
                }
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } => {
                let mut new_tps = Vec::new();
                let mut changed = false;
                for tp in type_params {
                    let nt = self.substitute_inner(tp, mapping, seen);
                    if nt != tp {
                        changed = true;
                    }
                    new_tps.push(nt);
                }
                let mut new_ps = Vec::new();
                for p in params {
                    let np = self.substitute_inner(p, mapping, seen);
                    if np != p {
                        changed = true;
                    }
                    new_ps.push(np);
                }
                let new_r = self.substitute_inner(result, mapping, seen);
                if new_r != result {
                    changed = true;
                }

                if changed {
                    self.function(new_tps, new_ps, new_r, effect)
                } else {
                    ty
                }
            }
            TypeKind::Apply { base, args } => {
                let mut new_args = Vec::new();
                let mut changed = false;
                for a in args {
                    let na = self.substitute_inner(a, mapping, seen);
                    if na != a {
                        changed = true;
                    }
                    new_args.push(na);
                }
                let new_base = self
                    .nominal_apply_base(base)
                    .map(|(_, _, _, base_id)| base_id)
                    .unwrap_or_else(|| self.substitute_inner(base, mapping, seen));
                if new_base != base {
                    changed = true;
                }

                if changed {
                    self.apply(new_base, new_args)
                } else {
                    ty
                }
            }
            TypeKind::Box(inner) => {
                let ni = self.substitute_inner(inner, mapping, seen);
                if ni != inner {
                    self.box_ty(ni)
                } else {
                    ty
                }
            }
            TypeKind::Reference(inner, is_mut) => {
                let ni = self.substitute_inner(inner, mapping, seen);
                if ni != inner {
                    self.reference(ni, is_mut)
                } else {
                    ty
                }
            }
        }
    }

    pub fn resolve(&mut self, ty: TypeId) -> TypeId {
        match self.get(ty) {
            TypeKind::Named(name) => {
                if let Some(actual) = self.named.get(&name).copied() {
                    if actual == ty {
                        return ty;
                    }
                    return self.resolve(actual);
                }
                ty
            }
            _ => ty,
        }
    }

    pub fn instantiate(
        &mut self,
        ty: TypeId,
    ) -> (
        TypeId,
        Vec<TypeId>,
        alloc::collections::BTreeMap<TypeId, TypeId>,
    ) {
        let ty = self.resolve_id(ty);
        if let TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } = self.get(ty)
        {
            if type_params.is_empty() {
                return (ty, Vec::new(), alloc::collections::BTreeMap::new());
            }
            let mut mapping = alloc::collections::BTreeMap::new();
            let mut fresh_args = Vec::new();
            for tp in &type_params {
                let fresh = self.fresh_var(None);
                if let TypeKind::Var(tv) = self.get(self.resolve_id(*tp)) {
                    self.set_var_capabilities(fresh, tv.copy_cap, tv.clone_cap, tv.drop_cap);
                }
                mapping.insert(*tp, fresh);
                fresh_args.push(fresh);
            }
            let new_params = params
                .iter()
                .map(|p| self.substitute(*p, &mapping))
                .collect();
            let new_result = self.substitute(result, &mapping);
            (
                self.function(Vec::new(), new_params, new_result, effect),
                fresh_args,
                mapping,
            )
        } else {
            (ty, Vec::new(), alloc::collections::BTreeMap::new())
        }
    }

    pub fn type_to_string(&self, ty: TypeId) -> String {
        let mut seen = BTreeSet::new();
        self.type_to_string_inner(ty, &mut seen)
    }

    fn type_to_string_inner(&self, ty: TypeId, seen: &mut BTreeSet<TypeId>) -> String {
        let ty = self.resolve_id(ty);
        if !seen.insert(ty) {
            if crate::log::is_verbose() {
                type_log!("CYCLE DETECTED in type_to_string: {:?}", ty);
            }
            return String::from("cycle");
        }
        let res = match self.get(ty) {
            TypeKind::Unit => String::from("unit"),
            TypeKind::I32 => String::from("i32"),
            TypeKind::U8 => String::from("u8"),
            TypeKind::F32 => String::from("f32"),
            TypeKind::Bool => String::from("bool"),
            TypeKind::Char => String::from("char"),
            TypeKind::Str => String::from("str"),
            TypeKind::Never => String::from("never"),
            TypeKind::Named(name) => name.clone(),
            TypeKind::Enum {
                name, type_params, ..
            } => {
                if type_params.is_empty() {
                    name.clone()
                } else {
                    let mut s = name.clone();
                    s.push('_');
                    for (i, tp) in type_params.iter().enumerate() {
                        if i > 0 {
                            s.push('_');
                        }
                        s.push_str(&self.type_to_string_inner(*tp, seen));
                    }
                    s
                }
            }
            TypeKind::Struct {
                name, type_params, ..
            } => {
                if type_params.is_empty() {
                    name.clone()
                } else {
                    let mut s = name.clone();
                    s.push('_');
                    for (i, tp) in type_params.iter().enumerate() {
                        if i > 0 {
                            s.push('_');
                        }
                        s.push_str(&self.type_to_string_inner(*tp, seen));
                    }
                    s
                }
            }
            TypeKind::Tuple { items } => {
                let mut s = String::from("tuple_");
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&self.type_to_string_inner(*item, seen));
                }
                s
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } => {
                let mut s = String::from("fn");
                if !type_params.is_empty() {
                    s.push_str("_gen_");
                    s.push_str(&type_params.len().to_string());
                }
                s.push_str("__");
                if params.is_empty() {
                    s.push_str("void");
                } else {
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 {
                            s.push('_');
                        }
                        s.push_str(&self.type_to_string_inner(*p, seen));
                    }
                }
                s.push_str("__");
                s.push_str(&self.type_to_string_inner(result, seen));
                match effect {
                    Effect::Pure => s.push_str("__pure"),
                    Effect::Impure => s.push_str("__imp"),
                }
                s
            }
            TypeKind::Var(tv) => match tv.label {
                Some(ref label) => label.clone(),
                None => format!("var_{}", ty.0),
            },
            TypeKind::Apply { base, args } => {
                let mut s = self.type_to_string_inner(base, seen);
                s.push('_');
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        s.push('_');
                    }
                    s.push_str(&self.type_to_string_inner(*arg, seen));
                }
                s
            }
            TypeKind::Box(inner) => {
                let mut s = String::from("box_");
                s.push_str(&self.type_to_string_inner(inner, seen));
                s
            }
            TypeKind::Reference(inner, is_mut) => {
                let mut s = String::from("ref_");
                if is_mut {
                    s.push_str("mut_");
                }
                s.push_str(&self.type_to_string_inner(inner, seen));
                s
            }
        };
        seen.remove(&ty);
        res
    }

    fn store(&mut self, kind: TypeKind) -> TypeId {
        let id = TypeId(self.arena.len());
        self.arena.push(kind);
        id
    }

    fn apply_arity_mismatch(&self, ty: TypeId) -> bool {
        match self.get(ty) {
            TypeKind::Apply { base, args } => match self.get(base) {
                TypeKind::Enum { type_params, .. }
                | TypeKind::Struct { type_params, .. }
                | TypeKind::Function { type_params, .. } => type_params.len() != args.len(),
                _ => false,
            },
            _ => false,
        }
    }

    fn occurs_in(&self, var: TypeId, ty: TypeId, seen: &mut BTreeSet<TypeId>) -> bool {
        let ty = self.resolve_id(ty);
        if ty == var {
            return true;
        }
        if !seen.insert(ty) {
            return false;
        }
        match self.get(ty) {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str
            | TypeKind::Never
            | TypeKind::Named(_) => false,
            TypeKind::Var(tv) => {
                if let Some(b) = tv.binding {
                    self.occurs_in(var, b, seen)
                } else {
                    false
                }
            }
            TypeKind::Enum {
                type_params,
                variants,
                ..
            } => {
                for tp in type_params {
                    if self.occurs_in(var, tp, seen) {
                        return true;
                    }
                }
                for v in variants {
                    if let Some(p) = v.payload {
                        if self.occurs_in(var, p, seen) {
                            return true;
                        }
                    }
                }
                false
            }
            TypeKind::Struct {
                type_params,
                fields,
                ..
            } => {
                for tp in type_params {
                    if self.occurs_in(var, tp, seen) {
                        return true;
                    }
                }
                for f in fields {
                    if self.occurs_in(var, f, seen) {
                        return true;
                    }
                }
                false
            }
            TypeKind::Tuple { items } => {
                for item in items {
                    if self.occurs_in(var, item, seen) {
                        return true;
                    }
                }
                false
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                ..
            } => {
                for tp in type_params {
                    if self.occurs_in(var, tp, seen) {
                        return true;
                    }
                }
                for p in params {
                    if self.occurs_in(var, p, seen) {
                        return true;
                    }
                }
                self.occurs_in(var, result, seen)
            }
            TypeKind::Apply { base, args } => {
                if self.occurs_in(var, base, seen) {
                    return true;
                }
                for a in args {
                    if self.occurs_in(var, a, seen) {
                        return true;
                    }
                }
                false
            }
            TypeKind::Box(inner) => self.occurs_in(var, inner, seen),
            TypeKind::Reference(inner, _) => self.occurs_in(var, inner, seen),
        }
    }
}

struct NominalDefinitionHasher {
    state: u64,
}

impl NominalDefinitionHasher {
    fn new(namespace: &str) -> Self {
        let mut hasher = Self {
            state: 0xcbf29ce484222325,
        };
        hasher.write_str(namespace);
        hasher
    }

    fn write_str(&mut self, value: &str) {
        self.write_usize(value.len());
        for byte in value.as_bytes() {
            self.write_u8(*byte);
        }
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.write_u8(byte);
        }
    }

    fn write_bool(&mut self, value: bool) {
        self.write_u8(u8::from(value));
    }

    fn write_u8(&mut self, value: u8) {
        self.state ^= u64::from(value);
        self.state = self.state.wrapping_mul(0x100000001b3);
    }

    fn finish(self) -> u64 {
        self.state
    }
}

fn stable_text_component(text: &str) -> String {
    format!("{}:{text}", text.len())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnifyError {
    Mismatch,
}

fn label_matches(label: &str, ty: &TypeKind) -> bool {
    match ty {
        TypeKind::Var(tv) => tv.label.as_ref().map(|l| l == label).unwrap_or(true),
        _ => true,
    }
}
