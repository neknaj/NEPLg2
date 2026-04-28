extern crate alloc;

use alloc::format;
use alloc::string::String;
use core::fmt::Write;

use super::model::{
    EffectOp, Place, PlaceProjection, PlaceRoot, RawBodyKind, ResourceModule, ResourceOp,
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
                    let _ = writeln!(out, "    {}", dump_op(op));
                }
                let _ = writeln!(out, "    terminator {}", dump_terminator(&block.terminator));
            }
        }
        out
    }
}

fn dump_op(op: &ResourceOp) -> String {
    match op {
        ResourceOp::Expr { kind, ty, span } => format!(
            "expr {:?} ty=t{} span={}:{}-{}",
            kind, ty.0, span.file_id.0, span.start, span.end
        ),
        ResourceOp::DeclareLocal {
            place,
            mutable,
            span,
        } => format!(
            "declare {} mut={} span={}:{}-{}",
            dump_place(place),
            mutable,
            span.file_id.0,
            span.start,
            span.end
        ),
        ResourceOp::Read { source, span } => format!(
            "read {} span={}:{}-{}",
            dump_place(source),
            span.file_id.0,
            span.start,
            span.end
        ),
        ResourceOp::Assign { target, span } => format!(
            "assign {} span={}:{}-{}",
            dump_place(target),
            span.file_id.0,
            span.start,
            span.end
        ),
        ResourceOp::Borrow { source, kind, span } => format!(
            "borrow {:?} {} span={}:{}-{}",
            kind,
            dump_place(source),
            span.file_id.0,
            span.start,
            span.end
        ),
        ResourceOp::Move { source, span } => format!(
            "move {} span={}:{}-{}",
            dump_place(source),
            span.file_id.0,
            span.start,
            span.end
        ),
        ResourceOp::Drop { place, span } => format!(
            "drop {} span={}:{}-{}",
            dump_place(place),
            span.file_id.0,
            span.start,
            span.end
        ),
        ResourceOp::CallEffect { effect, span } => format!(
            "effect {} span={}:{}-{}",
            dump_effect(effect),
            span.file_id.0,
            span.start,
            span.end
        ),
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
        PlaceProjection::StorageOffset(offset) => match offset.bytes {
            Some(bytes) => format!("[+{}]", bytes),
            None => String::from("[+?]"),
        },
    }
}

fn dump_effect(effect: &EffectOp) -> String {
    match effect {
        EffectOp::Pure => String::from("pure"),
        EffectOp::UserCall { name, effect } => format!("call({},{:?})", name, effect),
        EffectOp::InternalAlloc => String::from("internal_alloc"),
        EffectOp::UnsafeMemory { operation } => format!("unsafe_memory({})", operation),
        EffectOp::ExternalIo { operation } => format!("external_io({})", operation),
        EffectOp::Unknown { reason } => format!("unknown({})", reason),
    }
}
