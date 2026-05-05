extern crate alloc;

use alloc::format;
use alloc::string::String;
use core::fmt::Write;

use super::model::{
    AggregateKind, EffectOp, Place, PlaceProjection, PlaceRoot, RawBodyKind, RawMemoryOp,
    ResourceCallTarget, ResourceConditionFact, ResourceMatchPattern, ResourceModule, ResourceOp,
    ResourceTerminator,
};

impl ResourceModule {
    pub fn dump_text(&self) -> String {
        let mut out = String::new();
        let entry = self.entry.as_deref().unwrap_or("<none>");
        let _ = writeln!(out, "resource_module entry={}", entry);
        for function in &self.functions {
            let _ = writeln!(
                out,
                "fn {} effect={:?} result=t{} span={}:{}-{}",
                function.name,
                function.effect,
                function.result.0,
                function.span.file_id.0,
                function.span.start,
                function.span.end
            );
            for param in &function.params {
                let _ = writeln!(
                    out,
                    "  param {} mut={} ty=t{} place={}",
                    param.name,
                    param.mutable,
                    param.ty.0,
                    dump_place(&param.place)
                );
            }
            for block in &function.blocks {
                let _ = writeln!(out, "  block b{}:", block.id.0);
                for op in &block.ops {
                    dump_op(&mut out, op, 4);
                }
                write_indent(&mut out, 4);
                let _ = writeln!(out, "terminator {}", dump_terminator(&block.terminator));
            }
        }
        out
    }
}

