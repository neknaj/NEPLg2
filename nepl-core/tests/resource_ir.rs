use nepl_core::ast::Effect;
use nepl_core::hir::{
    HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use nepl_core::resource::{lower_hir_module_skeleton, PlaceRoot, ResourceExprKind, ResourceOp};
use nepl_core::span::Span;
use nepl_core::types::TypeId;

#[test]
fn resource_ir_lowering_skeleton_tracks_locals_and_dump() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(2),
            params: vec![HirParam {
                name: "arg".to_string(),
                ty: i32_ty,
                mutable: false,
            }],
            result: unit_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![
                    HirLine {
                        expr: HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::Let {
                                name: "x".to_string(),
                                mutable: true,
                                value: Box::new(HirExpr {
                                    ty: i32_ty,
                                    kind: HirExprKind::LiteralI32(1),
                                    span,
                                }),
                            },
                            span,
                        },
                        drop_result: true,
                    },
                    HirLine {
                        expr: HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::Var("x".to_string()),
                            span,
                        },
                        drop_result: true,
                    },
                ],
                ty: unit_ty,
                span,
            }),
            span,
        }],
        entry: Some("main".to_string()),
        externs: vec![],
        string_literals: vec![],
        traits: vec![],
        impls: vec![],
    };

    let resource = lower_hir_module_skeleton(&module);
    let function = &resource.functions[0];
    assert_eq!(function.params.len(), 1);
    assert_eq!(function.blocks.len(), 1);
    assert!(function.blocks[0].ops.iter().any(|op| matches!(
        op,
        ResourceOp::DeclareLocal { place, mutable: true, .. }
            if matches!(&place.root, PlaceRoot::Local(name) if name == "x")
    )));
    assert!(function.blocks[0].ops.iter().any(|op| matches!(
        op,
        ResourceOp::Expr {
            kind: ResourceExprKind::LocalRead,
            ..
        }
    )));

    assert_eq!(
        resource.dump_text(),
        concat!(
            "resource_module entry=main\n",
            "fn main effect=Pure result=t0 span=0:0-0\n",
            "  param arg mut=false ty=t1 place=%arg:t1\n",
            "  block b0:\n",
            "    expr Block ty=t0 span=0:0-0\n",
            "    expr Literal ty=t1 span=0:0-0\n",
            "    declare %x:t1 mut=true span=0:0-0\n",
            "    expr Let ty=t1 span=0:0-0\n",
            "    expr LocalRead ty=t1 span=0:0-0\n",
            "    read %x:t1 span=0:0-0\n",
            "    terminator return <implicit> span=0:0-0\n"
        )
    );
}
