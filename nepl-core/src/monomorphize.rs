#![no_std]
extern crate alloc;
extern crate std;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::*;
use crate::runtime_helpers::{find_runtime_helper_key, RuntimeHelperKind};
use crate::types::{TypeCtx, TypeId, TypeKind};

pub fn monomorphize(ctx: &mut TypeCtx, module: HirModule) -> HirModule {
    monomorphize_internal(ctx, module, true).0
}

fn monomorphize_internal(
    ctx: &mut TypeCtx,
    module: HirModule,
    assert_trait_calls: bool,
) -> (HirModule, Vec<String>) {
    let mut impl_map: BTreeMap<(String, String, TypeId), String> = BTreeMap::new();
    let mut impl_entries: Vec<(String, Vec<TypeId>, String, TypeId, String)> = Vec::new();
    for imp in &module.impls {
        let ty = ctx.resolve_id(imp.target_ty);
        for m in &imp.methods {
            impl_map.insert(
                (imp.trait_name.clone(), m.name.clone(), ty),
                m.func.name.clone(),
            );
            if let Some(base) = &imp.trait_base_name {
                impl_entries.push((
                    base.clone(),
                    imp.trait_args.clone(),
                    m.name.clone(),
                    ty,
                    m.func.name.clone(),
                ));
            }
        }
    }
    let mut mono = Monomorphizer {
        ctx,
        funcs: BTreeMap::new(),
        specialized: BTreeMap::new(),
        worklist: Vec::new(),
        queued: BTreeSet::new(),
        impl_map,
        impl_entries,
    };

    for f in module.functions {
        mono.funcs.insert(f.name.clone(), f);
    }

    // Start with the entry point or all non-generic functions
    let mut initial = Vec::new();
    if let Some(entry) = &module.entry {
        initial.push(entry.clone());
    } else {
        for (name, f) in &mono.funcs {
            if let TypeKind::Function { type_params, .. } = mono.ctx.get(f.func_ty) {
                if crate::log::is_verbose() {
                    std::eprintln!(
                        "monomorphize: checking {}, params.len={}",
                        name,
                        type_params.len()
                    );
                }
                if type_params.is_empty() {
                    initial.push(name.clone());
                }
            }
        }
    }

    // Ensure runtime-required helpers are retained even if not explicitly referenced.
    // Enum/struct/tuple codegen depends on allocator helper availability.
    for kind in [
        RuntimeHelperKind::Alloc,
        RuntimeHelperKind::Dealloc,
        RuntimeHelperKind::Realloc,
    ] {
        if let Some(name) = find_runtime_helper_key(&mono.funcs, kind) {
            if !initial.iter().any(|n| n == &name) {
                initial.push(String::from(name));
            }
        }
    }

    for name in initial {
        if crate::log::is_verbose() {
            std::eprintln!("monomorphize: initial function {}", name);
        }
        mono.request_instantiation(name, Vec::new());
    }

    while let Some((orig_name, args)) = mono.worklist.pop() {
        mono.process_instantiation(orig_name, args);
    }
    loop {
        mono.resolve_remaining_trait_calls();
        if mono.worklist.is_empty() {
            break;
        }
        while let Some((orig_name, args)) = mono.worklist.pop() {
            mono.process_instantiation(orig_name, args);
        }
    }

    let mut unresolved_trait_calls = Vec::new();
    for f in mono.specialized.values() {
        let unresolved = mono.collect_unresolved_trait_calls(f);
        if assert_trait_calls && !unresolved.is_empty() {
            mono.assert_no_trait_calls(f);
        }
        unresolved_trait_calls.extend(unresolved);
    }
    let mut new_functions = Vec::new();
    for (_, f) in mono.specialized {
        new_functions.push(f);
    }

    (
        HirModule {
            functions: new_functions,
            entry: module.entry,
            externs: module.externs,
            string_literals: module.string_literals,
            traits: module.traits,
            impls: module.impls,
        },
        unresolved_trait_calls,
    )
}

