use crate::ast::BinOp;
use crate::backend::RenderTarget;
use crate::layout_tree::{LayoutNode, LineStyle, NodeKind};
use crate::style::Style;

pub(crate) fn render_node(node: &LayoutNode, buf: &mut impl RenderTarget, x: usize, y: usize) {
    render_inner(node, buf, x, y, Style::new())
}

fn render_inner(
    node: &LayoutNode,
    buf: &mut impl RenderTarget,
    x: usize,
    y: usize,
    inherited: Style,
) {
    let style = inherited.merge(node.style);

    match &node.kind {
        NodeKind::Text { content } => {
            for (i, &c) in content.iter().enumerate() {
                buf.set(x + i, y, c, style);
            }
        }

        NodeKind::HStack { children, spacing } => {
            let mut cx = x;
            for child in children {
                let cy = y + (node.baseline - child.baseline);
                render_inner(child, buf, cx, cy, style);
                cx += child.width + spacing;
            }
        }

        NodeKind::VStack { top, bottom, line } => {
            let inner_w = top.width.max(bottom.width);
            let pad = 1;
            let top_x = x + pad + (inner_w.saturating_sub(top.width)) / 2;
            let bot_x = x + pad + (inner_w.saturating_sub(bottom.width)) / 2;

            render_inner(top, buf, top_x, y, style);
            render_inner(bottom, buf, bot_x, y + top.height + 1, style);

            if *line == LineStyle::Solid {
                let w = node.width;
                buf.fill_row(y + top.height, x, x + w, '─', style);
            }
        }

        NodeKind::Infix { lhs, op, rhs } => {
            let baseline = lhs.baseline.max(rhs.baseline);
            let lhs_y = y + (baseline - lhs.baseline);
            let rhs_y = y + (baseline - rhs.baseline);

            render_inner(lhs, buf, x, lhs_y, style);

            let op_char = match op {
                BinOp::Add => '+',
                BinOp::Sub => '-',
                BinOp::Eq => '=',
                BinOp::Mul => '·',
            };
            buf.set(x + lhs.width + 1, y + baseline, op_char, style);

            render_inner(rhs, buf, x + lhs.width + 3, rhs_y, style);
        }

        NodeKind::Superscript { inline, base, exp } => {
            let inline_shift = if !inline { exp.height } else { 0 };

            render_inner(exp, buf, x + base.width, y, style);
            render_inner(base, buf, x, y + inline_shift, style);
        }

        NodeKind::Subscript { inline, base, sub } => {
            let inline_shift = if !inline { base.height } else { 0 };

            render_inner(base, buf, x, y, style);
            render_inner(sub, buf, x + base.width, y + inline_shift, style);
        }

        NodeKind::BothScripts { base, sub, sup } => {
            render_inner(sup, buf, x + base.width, y, style);
            render_inner(base, buf, x, y + sup.height, style);
            let sub_y = y + sup.height + base.baseline + 1;
            render_inner(sub, buf, x + base.width, sub_y, style);
        }

        NodeKind::StretchyDelim {
            inner,
            left,
            right,
            fill,
        } => {
            if inner.height <= 1 {
                let resolved_left = resolve_delimiter_single(*left);
                let resolved_right = resolve_delimiter_single(*right);
                buf.set(x, y, resolved_left, style);
                render_inner(inner, buf, x + 1, y, style);
                buf.set(x + node.width - 1, y, resolved_right, style);
            } else {
                let resolved =
                    resolve_delimiters(*left, *right, inner.height, *fill, inner.baseline);
                for (row, (l, r)) in resolved.iter().enumerate().take(inner.height) {
                    buf.set(x, y + row, *l, style);
                    buf.set(x + node.width - 1, y + row, *r, style);
                }
                render_inner(inner, buf, x + 2, y, style);
            }
        }

        NodeKind::Accent {
            inner,
            mark,
            stretch,
        } => {
            if *stretch {
                for i in 0..node.width {
                    buf.set(x + i, y, *mark, style);
                }
            } else {
                buf.set(x + node.width / 2, y, *mark, style);
            }
            render_inner(inner, buf, x, y + 1, style);
        }

        NodeKind::Limits { base, lower, upper } => {
            let max_h = upper.height.max(lower.height);
            render_inner(upper, buf, x, y + (max_h - upper.height), style);
            render_inner(base, buf, x, y + max_h, style);
            render_inner(lower, buf, x, y + max_h + base.height, style);
        }

        NodeKind::Sqrt { inner, index } => {
            buf.set(x + 1, y, '┌', style);
            for i in 2..node.width {
                buf.set(x + i, y, '─', style);
            }
            for row in 1..node.height {
                buf.set(x + 1, y + row, '│', style);
            }
            buf.set(x, y + node.height - 1, '╲', style);
            render_inner(inner, buf, x + 3, y + 1, style);

            if let Some(idx) = index {
                render_inner(idx, buf, x, y, style);
            }
        }

        NodeKind::Summation { inner } => {
            render_summation(inner.as_deref(), buf, x, y, node.height, style);
        }

        NodeKind::Product { inner } => {
            render_product(inner.as_deref(), buf, x, y, node.height, style);
        }

        NodeKind::Integral { inner } => {
            render_integral(inner.as_deref(), buf, x, y, node.height, style);
        }

        NodeKind::Matrix { .. } => {
            render_matrix(node, buf, x, y, style);
        }

        NodeKind::Neg { inner } => {
            buf.set(x, y + inner.baseline, '-', style);
            render_inner(inner, buf, x + 1, y, style);
        }

        NodeKind::Prime { base, count } => {
            render_inner(base, buf, x, y, style);
            for i in 0..*count {
                buf.set(x + base.width + i, y + base.baseline, '\'', style);
            }
        }

        NodeKind::Empty => {}
    }
}

