use crate::ast::*;
use crate::error::ParseError;
use crate::glyph::{RenderCtx, SymbolRegistry};
use crate::layout::RenderNode;

pub fn render(
    expr: &Expr,
    reg: &SymbolRegistry,
    ctx: &mut RenderCtx,
) -> Result<RenderNode, ParseError> {
    match expr {
        Expr::Ident(s) | Expr::Number(s) => Ok(RenderNode::from_str(s)),

        Expr::Group(inner) => render(inner, reg, ctx),

        Expr::Parens(inner) => {
            let inner = render(inner, reg, ctx)?;
            Ok(RenderNode::stretchy_delim(&inner, '(', ')', false))
        }

        Expr::Brackets(inner) => {
            let inner = render(inner, reg, ctx)?;
            Ok(RenderNode::stretchy_delim(&inner, '[', ']', false))
        }

        Expr::Delimiter { left, right, inner } => {
            let inner = render(inner, reg, ctx)?;
            Ok(RenderNode::stretchy_delim(&inner, *left, *right, false))
        }

        Expr::Neg(inner) => {
            let inner = render(inner, reg, ctx)?;
            let mut result = RenderNode::new(inner.width + 1, inner.height, inner.baseline);
            result.data[inner.baseline * result.width] = '-';
            inner.blit_into(&mut result.data, result.width, 1, 0);
            Ok(result)
        }

        Expr::Command { name, opts, args } => {
            if let Some(glyph) = reg.get(name) {
                let (rendered_opts, rendered_args) = {
                    ctx.depth += 1;
                    let rendered = (|| {
                        Ok((
                            opts.iter()
                                .map(|a| render(a, reg, ctx))
                                .collect::<Result<Vec<_>, _>>()?,
                            args.iter()
                                .map(|a| render(a, reg, ctx))
                                .collect::<Result<Vec<_>, _>>()?,
                        ))
                    })();
                    ctx.depth -= 1;
                    rendered?
                };
                Ok(glyph.render(&rendered_args, &rendered_opts, ctx))
            } else {
                Ok(RenderNode::from_str(name))
            }
        }

        Expr::Superscript(base, sup) => {
            if let Expr::Command { name, .. } = base.as_ref()
                && let Some(glyph) = reg.get(name)
                && glyph.has_limits()
            {
                let base_r = render(base, reg, ctx)?;
                let sup_r = render(sup, reg, ctx)?;
                return Ok(RenderNode::limits(
                    &base_r,
                    &RenderNode::new(0, 0, 0),
                    &sup_r,
                ));
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
                return Ok(RenderNode::limits(
                    &base_r,
                    &sub_r,
                    &RenderNode::new(0, 0, 0),
                ));
            }

            let base = render(base, reg, ctx)?;
            let sub = render(sub, reg, ctx)?;
            Ok(RenderNode::subscript(&base, &sub))
        }

        Expr::BothScripts(base, sub, sup) => {
            if let Expr::Command { name, .. } = base.as_ref()
                && let Some(glyph) = reg.get(name)
                && glyph.has_limits()
            {
                let base_r = render(base, reg, ctx)?;
                let sub_r = render(sub, reg, ctx)?;
                let sup_r = render(sup, reg, ctx)?;
                return Ok(RenderNode::limits(&base_r, &sub_r, &sup_r));
            }

            let base_rendered = render(base, reg, ctx)?;
            let sub_rendered = render(sub, reg, ctx)?;
            let sup_rendered = render(sup, reg, ctx)?;
            Ok(RenderNode::both_scripts(
                &base_rendered,
                &sub_rendered,
                &sup_rendered,
            ))
        }

        Expr::Prime(base, n) => {
            let base = render(base, reg, ctx)?;
            Ok(RenderNode::prime_suffix(&base, *n))
        }

        Expr::BinOp(lhs, op, rhs) => {
            let lhs = render(lhs, reg, ctx)?;
            let rhs = render(rhs, reg, ctx)?;
            let op_char = match op {
                BinOp::Add => '+',
                BinOp::Sub => '-',
                BinOp::Eq => '=',
                BinOp::Mul => '·',
            };
            Ok(RenderNode::infix(&lhs, op_char, &rhs))
        }

        Expr::Escape(s) => Ok(match s.as_str() {
            " " => RenderNode::new(4, 1, 0),
            "," => RenderNode::new(1, 1, 0),
            ":" => RenderNode::new(2, 1, 0),
            ";" => RenderNode::new(3, 1, 0),
            "!" => RenderNode::new(0, 1, 0),
            _ => RenderNode::from_str(s),
        }),

        Expr::Juxtapose(exprs) => {
            let nodes: Vec<RenderNode> = exprs
                .iter()
                .map(|e| render(e, reg, ctx))
                .collect::<Result<_, _>>()?;
            Ok(RenderNode::hstack(&nodes, 0))
        }

        Expr::Empty => Ok(RenderNode::new(0, 0, 0)),

        // optimize this?
        Expr::Matrix { name, rows } => {
            if rows.is_empty() {
                return Ok(RenderNode::new(0, 0, 0));
            }

            let mut rendered_rows: Vec<Vec<RenderNode>> = Vec::new();

            let num_cols = rows[0].len();
            for row in rows {
                if row.len() != num_cols {
                    return Err(ParseError("matrix rows have different lengths".into()));
                }

                let mut rendered_row: Vec<RenderNode> = Vec::new();
                for item in row {
                    let rendered_item = render(item, reg, ctx)?;
                    rendered_row.push(rendered_item);
                }

                rendered_rows.push(rendered_row);
            }

            RenderNode::matrix(name, &rendered_rows)
        }
    }
}

fn render_power(
    base: RenderNode,
    exp: &Expr,
    reg: &SymbolRegistry,
    ctx: &mut RenderCtx,
) -> Result<RenderNode, ParseError> {
    if crate::COMPACT_SIMPLE_FRACTIONAL_EXPONENTS
        && let Expr::Command { name, args, .. } = exp
        && name == "frac"
        && args.len() == 2
        && let (Expr::Number(n), Expr::Number(d)) = (&args[0], &args[1])
    {
        let exp_str = format!("{n}/{d}");
        let exp_node = RenderNode::from_str(&exp_str);
        return Ok(RenderNode::superscript(&base, &exp_node));
    }

    let rendered_exp = render(exp, reg, ctx)?;
    Ok(RenderNode::superscript(&base, &rendered_exp))
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
        let rows: Vec<String> = node
            .data
            .chunks(node.width)
            .map(|row| row.iter().collect())
            .collect();
        let index_row = rows.iter().position(|row| row.contains('3')).unwrap();
        let radicand_row = rows.iter().position(|row| row.contains('8')).unwrap();

        assert!(index_row < radicand_row);
    }
}