struct Monomorphizer<'a> {
    ctx: &'a mut TypeCtx,
    funcs: BTreeMap<String, HirFunction>,
    specialized: BTreeMap<String, HirFunction>,
    worklist: Vec<(String, Vec<TypeId>)>,
    queued: BTreeSet<String>,
    impl_map: BTreeMap<(String, String, TypeId), String>,
    impl_entries: Vec<(String, Vec<TypeId>, String, TypeId, String)>,
}

impl<'a> Monomorphizer<'a> {
    fn type_has_unbound_var(&self, ty: TypeId) -> bool {
        let resolved = self.ctx.resolve_id(ty);
        match self.ctx.get(resolved) {
            TypeKind::Var(tv) => match tv.binding {
                Some(next) => self.type_has_unbound_var(next),
                None => true,
            },
            TypeKind::Tuple { items } => items.iter().any(|item| self.type_has_unbound_var(*item)),
            TypeKind::Struct {
                type_params,
                fields,
                ..
            } => {
                type_params.iter().any(|tp| self.type_has_unbound_var(*tp))
                    || fields.iter().any(|field| self.type_has_unbound_var(*field))
            }
            TypeKind::Enum {
                type_params,
                variants,
                ..
            } => {
                type_params.iter().any(|tp| self.type_has_unbound_var(*tp))
                    || variants
                        .iter()
                        .filter_map(|variant| variant.payload)
                        .any(|payload| self.type_has_unbound_var(payload))
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                ..
            } => {
                type_params.iter().any(|tp| self.type_has_unbound_var(*tp))
                    || params.iter().any(|param| self.type_has_unbound_var(*param))
                    || self.type_has_unbound_var(result)
            }
            TypeKind::Apply { base, args } => {
                self.type_has_unbound_var(base)
                    || args.iter().any(|arg| self.type_has_unbound_var(*arg))
            }
            TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
                self.type_has_unbound_var(inner)
            }
            _ => false,
        }
    }

    fn resolve_user_function_name(&self, name: &str) -> Option<String> {
        if self.funcs.contains_key(name) {
            return Some(String::from(name));
        }
        let mut prefix = String::from(name);
        prefix.push_str("__");
        let mut matched: Option<String> = None;
        for cand in self.funcs.keys() {
            if cand.starts_with(&prefix) {
                if matched.is_some() {
                    return None;
                }
                matched = Some(cand.clone());
            }
        }
        matched
    }

    fn collect_unresolved_trait_calls(&self, func: &HirFunction) -> Vec<String> {
        let mut out = Vec::new();
        let mut stack = Vec::new();
        if let HirBody::Block(block) = &func.body {
            for line in block.lines.iter().rev() {
                stack.push(&line.expr);
            }
        }
        while let Some(expr) = stack.pop() {
            match &expr.kind {
                HirExprKind::Call { callee, args } => {
                    for arg in args.iter().rev() {
                        stack.push(arg);
                    }
                    if let FuncRef::Trait {
                        trait_name,
                        trait_args,
                        method,
                        self_ty,
                    } = callee
                    {
                        let rendered_args = trait_args
                            .iter()
                            .map(|ty| self.ctx.type_to_string(*ty))
                            .collect::<Vec<_>>()
                            .join(", ");
                        out.push(format!(
                            "{} :: {}<{}>::{} [self={}]",
                            func.name,
                            trait_name,
                            rendered_args,
                            method,
                            self.ctx.type_to_string(*self_ty),
                        ));
                    }
                }
                HirExprKind::CallIndirect { callee, args, .. } => {
                    for arg in args.iter().rev() {
                        stack.push(arg);
                    }
                    stack.push(callee);
                }
                HirExprKind::If {
                    cond,
                    then_branch,
                    else_branch,
                } => {
                    stack.push(else_branch);
                    stack.push(then_branch);
                    stack.push(cond);
                }
                HirExprKind::While { cond, body } => {
                    stack.push(body);
                    stack.push(cond);
                }
                HirExprKind::Match { scrutinee, arms } => {
                    for arm in arms.iter().rev() {
                        stack.push(&arm.body);
                    }
                    stack.push(scrutinee);
                }
                HirExprKind::Block(block) => {
                    for line in block.lines.iter().rev() {
                        stack.push(&line.expr);
                    }
                }
                HirExprKind::Let { value, .. }
                | HirExprKind::Set { value, .. }
                | HirExprKind::AddrOf(value)
                | HirExprKind::Deref(value) => stack.push(value),
                HirExprKind::TupleConstruct { items }
                | HirExprKind::Intrinsic { args: items, .. } => {
                    for item in items.iter().rev() {
                        stack.push(item);
                    }
                }
                HirExprKind::EnumConstruct { payload, .. } => {
                    if let Some(payload) = payload {
                        stack.push(payload);
                    }
                }
                HirExprKind::StructConstruct { fields, .. } => {
                    for field in fields.iter().rev() {
                        stack.push(field);
                    }
                }
                HirExprKind::FnValue(_)
                | HirExprKind::Var(_)
                | HirExprKind::Unit
                | HirExprKind::LiteralI32(_)
                | HirExprKind::LiteralF32(_)
                | HirExprKind::LiteralBool(_)
                | HirExprKind::LiteralStr(_)
                | HirExprKind::Drop { .. } => {}
            }
        }
        out
    }

    fn assert_no_trait_calls(&self, func: &HirFunction) {
        let unresolved = self.collect_unresolved_trait_calls(func);
        if let Some(first) = unresolved.first() {
            panic!(
                "internal compiler error: unresolved trait call remained after monomorphize: {}",
                first,
            );
        }
    }

    fn resolve_remaining_trait_calls(&mut self) {
        let names: Vec<String> = self.specialized.keys().cloned().collect();
        for name in names {
            let Some(mut func) = self.specialized.remove(&name) else {
                continue;
            };
            match &mut func.body {
                HirBody::Block(block) => self.resolve_trait_calls_in_block(block),
                HirBody::Wasm(_) | HirBody::LlvmIr(_) => {}
            }
            self.specialized.insert(name, func);
        }
    }

    fn resolve_trait_calls_in_block(&mut self, block: &mut HirBlock) {
        let mut stack = Vec::new();
        for line in block.lines.iter_mut().rev() {
            stack.push(&mut line.expr);
        }
        while let Some(expr) = stack.pop() {
            match &mut expr.kind {
                HirExprKind::Call { callee, args } => {
                    if let FuncRef::Trait {
                        trait_name,
                        trait_args,
                        method,
                        self_ty,
                    } = callee
                    {
                        for trait_arg in trait_args.iter_mut() {
                            *trait_arg = self.ctx.resolve_id(*trait_arg);
                        }
                        let resolved = self.ctx.resolve_id(*self_ty);
                        let dispatch_self_ty = match self.ctx.get(resolved) {
                            TypeKind::Var(_) => args
                                .first()
                                .map(|arg| self.ctx.resolve_id(arg.ty))
                                .unwrap_or(resolved),
                            _ => resolved,
                        };
                        *self_ty = dispatch_self_ty;
                        if let Some(name) = self.resolve_trait_impl_name(
                            trait_name.as_str(),
                            trait_args,
                            method.as_str(),
                            dispatch_self_ty,
                        ) {
                            *callee = FuncRef::User(
                                self.request_instantiation(name, trait_args.clone()),
                                Vec::new(),
                            );
                        }
                    }
                    for arg in args.iter_mut().rev() {
                        stack.push(arg);
                    }
                }
                HirExprKind::CallIndirect { callee, args, .. } => {
                    for arg in args.iter_mut().rev() {
                        stack.push(arg);
                    }
                    stack.push(callee.as_mut());
                }
                HirExprKind::If {
                    cond,
                    then_branch,
                    else_branch,
                } => {
                    stack.push(else_branch.as_mut());
                    stack.push(then_branch.as_mut());
                    stack.push(cond.as_mut());
                }
                HirExprKind::While { cond, body } => {
                    stack.push(body.as_mut());
                    stack.push(cond.as_mut());
                }
                HirExprKind::Match { scrutinee, arms } => {
                    for arm in arms.iter_mut().rev() {
                        stack.push(&mut arm.body);
                    }
                    stack.push(scrutinee.as_mut());
                }
                HirExprKind::Block(block) => {
                    for line in block.lines.iter_mut().rev() {
                        stack.push(&mut line.expr);
                    }
                }
                HirExprKind::Let { value, .. }
                | HirExprKind::Set { value, .. }
                | HirExprKind::AddrOf(value)
                | HirExprKind::Deref(value) => stack.push(value.as_mut()),
                HirExprKind::TupleConstruct { items }
                | HirExprKind::Intrinsic { args: items, .. } => {
                    for item in items.iter_mut().rev() {
                        stack.push(item);
                    }
                }
                HirExprKind::EnumConstruct { payload, .. } => {
                    if let Some(payload) = payload {
                        stack.push(payload.as_mut());
                    }
                }
                HirExprKind::StructConstruct { fields, .. } => {
                    for field in fields.iter_mut().rev() {
                        stack.push(field);
                    }
                }
                HirExprKind::FnValue(_)
                | HirExprKind::Var(_)
                | HirExprKind::Unit
                | HirExprKind::LiteralI32(_)
                | HirExprKind::LiteralF32(_)
                | HirExprKind::LiteralBool(_)
                | HirExprKind::LiteralStr(_)
                | HirExprKind::Drop { .. } => {}
            }
        }
    }

    fn resolve_trait_impl_name(
        &self,
        trait_name: &str,
        trait_args: &[TypeId],
        method: &str,
        resolved_self_ty: TypeId,
    ) -> Option<String> {
        let key = (
            String::from(trait_name),
            String::from(method),
            resolved_self_ty,
        );
        if let Some(name) = self.impl_map.get(&key) {
            return Some(name.clone());
        }
        for ((tr, meth, target_ty), func_name) in self.impl_map.iter() {
            if tr != trait_name || meth != method {
                continue;
            }
            if self.ctx.same_type(resolved_self_ty, *target_ty) {
                return Some(func_name.clone());
            }
        }
        for (base, impl_trait_args, meth, target_ty, func_name) in self.impl_entries.iter() {
            if base != trait_name || meth != method {
                continue;
            }
            if !self.ctx.same_type(resolved_self_ty, *target_ty) {
                continue;
            }
            if impl_trait_args.len() != trait_args.len() {
                continue;
            }
            let mut matched = true;
            for (impl_arg, call_arg) in impl_trait_args.iter().zip(trait_args.iter()) {
                let impl_arg = self.ctx.resolve_id(*impl_arg);
                let call_arg = self.ctx.resolve_id(*call_arg);
                if !self.ctx.type_pattern_matches(impl_arg, call_arg)
                    && !self.ctx.type_pattern_matches(call_arg, impl_arg)
                {
                    matched = false;
                    break;
                }
            }
            if matched {
                return Some(func_name.clone());
            }
        }
        None
    }

    fn request_instantiation(&mut self, name: String, args: Vec<TypeId>) -> String {
        let mut resolved_args = Vec::new();
        for arg in &args {
            resolved_args.push(self.ctx.resolve_id(*arg));
        }
        let args = resolved_args;
        let mangled = if args.is_empty() {
            name.clone()
        } else {
            let mut s = name.clone();
            s.push('_');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&self.ctx.type_to_string(*arg));
            }
            s
        };

        if crate::log::is_verbose() {
            let rendered_args = args
                .iter()
                .map(|arg| self.ctx.type_to_string(*arg))
                .collect::<Vec<_>>()
                .join(", ");
            std::eprintln!(
                "monomorphize: request '{}' [{}] -> '{}'",
                name,
                rendered_args,
                mangled
            );
        }

        if !self.specialized.contains_key(&mangled) {
            if self.queued.insert(mangled.clone()) {
                self.worklist.push((name, args));
            }
        }
        mangled
    }

    fn take_function_for_instantiation(
        &mut self,
        orig_name: &str,
        can_move_original: bool,
    ) -> Option<HirFunction> {
        if can_move_original {
            if let Some(func) = self.funcs.remove(orig_name) {
                return Some(func);
            }
        }
        self.funcs.get(orig_name).cloned()
    }

    fn queue_concrete_callees(&mut self, func: &mut HirFunction) {
        let mut local_names: BTreeSet<String> = BTreeSet::new();
        for p in &func.params {
            local_names.insert(p.name.clone());
        }
        if let HirBody::Block(block) = &mut func.body {
            collect_local_names_in_block(block, &mut local_names);
            self.queue_concrete_callees_in_block(block, &local_names);
        }
    }

    fn queue_concrete_callees_in_block(
        &mut self,
        block: &mut HirBlock,
        local_names: &BTreeSet<String>,
    ) {
        let mut stack = Vec::new();
        for line in block.lines.iter_mut().rev() {
            stack.push(&mut line.expr);
        }
        while let Some(expr) = stack.pop() {
            match &mut expr.kind {
                HirExprKind::Call { callee, args } => {
                    for arg in args.iter_mut().rev() {
                        stack.push(arg);
                    }
                    match callee {
                        FuncRef::User(name, type_args) => {
                            for arg in type_args.iter_mut() {
                                *arg = self.ctx.resolve_id(*arg);
                            }
                            let inst = if let Some(found) =
                                self.resolve_user_function_name(name.as_str())
                            {
                                self.request_instantiation(found, type_args.clone())
                            } else {
                                self.request_instantiation(name.clone(), type_args.clone())
                            };
                            *name = inst;
                            type_args.clear();
                        }
                        FuncRef::Trait {
                            trait_args,
                            self_ty,
                            ..
                        } => {
                            for trait_arg in trait_args.iter_mut() {
                                *trait_arg = self.ctx.resolve_id(*trait_arg);
                            }
                            *self_ty = self.ctx.resolve_id(*self_ty);
                        }
                        FuncRef::Builtin(_) => {}
                    }
                }
                HirExprKind::CallIndirect { callee, args, .. } => {
                    for arg in args.iter_mut().rev() {
                        stack.push(arg);
                    }
                    stack.push(callee.as_mut());
                }
                HirExprKind::If {
                    cond,
                    then_branch,
                    else_branch,
                } => {
                    stack.push(else_branch.as_mut());
                    stack.push(then_branch.as_mut());
                    stack.push(cond.as_mut());
                }
                HirExprKind::While { cond, body } => {
                    stack.push(body.as_mut());
                    stack.push(cond.as_mut());
                }
                HirExprKind::Match { scrutinee, arms } => {
                    for arm in arms.iter_mut().rev() {
                        stack.push(&mut arm.body);
                    }
                    stack.push(scrutinee.as_mut());
                }
                HirExprKind::EnumConstruct { payload, .. } => {
                    if let Some(payload) = payload {
                        stack.push(payload.as_mut());
                    }
                }
                HirExprKind::StructConstruct { fields, .. } => {
                    for field in fields.iter_mut().rev() {
                        stack.push(field);
                    }
                }
                HirExprKind::TupleConstruct { items } => {
                    for item in items.iter_mut().rev() {
                        stack.push(item);
                    }
                }
                HirExprKind::Block(block) => {
                    for line in block.lines.iter_mut().rev() {
                        stack.push(&mut line.expr);
                    }
                }
                HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
                    stack.push(value.as_mut());
                }
                HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
                    stack.push(inner.as_mut());
                }
                HirExprKind::Intrinsic { args, .. } => {
                    for arg in args.iter_mut().rev() {
                        stack.push(arg);
                    }
                }
                HirExprKind::Var(name) => {
                    if local_names.contains(name) {
                        continue;
                    }
                    if let Some(found) = self.resolve_user_function_name(name.as_str()) {
                        *name = self.request_instantiation(found, Vec::new());
                    }
                }
                HirExprKind::FnValue(name) => {
                    if let Some(found) = self.resolve_user_function_name(name.as_str()) {
                        *name = self.request_instantiation(found, Vec::new());
                    }
                }
                HirExprKind::Unit
                | HirExprKind::LiteralI32(_)
                | HirExprKind::LiteralF32(_)
                | HirExprKind::LiteralBool(_)
                | HirExprKind::LiteralStr(_)
                | HirExprKind::Drop { .. } => {}
            }
        }
    }

    fn process_instantiation(&mut self, orig_name: String, args: Vec<TypeId>) {
        let mut resolved_args = Vec::new();
        for arg in &args {
            resolved_args.push(self.ctx.resolve_id(*arg));
        }
        let args = resolved_args;
        let mangled = if args.is_empty() {
            orig_name.clone()
        } else {
            let mut s = orig_name.clone();
            s.push('_');
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    s.push('_');
                }
                s.push_str(&self.ctx.type_to_string(*arg));
            }
            s
        };

        if self.specialized.contains_key(&mangled) {
            return;
        }

        if crate::log::is_verbose() && orig_name.contains("partition") {
            std::eprintln!(
                "monomorphize: process '{}' -> '{}' args={}",
                orig_name,
                mangled,
                args.iter()
                    .map(|arg| self.ctx.type_to_string(*arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        let mut f = match self.take_function_for_instantiation(&orig_name, args.is_empty()) {
            Some(f) => f,
            None => {
                if crate::log::is_verbose() {
                    let related = self
                        .funcs
                        .keys()
                        .filter(|cand| {
                            cand.contains("partition") || cand.contains(orig_name.as_str())
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    std::eprintln!(
                        "monomorphize: missing original function '{}' candidates={:?}",
                        orig_name,
                        related
                    );
                }
                return;
            }
        };

        let mut mapping = BTreeMap::new();
        if let TypeKind::Function { type_params, .. } = self.ctx.get(f.func_ty) {
            for (tp, arg) in type_params.iter().zip(args.iter()) {
                mapping.insert(self.ctx.resolve_id(*tp), self.ctx.resolve_id(*arg));
            }
        }

        f.name = mangled.clone();
        if mapping.is_empty() {
            self.queue_concrete_callees(&mut f);
        } else {
            let mut local_names: BTreeSet<String> = BTreeSet::new();
            for p in &f.params {
                local_names.insert(p.name.clone());
            }
            if let HirBody::Block(b) = &f.body {
                collect_local_names_in_block(b, &mut local_names);
            }

            f.result = self.ctx.substitute(f.result, &mapping);
            for p in &mut f.params {
                p.ty = self.ctx.substitute(p.ty, &mapping);
            }
            f.func_ty = match self.ctx.get(f.func_ty) {
                TypeKind::Function { effect, .. } => {
                    let params = f.params.iter().map(|p| p.ty).collect::<Vec<_>>();
                    self.ctx.function(Vec::new(), params, f.result, effect)
                }
                _ => self.ctx.substitute(f.func_ty, &mapping),
            };

            match &mut f.body {
                HirBody::Block(b) => self.substitute_block(b, &mapping, &local_names),
                HirBody::Wasm(_) => {} // Wasm blocks don't hold TypeIds usually
                HirBody::LlvmIr(_) => {} // LLVM IR blocks don't hold TypeIds usually
            }
        }

        if let HirBody::Block(b) = &f.body {
            let block_ty = self.ctx.resolve_id(b.ty);
            if self.type_has_unbound_var(f.result) && !self.type_has_unbound_var(block_ty) {
                f.result = block_ty;
                if let TypeKind::Function { effect, .. } = self.ctx.get(f.func_ty) {
                    let params = f.params.iter().map(|p| p.ty).collect::<Vec<_>>();
                    f.func_ty = self.ctx.function(Vec::new(), params, f.result, effect);
                }
            }
        }

        if crate::log::is_verbose() && f.name.contains("partition") {
            std::eprintln!(
                "monomorphize: insert specialized '{}' result={} block_ty={} func_ty={}",
                mangled,
                self.ctx.type_to_string(f.result),
                match &f.body {
                    HirBody::Block(b) => self.ctx.type_to_string(b.ty),
                    _ => String::from("<non-block>"),
                },
                self.ctx.type_to_string(f.func_ty)
            );
        }
        self.specialized.insert(mangled, f);
    }

    fn substitute_block(
        &mut self,
        b: &mut HirBlock,
        mapping: &BTreeMap<TypeId, TypeId>,
        local_names: &BTreeSet<String>,
    ) {
        b.ty = self.ctx.substitute(b.ty, mapping);
        for line in &mut b.lines {
            self.substitute_expr(&mut line.expr, mapping, local_names);
        }
    }

    fn substitute_expr(
        &mut self,
        expr: &mut HirExpr,
        mapping: &BTreeMap<TypeId, TypeId>,
        local_names: &BTreeSet<String>,
    ) {
        expr.ty = self.ctx.substitute(expr.ty, mapping);
        match &mut expr.kind {
            HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_) => {}
            HirExprKind::Var(name) => {
                if local_names.contains(name) {
                    return;
                }
                if let Some(found) = self.resolve_user_function_name(name.as_str()) {
                    *name = self.request_instantiation(found, Vec::new());
                }
            }
            HirExprKind::FnValue(name) => {
                if let Some(found) = self.resolve_user_function_name(name.as_str()) {
                    *name = self.request_instantiation(found, Vec::new());
                }
            }
            HirExprKind::Call { callee, args } => {
                for arg in args.iter_mut() {
                    self.substitute_expr(arg, mapping, local_names);
                }
                match callee {
                    FuncRef::User(name, type_args) => {
                        for arg in type_args.iter_mut() {
                            *arg = self.ctx.substitute(*arg, mapping);
                        }
                        // Request instantiation of the callee with concrete types
                        if let Some(found) = self.resolve_user_function_name(name.as_str()) {
                            *name = self.request_instantiation(found, type_args.clone());
                        } else {
                            *name = self.request_instantiation(name.clone(), type_args.clone());
                        }
                        type_args.clear(); // Call site in WASM doesn't need type_args anymore
                    }
                    FuncRef::Trait {
                        trait_name,
                        trait_args,
                        method,
                        self_ty,
                    } => {
                        for trait_arg in trait_args.iter_mut() {
                            *trait_arg = self.ctx.substitute(*trait_arg, mapping);
                            *trait_arg = self.ctx.resolve_id(*trait_arg);
                        }
                        *self_ty = self.ctx.substitute(*self_ty, mapping);
                        let resolved = self.ctx.resolve_id(*self_ty);
                        let dispatch_self_ty = match self.ctx.get(resolved) {
                            TypeKind::Var(_) => args
                                .first()
                                .map(|arg| self.ctx.resolve_id(arg.ty))
                                .unwrap_or(resolved),
                            _ => resolved,
                        };
                        *self_ty = dispatch_self_ty;
                        if let Some(func_name) = self.resolve_trait_impl_name(
                            trait_name.as_str(),
                            trait_args,
                            method.as_str(),
                            dispatch_self_ty,
                        ) {
                            let inst = self.request_instantiation(func_name, trait_args.clone());
                            *callee = FuncRef::User(inst, Vec::new());
                        }
                    }
                    FuncRef::Builtin(_) => {}
                }
            }
            HirExprKind::CallIndirect {
                callee,
                params,
                result,
                args,
            } => {
                self.substitute_expr(callee, mapping, local_names);
                for param in params.iter_mut() {
                    *param = self.ctx.substitute(*param, mapping);
                }
                *result = self.ctx.substitute(*result, mapping);
                for arg in args {
                    self.substitute_expr(arg, mapping, local_names);
                }
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.substitute_expr(cond, mapping, local_names);
                self.substitute_expr(then_branch, mapping, local_names);
                self.substitute_expr(else_branch, mapping, local_names);
            }
            HirExprKind::While { cond, body } => {
                self.substitute_expr(cond, mapping, local_names);
                self.substitute_expr(body, mapping, local_names);
            }
            HirExprKind::Match { scrutinee, arms } => {
                self.substitute_expr(scrutinee, mapping, local_names);
                for arm in arms {
                    self.substitute_expr(&mut arm.body, mapping, local_names);
                }
            }
            HirExprKind::EnumConstruct {
                variant: _,
                type_args,
                payload,
                ..
            } => {
                for arg in type_args.iter_mut() {
                    *arg = self.ctx.substitute(*arg, mapping);
                }
                if let Some(p) = payload {
                    self.substitute_expr(p, mapping, local_names);
                }
            }
            HirExprKind::StructConstruct {
                type_args, fields, ..
            } => {
                for arg in type_args.iter_mut() {
                    *arg = self.ctx.substitute(*arg, mapping);
                }
                for f in fields {
                    self.substitute_expr(f, mapping, local_names);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items {
                    self.substitute_expr(item, mapping, local_names);
                }
            }
            HirExprKind::Block(b) => self.substitute_block(b, mapping, local_names),
            HirExprKind::Let { value, .. } => self.substitute_expr(value, mapping, local_names),
            HirExprKind::Set { value, .. } => self.substitute_expr(value, mapping, local_names),
            HirExprKind::AddrOf(inner) => self.substitute_expr(inner, mapping, local_names),
            HirExprKind::Deref(inner) => self.substitute_expr(inner, mapping, local_names),
            HirExprKind::Drop { .. } => {}
            HirExprKind::Intrinsic {
                type_args,
                args,
                name: _,
            } => {
                for arg in type_args.iter_mut() {
                    *arg = self.ctx.substitute(*arg, mapping);
                }
                for arg in args {
                    self.substitute_expr(arg, mapping, local_names);
                }
            }
        }
    }
}

fn collect_local_names_in_block(block: &HirBlock, out: &mut BTreeSet<String>) {
    for line in &block.lines {
        collect_local_names_in_expr(&line.expr, out);
    }
}

fn collect_local_names_in_expr(expr: &HirExpr, out: &mut BTreeSet<String>) {
    match &expr.kind {
        HirExprKind::Let { name, value, .. } => {
            out.insert(name.clone());
            collect_local_names_in_expr(value, out);
        }
        HirExprKind::Set { value, .. } => {
            collect_local_names_in_expr(value, out);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                collect_local_names_in_expr(arg, out);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            collect_local_names_in_expr(callee, out);
            for arg in args {
                collect_local_names_in_expr(arg, out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            collect_local_names_in_expr(cond, out);
            collect_local_names_in_expr(then_branch, out);
            collect_local_names_in_expr(else_branch, out);
        }
        HirExprKind::While { cond, body } => {
            collect_local_names_in_expr(cond, out);
            collect_local_names_in_expr(body, out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            collect_local_names_in_expr(scrutinee, out);
            for arm in arms {
                if let Some(bind) = &arm.bind_local {
                    out.insert(bind.clone());
                }
                collect_local_names_in_expr(&arm.body, out);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                collect_local_names_in_expr(p, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                collect_local_names_in_expr(field, out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                collect_local_names_in_expr(item, out);
            }
        }
        HirExprKind::Block(b) => {
            collect_local_names_in_block(b, out);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                collect_local_names_in_expr(arg, out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            collect_local_names_in_expr(inner, out);
        }
        HirExprKind::Unit
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Var(_)
        | HirExprKind::FnValue(_)
        | HirExprKind::Drop { .. } => {}
    }
}
