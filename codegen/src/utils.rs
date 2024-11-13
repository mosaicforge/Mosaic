use swc_common::{Span, SyntaxContext};
use swc_core::quote_expr;
use swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, BindingIdent, BlockStmt, Class, ClassDecl, ClassMember,
    ClassMethod, ClassProp, Constructor, Expr, ExprStmt, Function, Ident, IdentName, MemberExpr,
    MemberProp, MethodKind, Param, ParamOrTsParamProp, Pat, PropName, ReturnStmt,
    SimpleAssignTarget, Stmt, TsExprWithTypeArgs, TsType, TsTypeAnn,
};

pub fn ident(name: impl ToString) -> Ident {
    Ident {
        span: Span::default(),
        ctxt: SyntaxContext::from_u32(0),
        sym: name.to_string().into(),
        optional: false,
    }
}

pub fn type_ann(ts_type: TsType) -> TsTypeAnn {
    TsTypeAnn {
        span: Span::default(),
        type_ann: Box::new(ts_type),
    }
}

pub fn class_prop(name: impl ToString, ts_type: TsType) -> ClassProp {
    ClassProp {
        span: Span::default(),
        key: PropName::Ident(IdentName {
            span: Span::default(),
            sym: name.to_string().into(),
        }),
        value: None,
        type_ann: Some(Box::new(type_ann(ts_type))),
        is_static: false,
        decorators: vec![],
        accessibility: None,
        is_abstract: false,
        is_optional: false,
        is_override: false,
        readonly: false,
        declare: false,
        definite: false,
    }
}

pub fn param(name: impl ToString, ts_type: TsType) -> Param {
    Param {
        span: Span::default(),
        decorators: vec![],
        pat: Pat::Ident(BindingIdent {
            id: ident(name),
            type_ann: Some(Box::new(type_ann(ts_type))),
        }),
    }
}

pub fn constructor(params: Vec<Param>, body: Option<Vec<Box<Expr>>>) -> Constructor {
    Constructor {
        span: Span::default(),
        ctxt: SyntaxContext::from_u32(0),
        key: PropName::Ident(IdentName {
            span: Span::default(),
            sym: "constructor".into(),
        }),
        params: params.into_iter().map(ParamOrTsParamProp::Param).collect(),
        body: Some(BlockStmt {
            span: Span::default(),
            ctxt: SyntaxContext::from_u32(0),
            stmts: body
                .unwrap_or_default()
                .into_iter()
                .map(|expr| {
                    Stmt::Expr(ExprStmt {
                        span: Span::default(),
                        expr,
                    })
                })
                .collect(),
        }),
        accessibility: None,
        is_optional: false,
    }
}

pub fn method(
    name: impl ToString,
    params: Vec<Param>,
    body: Option<Vec<Box<Expr>>>,
    r#return: Option<Box<Expr>>,
    is_async: bool,
    return_type: Option<TsType>,
) -> ClassMethod {
    let mut body: Vec<Stmt> = body
        .unwrap_or_default()
        .into_iter()
        .map(|expr| {
            Stmt::Expr(ExprStmt {
                span: Span::default(),
                expr,
            })
        })
        .collect();

    body.push(Stmt::Return(ReturnStmt {
        span: Span::default(),
        arg: r#return,
    }));

    ClassMethod {
        span: Span::default(),
        key: PropName::Ident(IdentName {
            span: Span::default(),
            sym: name.to_string().into(),
        }),
        function: Box::new(Function {
            params,
            decorators: vec![],
            span: Span::default(),
            ctxt: SyntaxContext::from_u32(0),
            body: Some(BlockStmt {
                span: Span::default(),
                ctxt: SyntaxContext::from_u32(0),
                stmts: body,
            }),
            is_generator: false,
            is_async,
            type_params: None,
            return_type: return_type.map(|return_type| Box::new(type_ann(return_type))),
        }),
        kind: MethodKind::Method,
        is_static: false,
        accessibility: None,
        is_abstract: false,
        is_optional: false,
        is_override: false,
    }
}

pub fn class(
    name: impl ToString,
    attributes: Vec<ClassProp>,
    constructor: Option<Constructor>,
    methods: Vec<ClassMethod>,
    implements: Vec<Ident>,
    extends: Option<Ident>,
) -> ClassDecl {
    let body: Vec<ClassMember> = attributes
        .into_iter()
        .map(ClassMember::ClassProp)
        .chain(constructor.into_iter().map(ClassMember::Constructor))
        .chain(methods.into_iter().map(ClassMember::Method))
        .collect();

    ClassDecl {
        ident: ident(name),
        declare: false,
        class: Box::new(Class {
            span: Span::default(),
            ctxt: SyntaxContext::from_u32(0),
            decorators: vec![],
            body,
            super_class: extends.map(|extends| Box::new(Expr::Ident(extends))),
            is_abstract: false,
            type_params: None,
            super_type_params: None,
            implements: implements
                .into_iter()
                .map(|implements| TsExprWithTypeArgs {
                    span: Span::default(),
                    expr: Box::new(Expr::Ident(implements)),
                    type_args: None,
                })
                .collect(),
        }),
    }
}

pub fn assign_this(prop_name: impl ToString, value: Box<Expr>) -> Expr {
    Expr::Assign(AssignExpr {
        span: Span::default(),
        op: AssignOp::Assign,
        left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
            span: Span::default(),
            obj: quote_expr!("this"),
            prop: MemberProp::Ident(IdentName {
                span: Span::default(),
                sym: prop_name.to_string().into(),
            }),
        })),
        right: value,
    })
}