fn resolve_delimiter_single(c: char) -> char {
    match c {
        '|' => '│',
        _ => c,
    }
}

fn resolve_delimiters(
    left: char,
    right: char,
    height: usize,
    fill: bool,
    baseline: usize,
) -> Vec<(char, char)> {
    let (tl, tr, bl, br, ml, mr) = match (left, right) {
        ('(', ')') => ('⎛', '⎞', '⎝', '⎠', '⎜', '⎟'),
        ('[', ']') => ('⎡', '⎤', '⎣', '⎦', '⎢', '⎥'),
        ('{', '}') => ('⎧', '⎫', '⎩', '⎭', '⎪', '⎪'),
        ('|', '|') => ('⎪', '⎪', '⎪', '⎪', '⎪', '⎪'),
        _ if fill => (left, right, left, right, left, right),
        _ => (left, right, left, right, '│', '│'),
    };

    let mut result = Vec::with_capacity(height);
    for row in 0..height {
        let (l, r) = if row == 0 {
            (tl, tr)
        } else if row == height - 1 {
            (bl, br)
        } else if left == '{' && row == baseline {
            ('⎨', '⎬')
        } else {
            (ml, mr)
        };
        result.push((l, r));
    }
    result
}

fn render_summation(
    inner: Option<&LayoutNode>,
    buf: &mut impl RenderTarget,
    x: usize,
    y: usize,
    h: usize,
    style: Style,
) {
    let inner = match inner {
        Some(i) => i,
        None => {
            buf.set(x, y, '━', style);
            buf.set(x + 1, y, '━', style);
            buf.set(x + 2, y, '━', style);
            buf.set(x + 3, y, '┓', style);
            buf.set(x, y + 1, '⟩', style);
            buf.set(x, y + 2, '━', style);
            buf.set(x + 1, y + 2, '━', style);
            buf.set(x + 2, y + 2, '━', style);
            buf.set(x + 3, y + 2, '┛', style);

            return;
        }
    };

    if inner.height <= 2 {
        let w_sigma = 4;
        buf.set(x, y, '━', style);
        buf.set(x + 1, y, '━', style);
        buf.set(x + 2, y, '━', style);
        buf.set(x + 3, y, '┓', style);
        buf.set(x, y + 1, '⟩', style);
        buf.set(x, y + 2, '━', style);
        buf.set(x + 1, y + 2, '━', style);
        buf.set(x + 2, y + 2, '━', style);
        buf.set(x + 3, y + 2, '┛', style);
        let inner_y = y + (h.saturating_sub(inner.height)) / 2;
        render_inner(inner, buf, x + w_sigma + 1, inner_y, style);
        return;
    }

    let w_sigma = ((1.5 * h as f32) as usize).max(h / 2 + 2);

    buf.fill_row(y, x, x + w_sigma - 1, '━', style);
    buf.set(x + w_sigma - 1, y, '┓', style);

    buf.fill_row(y + h - 1, x, x + w_sigma - 1, '━', style);
    buf.set(x + w_sigma - 1, y + h - 1, '┛', style);

    for r in 1..h - 1 {
        let d = r.min(h - 1 - r);
        let col = d.saturating_sub(1);

        let ch = if !h.is_power_of_two() && r == h / 2 {
            '⟩'
        } else if r < h / 2 {
            '╲'
        } else {
            '╱'
        };

        buf.set(x + col, y + r, ch, style);
    }

    render_inner(inner, buf, x + w_sigma + 1, y, style);
}

fn render_product(
    inner: Option<&LayoutNode>,
    buf: &mut impl RenderTarget,
    x: usize,
    y: usize,
    h: usize,
    style: Style,
) {
    let inner = match inner {
        Some(i) => i,
        None => {
            buf.set(x, y, '┳', style);
            buf.set(x + 1, y, '━', style);
            buf.set(x + 2, y, '┳', style);
            buf.set(x, y + 1, '┃', style);
            buf.set(x + 2, y + 1, '┃', style);
            return;
        }
    };

    let w_pi = if h <= 2 { 3 } else { (h / 2 + 2).max(3) };

    buf.set(x, y, '┳', style);
    if w_pi > 2 {
        buf.fill_row(y, x + 1, x + w_pi - 1, '━', style);
    }
    buf.set(x + w_pi - 1, y, '┳', style);

    for row in 1..h {
        buf.set(x, y + row, '┃', style);
        buf.set(x + w_pi - 1, y + row, '┃', style);
    }

    let inner_y = y + (h.saturating_sub(inner.height)) / 2;
    render_inner(inner, buf, x + w_pi + 1, inner_y, style);
}

