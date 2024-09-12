use crate::{ast::Literal, constraint::AllEqualTypeConstraint};

use super::*;

#[derive(Debug)]
pub struct U64Sort {
    name: Symbol,
}

impl U64Sort {
    pub fn new(name: Symbol) -> Self {
        Self { name }
    }
}

impl Sort for U64Sort {
    fn name(&self) -> Symbol {
        self.name
    }

    fn as_arc_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync + 'static> {
        self
    }

    #[rustfmt::skip]
    // We need the closure for division and mod operations, as they can panic.
    // cf https://github.com/rust-lang/rust-clippy/issues/9422
    #[allow(clippy::unnecessary_lazy_evaluations)]
    fn register_primitives(self: Arc<Self>, typeinfo: &mut TypeInfo) {
        typeinfo.add_primitive(TermOrderingMin {
           });
        typeinfo.add_primitive(TermOrderingMax {
           });

        type Opt<T=()> = Option<T>;

        add_primitives!(typeinfo, "+" = |a: u64, b: u64| -> u64 { a + b });
        add_primitives!(typeinfo, "-" = |a: u64, b: u64| -> u64 { a - b });
        add_primitives!(typeinfo, "*" = |a: u64, b: u64| -> u64 { a * b });
        add_primitives!(typeinfo, "/" = |a: u64, b: u64| -> Opt<u64> { (b != 0).then(|| a / b) });
        add_primitives!(typeinfo, "%" = |a: u64, b: u64| -> Opt<u64> { (b != 0).then(|| a % b) });

        add_primitives!(typeinfo, "&" = |a: u64, b: u64| -> u64 { a & b });
        add_primitives!(typeinfo, "|" = |a: u64, b: u64| -> u64 { a | b });
        add_primitives!(typeinfo, "^" = |a: u64, b: u64| -> u64 { a ^ b });
        add_primitives!(typeinfo, "<<" = |a: u64, b: u64| -> Opt<u64> { b.try_into().ok().and_then(|b| a.checked_shl(b)) });
        add_primitives!(typeinfo, ">>" = |a: u64, b: u64| -> Opt<u64> { b.try_into().ok().and_then(|b| a.checked_shr(b)) });

        add_primitives!(typeinfo, "log2" = |a: u64| -> u64 { (a as u64).ilog2() as u64 });

        add_primitives!(typeinfo, "<" = |a: u64, b: u64| -> Opt { (a < b).then(|| ()) });
        add_primitives!(typeinfo, ">" = |a: u64, b: u64| -> Opt { (a > b).then(|| ()) });
        add_primitives!(typeinfo, "<=" = |a: u64, b: u64| -> Opt { (a <= b).then(|| ()) });
        add_primitives!(typeinfo, ">=" = |a: u64, b: u64| -> Opt { (a >= b).then(|| ()) });

        add_primitives!(typeinfo, "bool-=" = |a: u64, b: u64| -> bool { a == b });
        add_primitives!(typeinfo, "bool-<" = |a: u64, b: u64| -> bool { a < b });
        add_primitives!(typeinfo, "bool->" = |a: u64, b: u64| -> bool { a > b });
        add_primitives!(typeinfo, "bool-<=" = |a: u64, b: u64| -> bool { a <= b });
        add_primitives!(typeinfo, "bool->=" = |a: u64, b: u64| -> bool { a >= b });

        add_primitives!(typeinfo, "min" = |a: u64, b: u64| -> u64 { a.min(b) });
        add_primitives!(typeinfo, "max" = |a: u64, b: u64| -> u64 { a.max(b) });

        add_primitives!(typeinfo, "to-string" = |a: u64| -> Symbol { a.to_string().into() });

        // Must be in the u64 sort register function because the string sort is registered before the u64 sort.
        typeinfo.add_primitive(CountMatches {
            name: "count-matches".into(),
            string: typeinfo.get_sort_nofail(),
            uint: self.clone(),
        });

    }

    fn make_expr(&self, _egraph: &EGraph, value: Value) -> (Cost, Expr) {
        assert!(value.tag == self.name());
        (
            1,
            GenericExpr::Lit(DUMMY_SPAN.clone(), Literal::Int(value.bits as _)),
        )
    }
}

impl IntoSort for u64 {
    type Sort = U64Sort;
    fn store(self, sort: &Self::Sort) -> Option<Value> {
        Some(Value {
            tag: sort.name,
            bits: self,
        })
    }
}

impl FromSort for u64 {
    type Sort = U64Sort;
    fn load(_sort: &Self::Sort, value: &Value) -> Self {
        value.bits as Self
    }
}

struct CountMatches {
    name: Symbol,
    string: Arc<StringSort>,
    uint: Arc<U64Sort>,
}

impl PrimitiveLike for CountMatches {
    fn name(&self) -> Symbol {
        self.name
    }

    fn get_type_constraints(&self, span: &Span) -> Box<dyn TypeConstraint> {
        AllEqualTypeConstraint::new(self.name(), span.clone())
            .with_all_arguments_sort(self.string.clone())
            .with_exact_length(3)
            .with_output_sort(self.uint.clone())
            .into_box()
    }

    fn apply(&self, values: &[Value], _egraph: Option<&mut EGraph>) -> Option<Value> {
        let string1 = Symbol::load(&self.string, &values[0]).to_string();
        let string2 = Symbol::load(&self.string, &values[1]).to_string();
        Some(Value::from(string1.matches(&string2).count() as u64))
    }
}
