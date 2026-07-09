use crate::ast::*;
use crate::glyph::{RenderCtx, SymbolRegistry};
use crate::layout::RenderNode;

const COMPACT_SIMPLE_FRACTIONAL_EXPONENTS: bool = false;

pub fn render(expr: &Expr, reg: &SymbolRegistry, ctx: &mut RenderCtx) -> RenderNode {
    match expr {
        Expr::Ident(s) | Expr::Number(s) => RenderNode::from_str(s),

        Expr::Group(inner) => render(inner, reg, ctx),

        Expr::Parens(inner) => {
            let inner = render(inner, reg, ctx);
            RenderNode::stretchy_delim(&inner, '(', ')')
        }

        Expr::Brackets(inner) => {
            let inner = render(inner, reg, ctx);
            RenderNode::stretchy_delim(&inner, '[', ']')
        }

        Expr::Neg(inner) => {
            let inner = render(inner, reg, ctx);
            let mut result = RenderNode::new(inner.width + 1, inner.height, inner.baseline);
            result.data[inner.baseline * result.width] = '-';
            inner.blit_into(&mut result.data, result.width, 1, 0);
            result
        }

        Expr::Command { name, args } => {
            if let Some(glyph) = reg.get(name) {
                ctx.depth += 1;
                let rendered_args: Vec<RenderNode> =
                    args.iter().map(|a| render(a, reg, ctx)).collect();
                ctx.depth -= 1;
                glyph.render(&rendered_args, &[], ctx)
            } else {
                eprintln!("DEBUG glyph NOT FOUND: {}", name);
                RenderNode::from_str(name)
            }
        }

        Expr::Superscript(base, exp) => {
            let base = render(base, reg, ctx);
            render_power(base, exp, reg, ctx)
        }

        Expr::Subscript(base, sub) => {
            let base = render(base, reg, ctx);
            let sub = render(sub, reg, ctx);
            RenderNode::subscript(&base, &sub)
        }

        Expr::BothScripts(base, sub, sup) => {
            let base_rendered = render(base, reg, ctx);
            let sub_rendered = render(sub, reg, ctx);
            let sup_rendered = render(sup, reg, ctx);
            RenderNode::both_scripts(&base_rendered, &sub_rendered, &sup_rendered)
        }

        Expr::Prime(base, n) => {
            let base = render(base, reg, ctx);
            RenderNode::prime_suffix(&base, *n)
        }

        Expr::BinOp(lhs, op, rhs) => {
            let lhs = render(lhs, reg, ctx);
            let rhs = render(rhs, reg, ctx);
            let op_char = match op {
                BinOp::Add => '+',
                BinOp::Sub => '-',
                BinOp::Eq => '=',
                BinOp::Mul => '·',
            };
            RenderNode::infix(&lhs, op_char, &rhs)
        }

        Expr::Escape(s) => {
            match s.as_str() {
                " " => RenderNode::new(4, 1, 0),
                "," => RenderNode::new(1, 1, 0),
                ":" => RenderNode::new(2, 1, 0),
                ";" => RenderNode::new(3, 1, 0),
                "!" => RenderNode::new(0, 1, 0),
                _ => RenderNode::from_str(s),
            }
        }

        Expr::Juxtapose(exprs) => {
            let nodes: Vec<RenderNode> = exprs.iter().map(|e| render(e, reg, ctx)).collect();
            RenderNode::hstack(&nodes, 0)
        }

        Expr::Empty => RenderNode::new(0, 0, 0),
    }
}

fn render_power(
    base: RenderNode,
    exp: &Expr,
    reg: &SymbolRegistry,
    ctx: &mut RenderCtx,
) -> RenderNode {
    if let Expr::Number(s) = exp
        && s.len() == 1
    {
        let c = s.chars().next().unwrap();
        if c.is_ascii_digit() {
            return RenderNode::superscript_digit(&base, c);
        }
    }

    if COMPACT_SIMPLE_FRACTIONAL_EXPONENTS
        && let Expr::Command { name, args } = exp
        && name == "frac"
        && args.len() == 2
        && let (Expr::Number(n), Expr::Number(d)) = (&args[0], &args[1])
    {
        let exp_str = format!("{n}/{d}");
        let exp_node = RenderNode::from_str(&exp_str);
        return RenderNode::superscript(&base, &exp_node);
    }

    let rendered_exp = render(exp, reg, ctx);
    RenderNode::superscript(&base, &rendered_exp)
}