fn dump_op(out: &mut String, op: &ResourceOp, indent: usize) {
    write_indent(out, indent);
    match op {
        ResourceOp::Expr {
            kind,
            output,
            ty,
            span,
        } => {
            let _ = writeln!(
                out,
                "expr {:?} out={} ty=t{} span={}:{}-{}",
                kind,
                dump_place(output),
                ty.0,
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::DeclareLocal {
            place,
            mutable,
            initializer,
            span,
        } => {
            let init = initializer
                .as_ref()
                .map(dump_place)
                .unwrap_or_else(|| String::from("<none>"));
            let _ = writeln!(
                out,
                "declare {} mut={} init={} span={}:{}-{}",
                dump_place(place),
                mutable,
                init,
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::Read {
            source,
            output,
            span,
        } => {
            let _ = writeln!(
                out,
                "read {} -> {} span={}:{}-{}",
                dump_place(source),
                dump_place(output),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::Assign {
            target,
            value,
            span,
        } => {
            let _ = writeln!(
                out,
                "assign {} = {} span={}:{}-{}",
                dump_place(target),
                dump_place(value),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::Borrow {
            source,
            output,
            kind,
            span,
        } => {
            let _ = writeln!(
                out,
                "borrow {:?} {} -> {} span={}:{}-{}",
                kind,
                dump_place(source),
                dump_place(output),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::Move {
            source,
            output,
            span,
        } => {
            let _ = writeln!(
                out,
                "move {} -> {} span={}:{}-{}",
                dump_place(source),
                dump_place(output),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::Drop { place, span } => {
            let _ = writeln!(
                out,
                "drop {} span={}:{}-{}",
                dump_place(place),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::CallEffect { effect, span } => {
            let _ = writeln!(
                out,
                "effect {} span={}:{}-{}",
                dump_effect(effect),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::FunctionValue {
            output,
            name,
            effect,
            span,
        } => {
            let _ = writeln!(
                out,
                "function_value {} out={} effect={} span={}:{}-{}",
                name,
                dump_place(output),
                dump_effect(effect),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::Call {
            output,
            target,
            args,
            effect,
            span,
        } => {
            let _ = writeln!(
                out,
                "call {} out={} args=[{}] effect={} span={}:{}-{}",
                dump_call_target(target),
                dump_place(output),
                dump_place_list(args),
                dump_effect(effect),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            params,
            result,
            args,
            effect,
            span,
        } => {
            let _ = writeln!(
                out,
                "indirect_call out={} callee={} params=[{}] result=t{} args=[{}] effect={} span={}:{}-{}",
                dump_place(output),
                dump_place(callee),
                dump_type_list(params),
                result.0,
                dump_place_list(args),
                dump_effect(effect),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::RawMemory {
            operation,
            output,
            args,
            span,
        } => {
            let _ = writeln!(
                out,
                "raw_memory {} out={} args=[{}] span={}:{}-{}",
                dump_raw_memory_op(operation),
                dump_place(output),
                dump_place_list(args),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::RawAddressAlias {
            source,
            target,
            span,
        } => {
            let _ = writeln!(
                out,
                "raw_address_alias {} -> {} span={}:{}-{}",
                dump_place(source),
                dump_place(target),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::RawAddressView {
            source,
            target,
            span,
        } => {
            let _ = writeln!(
                out,
                "raw_address_view {} -> {} span={}:{}-{}",
                dump_place(source),
                dump_place(target),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::Construct {
            output,
            kind,
            inputs,
            span,
        } => {
            let _ = writeln!(
                out,
                "construct {} {} inputs=[{}] span={}:{}-{}",
                dump_construct_kind(kind),
                dump_place(output),
                dump_place_list(inputs),
                span.file_id.0,
                span.start,
                span.end
            );
        }
        ResourceOp::Branch {
            output,
            condition,
            condition_fact,
            then_ops,
            then_value,
            else_ops,
            else_value,
            span,
        } => {
            let _ = writeln!(
                out,
                "branch {} cond={} fact={} span={}:{}-{}",
                dump_place(output),
                dump_place(condition),
                dump_condition_fact(condition_fact),
                span.file_id.0,
                span.start,
                span.end
            );
            write_indent(out, indent + 2);
            let _ = writeln!(out, "then value={}:", dump_place(then_value));
            for op in then_ops {
                dump_op(out, op, indent + 4);
            }
            write_indent(out, indent + 2);
            let _ = writeln!(out, "else value={}:", dump_place(else_value));
            for op in else_ops {
                dump_op(out, op, indent + 4);
            }
        }
        ResourceOp::Loop {
            condition_ops,
            condition,
            body_ops,
            span,
        } => {
            let _ = writeln!(
                out,
                "loop cond={} span={}:{}-{}",
                dump_place(condition),
                span.file_id.0,
                span.start,
                span.end
            );
            write_indent(out, indent + 2);
            let _ = writeln!(out, "condition:");
            for op in condition_ops {
                dump_op(out, op, indent + 4);
            }
            write_indent(out, indent + 2);
            let _ = writeln!(out, "body:");
            for op in body_ops {
                dump_op(out, op, indent + 4);
            }
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            span,
        } => {
            let _ = writeln!(
                out,
                "match {} scrutinee={} span={}:{}-{}",
                dump_place(output),
                dump_place(scrutinee),
                span.file_id.0,
                span.start,
                span.end
            );
            for arm in arms {
                write_indent(out, indent + 2);
                let bind = arm
                    .bind_local
                    .as_ref()
                    .map(dump_place)
                    .unwrap_or_else(|| String::from("<none>"));
                let _ = writeln!(
                    out,
                    "arm {} bind={} value={}:",
                    dump_match_pattern(&arm.pattern),
                    bind,
                    dump_place(&arm.value)
                );
                for op in &arm.ops {
                    dump_op(out, op, indent + 4);
                }
            }
        }
    }
}

fn write_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push(' ');
    }
}

fn dump_terminator(terminator: &ResourceTerminator) -> String {
    match terminator {
        ResourceTerminator::Return { value, span } => {
            let value = value
                .as_ref()
                .map(dump_place)
                .unwrap_or_else(|| String::from("<implicit>"));
            format!(
                "return {} span={}:{}-{}",
                value, span.file_id.0, span.start, span.end
            )
        }
        ResourceTerminator::Unreachable { span } => {
            format!(
                "unreachable span={}:{}-{}",
                span.file_id.0, span.start, span.end
            )
        }
        ResourceTerminator::RawBody { kind, span } => format!(
            "raw_body {} span={}:{}-{}",
            dump_raw_body_kind(*kind),
            span.file_id.0,
            span.start,
            span.end
        ),
    }
}

fn dump_raw_body_kind(kind: RawBodyKind) -> &'static str {
    match kind {
        RawBodyKind::Wasm => "wasm",
        RawBodyKind::LlvmIr => "llvmir",
    }
}

fn dump_condition_fact(fact: &Option<ResourceConditionFact>) -> String {
    match fact {
        Some(fact) => dump_condition_fact_value(fact),
        None => String::from("<none>"),
    }
}

fn dump_condition_fact_value(fact: &ResourceConditionFact) -> String {
    match fact {
        ResourceConditionFact::EqZero { place } => {
            format!("eq_zero({})", dump_place(place))
        }
        ResourceConditionFact::NeZero { place } => {
            format!("ne_zero({})", dump_place(place))
        }
        ResourceConditionFact::Positive { place } => {
            format!("positive({})", dump_place(place))
        }
        ResourceConditionFact::NonPositive { place } => {
            format!("non_positive({})", dump_place(place))
        }
        ResourceConditionFact::Negative { place } => {
            format!("negative({})", dump_place(place))
        }
        ResourceConditionFact::NonNegative { place } => {
            format!("non_negative({})", dump_place(place))
        }
        ResourceConditionFact::Any(facts) => dump_condition_fact_list("any", facts),
        ResourceConditionFact::All(facts) => dump_condition_fact_list("all", facts),
    }
}

fn dump_condition_fact_list(name: &str, facts: &[ResourceConditionFact]) -> String {
    let mut out = format!("{}(", name);
    for (index, fact) in facts.iter().enumerate() {
        if index > 0 {
            out.push_str(", ");
        }
        out.push_str(&dump_condition_fact_value(fact));
    }
    out.push(')');
    out
}

fn dump_place(place: &Place) -> String {
    let mut out = match &place.root {
        PlaceRoot::Local(name) => format!("%{}:t{}", name, place.ty.0),
        PlaceRoot::Temporary(id) => format!("tmp{}:t{}", id.0, place.ty.0),
        PlaceRoot::Return => format!("return:t{}", place.ty.0),
        PlaceRoot::Storage(id) => format!("storage{}:t{}", id.0, place.ty.0),
        PlaceRoot::Unknown => format!("unknown:t{}", place.ty.0),
    };
    for projection in &place.projections {
        out.push_str(&dump_projection(projection));
    }
    out
}

fn dump_place_list(places: &[Place]) -> String {
    let mut out = String::new();
    for (idx, place) in places.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        out.push_str(&dump_place(place));
    }
    out
}

fn dump_type_list(types: &[crate::types::TypeId]) -> String {
    let mut out = String::new();
    for (idx, ty) in types.iter().enumerate() {
        if idx > 0 {
            out.push_str(", ");
        }
        let _ = write!(out, "t{}", ty.0);
    }
    out
}

fn dump_projection(projection: &PlaceProjection) -> String {
    match projection {
        PlaceProjection::Field {
            index,
            offset_bytes,
        } => format!(".field{}@{}", index, offset_bytes),
        PlaceProjection::TupleField {
            index,
            offset_bytes,
        } => format!(".tuple{}@{}", index, offset_bytes),
        PlaceProjection::EnumPayload { variant } => format!(".payload({})", variant),
        PlaceProjection::Deref => String::from(".*"),
        PlaceProjection::StorageOffset(offset) => match offset {
            super::model::ResourceOffset::Exact(bytes) => format!("[+{}]", bytes),
            super::model::ResourceOffset::Dynamic => String::from("[+?]"),
        },
    }
}

fn dump_effect(effect: &EffectOp) -> String {
    match effect {
        EffectOp::Pure => String::from("pure"),
        EffectOp::UserCall { name, effect } => format!("call({},{:?})", name, effect),
        EffectOp::IndirectCall { effect } => format!("indirect_call({:?})", effect),
        EffectOp::InternalAlloc => String::from("internal_alloc"),
        EffectOp::UnsafeMemory { operation } => format!("unsafe_memory({})", operation),
        EffectOp::ExternalIo { operation } => format!("external_io({})", operation),
        EffectOp::Nondet { operation } => format!("nondet({})", operation),
        EffectOp::Unknown { reason } => format!("unknown({})", reason),
    }
}

fn dump_construct_kind(kind: &AggregateKind) -> String {
    match kind {
        AggregateKind::Enum { name, variant } => format!("enum({}::{})", name, variant),
        AggregateKind::Struct { name, .. } => format!("struct({})", name),
        AggregateKind::Tuple { .. } => String::from("tuple"),
    }
}

fn dump_raw_memory_op(operation: &RawMemoryOp) -> String {
    match operation {
        RawMemoryOp::Alloc => String::from("alloc"),
        RawMemoryOp::Dealloc => String::from("dealloc"),
        RawMemoryOp::Realloc => String::from("realloc"),
        RawMemoryOp::Load => String::from("load"),
        RawMemoryOp::Store => String::from("store"),
        RawMemoryOp::BulkCopy => String::from("bulk_copy"),
        RawMemoryOp::BulkMove => String::from("bulk_move"),
        RawMemoryOp::MemorySize => String::from("memory_size"),
        RawMemoryOp::MemoryGrow => String::from("memory_grow"),
        RawMemoryOp::Fill => String::from("fill"),
        RawMemoryOp::Other { name } => format!("other({})", name),
    }
}

fn dump_call_target(target: &ResourceCallTarget) -> String {
    match target {
        ResourceCallTarget::Builtin { name } => format!("builtin({})", name),
        ResourceCallTarget::User { name, type_args } => {
            format!("user({}<{}>)", name, dump_type_list(type_args))
        }
        ResourceCallTarget::Trait {
            trait_name,
            trait_args,
            method,
            self_ty,
        } => format!(
            "trait({}<{}>::{} self=t{})",
            trait_name,
            dump_type_list(trait_args),
            method,
            self_ty.0
        ),
    }
}

fn dump_match_pattern(pattern: &ResourceMatchPattern) -> String {
    match pattern {
        ResourceMatchPattern::Variant(name) => format!("variant({})", name),
        ResourceMatchPattern::IntLiteral(value) => format!("int({})", value),
        ResourceMatchPattern::BoolLiteral(value) => format!("bool({})", value),
        ResourceMatchPattern::Wildcard => String::from("_"),
    }
}
