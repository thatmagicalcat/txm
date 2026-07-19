use crate::ast::*;
use crate::error::ParseError;
use crate::glyph::{RenderCtx, SymbolRegistry};
use crate::layout_tree::LayoutNode;

pub fn render(
    expr: &Expr,
    reg: &SymbolRegistry,
    ctx: &mut RenderCtx,
) -> Result<LayoutNode, ParseError> {
    match expr {
        Expr::Ident(s) => {
            let mut node = LayoutNode::text(s.chars().collect());
            node.style = ctx.current_style;
            Ok(node)
        }

        Expr::Number(s) => {
            let mut node = LayoutNode::text_str(s);
            node.style = ctx.current_style;
            Ok(node)
        }

        Expr::Delimiter { left, right, inner } => {
            let inner = render(inner, reg, ctx)?;
            Ok(LayoutNode::stretchy_delim(
                inner,
                *left,
                *right,
                false,
                ctx.current_style.un_italic(),
            ))
        }

        Expr::Neg(inner) => {
            let inner = render(inner, reg, ctx)?;
            let mut node = LayoutNode::negate(inner);
            node.style = ctx.current_style;
            Ok(node)
        }

        Expr::Command { name, opts, args } => {
            if let Some(glyph) = reg.get(name) {
                let mut eval =
                    |expr: &Expr, eval_ctx: &mut RenderCtx| -> Result<LayoutNode, ParseError> {
                        render(expr, reg, eval_ctx)
                    };

                ctx.depth += 1;
                let rendered_node = glyph.render_macro(args, opts, ctx, &mut eval);
                ctx.depth -= 1;

                rendered_node
            } else {
                let mut node = LayoutNode::text_str(name);
                node.style = ctx.current_style;
                Ok(node)
            }
        }

        Expr::Superscript(base, sup) => {
            if let Expr::Command { name, .. } = base.as_ref()
                && let Some(glyph) = reg.get(name)
                && glyph.has_limits()
            {
                let base_r = render(base, reg, ctx)?;
                let sup_r = render(sup, reg, ctx)?;
                return Ok(LayoutNode::limits(base_r, LayoutNode::empty(), sup_r));
            }

            let base = render(base, reg, ctx)?;
            render_power(base, sup, reg, ctx)
        }

        Expr::Subscript(base, sub) => {
            if let Expr::Command { name, .. } = base.as_ref()
                && let Some(glyph) = reg.get(name)
                && glyph.has_limits()
            {
                let base_r = render(base, reg, ctx)?;
                let sub_r = render(sub, reg, ctx)?;
                return Ok(LayoutNode::limits(base_r, sub_r, LayoutNode::empty()));
            }

            let base = render(base, reg, ctx)?;
            let sub = render(sub, reg, ctx)?;
            Ok(LayoutNode::subscript(base, sub, ctx.current_style))
        }

        Expr::BothScripts(base, sub, sup) => {
            if let Expr::Command { name, .. } = base.as_ref()
                && let Some(glyph) = reg.get(name)
                && glyph.has_limits()
            {
                let base_r = render(base, reg, ctx)?;
                let sub_r = render(sub, reg, ctx)?;
                let sup_r = render(sup, reg, ctx)?;
                return Ok(LayoutNode::limits(base_r, sub_r, sup_r));
            }

            let base_rendered = render(base, reg, ctx)?;
            let sub_rendered = render(sub, reg, ctx)?;
            let sup_rendered = render(sup, reg, ctx)?;

            Ok(LayoutNode::both_scripts(
                base_rendered,
                sub_rendered,
                sup_rendered,
                ctx.current_style,
            ))
        }

        Expr::Prime(base, n) => {
            let base = render(base, reg, ctx)?;
            let mut node = LayoutNode::prime(base, *n);
            node.style = ctx.current_style;
            Ok(node)
        }

        Expr::BinOp(lhs, op, rhs) => {
            let lhs = render(lhs, reg, ctx)?;
            let rhs = render(rhs, reg, ctx)?;
            let mut node = LayoutNode::infix(lhs, *op, rhs);
            node.style = ctx.current_style;
            Ok(node)
        }

        Expr::Escape(s) => {
            let mut node = match s.as_str() {
                " " => LayoutNode::text(vec![' '; 4]),
                "," => LayoutNode::text(vec![' ']),
                ":" => LayoutNode::text(vec![':'; 2]),
                ";" => LayoutNode::text(vec![';'; 3]),
                "!" => LayoutNode::empty(),
                _ => LayoutNode::text_str(s),
            };
            node.style = ctx.current_style;
            Ok(node)
        }

        Expr::Juxtapose(exprs) => {
            let nodes: Vec<LayoutNode> = exprs
                .iter()
                .map(|e| render(e, reg, ctx))
                .collect::<Result<_, _>>()?;
            Ok(LayoutNode::hstack(&nodes, 0))
        }

        Expr::Empty => Ok(LayoutNode::empty()),

        Expr::Matrix { name, rows } => {
            if rows.is_empty() {
                return Ok(LayoutNode::empty());
            }

            let mut rendered_rows: Vec<Vec<LayoutNode>> = Vec::new();

            let num_cols = rows[0].len();
            for row in rows {
                if row.len() != num_cols {
                    return Err(ParseError::MismatchedMatrixRows);
                }

                let mut rendered_row: Vec<LayoutNode> = Vec::new();
                for item in row {
                    let rendered_item = render(item, reg, ctx)?;
                    rendered_row.push(rendered_item);
                }

                rendered_rows.push(rendered_row);
            }

            LayoutNode::matrix(name, &rendered_rows, ctx.current_style.un_italic())
        }
    }
}

fn render_power(
    base: LayoutNode,
    exp: &Expr,
    reg: &SymbolRegistry,
    ctx: &mut RenderCtx,
) -> Result<LayoutNode, ParseError> {
    if crate::COMPACT_SIMPLE_FRACTIONAL_EXPONENTS
        && let Expr::Command { name, args, .. } = exp
        && name == "frac"
        && args.len() == 2
        && let (Expr::Number(n), Expr::Number(d)) = (&args[0], &args[1])
    {
        let exp_str = format!("{n}/{d}");
        let exp_node = LayoutNode::text_str(&exp_str);
        return Ok(LayoutNode::superscript(base, exp_node, ctx.current_style));
    }

    let rendered_exp = render(exp, reg, ctx)?;
    Ok(LayoutNode::superscript(
        base,
        rendered_exp,
        ctx.current_style,
    ))
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::glyph::{RenderCtx, SqrtGlyph, SymbolRegistry};
    use crate::parser::Parser;
    use crate::token::tokenize;

    #[test]
    fn sqrt_optional_index_keeps_radicand() {
        let mut registry = SymbolRegistry::new();
        registry.register("sqrt", SqrtGlyph);
        let input = r"\sqrt[3]{8}";
        let tokens = tokenize(input).unwrap();
        let expr = Parser::new(input, &tokens, &registry).parse_expr().unwrap();
        let node = render(&expr, &registry, &mut RenderCtx::default()).unwrap();

        // The sqrt node should have an index
        match &node.kind {
            crate::layout_tree::NodeKind::Sqrt { index, .. } => {
                assert!(index.is_some());
                let index = index.as_ref().unwrap();
                match &index.kind {
                    crate::layout_tree::NodeKind::Text { content } => {
                        assert_eq!(content, &vec!['3']);
                    }
                    _ => panic!("expected text node for index"),
                }
            }
            _ => panic!("expected sqrt node"),
        }
    }
}