fn render_integral(
    inner: Option<&LayoutNode>,
    buf: &mut impl RenderTarget,
    x: usize,
    y: usize,
    _h: usize,
    style: Style,
) {
    let inner = match inner {
        Some(i) => i,
        None => {
            buf.set(x, y, '⎛', style);
            buf.set(x, y + 1, '⎜', style);
            buf.set(x, y + 2, '⎠', style);
            return;
        }
    };

    if inner.height <= 3 {
        buf.set(x, y, '⎛', style);
        buf.set(x, y + 1, '⎜', style);
        buf.set(x, y + 2, '⎠', style);
        let inner_y = if inner.height == 1 { y + 1 } else { y };
        render_inner(inner, buf, x + 2, inner_y, style);
    } else {
        buf.set(x, y, '⎛', style);
        for row in 1..inner.height - 1 {
            buf.set(x, y + row, '⎜', style);
        }
        buf.set(x, y + inner.height - 1, '⎠', style);
        render_inner(inner, buf, x + 2, y, style);
    }
}

fn render_matrix(
    node: &LayoutNode,
    buf: &mut impl RenderTarget,
    x: usize,
    y: usize,
    inherited: Style,
) {
    let NodeKind::Matrix { name: _, rows } = &node.kind else {
        return;
    };

    if rows.is_empty() || rows[0].is_empty() {
        return;
    }

    let num_rows = rows.len();

    let mut row_max_depths = vec![0; num_rows];
    let mut row_max_baselines = vec![0; num_rows];
    let mut max_item_width = 0;

    for (i, row) in rows.iter().enumerate() {
        let mut max_b = 0;
        let mut max_d = 0;
        for item in row {
            max_item_width = max_item_width.max(item.width);
            max_b = max_b.max(item.baseline);
            max_d = max_d.max(item.height.saturating_sub(item.baseline));
        }
        row_max_baselines[i] = max_b;
        row_max_depths[i] = max_d;
    }

    let cell_width = max_item_width;
    let mut cell_height = 0;
    for i in 0..num_rows {
        let row_content_height = row_max_baselines[i] + row_max_depths[i];
        cell_height = cell_height.max(row_content_height);
    }

    let row_padding = 1;
    cell_height = cell_height.max(1);

    let active_cell_height = if num_rows > 1 {
        cell_height + row_padding
    } else {
        cell_height
    };

    let hspacing = 4;

    let inner_x = x;
    let inner_y = y;

    for (i, row) in rows.iter().enumerate() {
        let row_content_height = row_max_baselines[i] + row_max_depths[i];
        let row_padding_top = (active_cell_height - row_content_height) / 2;
        let row_cell_baseline = row_padding_top + row_max_baselines[i];

        for (j, item) in row.iter().enumerate() {
            let cell_x = inner_x + j * (cell_width + hspacing);
            let cell_y = inner_y + i * active_cell_height;

            let item_x_in_cell = (cell_width - item.width) / 2;
            let item_y_in_cell = row_cell_baseline - item.baseline;

            render_inner(
                item,
                buf,
                cell_x + item_x_in_cell,
                cell_y + item_y_in_cell,
                inherited,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::render_node;
    use crate::backend::RenderTarget;
    use crate::style::{Color, Style};

    struct StyleBuf {
        cells: Vec<(char, Style)>,
        width: usize,
    }

    impl StyleBuf {
        fn new(width: usize, height: usize) -> Self {
            Self {
                cells: vec![(' ', Style::new()); width * height],
                width,
            }
        }
    }

    impl RenderTarget for StyleBuf {
        fn set(&mut self, x: usize, y: usize, ch: char, style: Style) {
            let index = y * self.width + x;
            self.cells[index] = (ch, style);
        }

        fn fill_row(&mut self, y: usize, x_start: usize, x_end: usize, ch: char, style: Style) {
            for x in x_start..x_end {
                self.set(x, y, ch, style);
            }
        }
    }

    #[test]
    fn current_color_reaches_generated_renderer_cells() {
        for input in [
            r"\color{red}{x+y}",
            r"\color{red}{-x}",
            r"\color{red}{x'}",
            r"\color{red}{\int_a^b{f'\left(x\right) dx}}",
        ] {
            let tree = crate::layout(input).unwrap();
            let mut buf = StyleBuf::new(tree.width, tree.height);
            render_node(&tree, &mut buf, 0, 0);

            let uncolored: String = buf
                .cells
                .iter()
                .filter(|(ch, style)| *ch != ' ' && style.fg_color() != Color::Red)
                .map(|(ch, _)| *ch)
                .collect();

            assert!(
                uncolored.is_empty(),
                "expected all cells in {input:?} to be red, uncolored: {uncolored:?}"
            );
        }
    }
}
