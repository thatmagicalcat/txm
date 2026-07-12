use std::fmt;

use crate::{UNIFORM_FRACTION_HEIGHT, buffer::RenderBuffer, error::ParseError, glyph::RenderCtx};

#[derive(Clone)]
pub struct RenderNode {
    pub width: usize,
    pub height: usize,
    pub baseline: usize,
    pub buffer: RenderBuffer,
}

impl RenderNode {
    pub fn new(width: usize, height: usize, baseline: usize) -> Self {
        Self {
            width,
            height,
            baseline,
            buffer: RenderBuffer::new(width, height),
        }
    }

    pub fn from_char(c: char) -> Self {
        Self {
            width: 1,
            height: 1,
            baseline: 0,
            buffer: vec![c].into(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let width = chars.len();
        Self {
            width,
            height: 1,
            baseline: 0,
            buffer: chars.into(),
        }
    }

    pub fn blit_into(&self, target: &mut RenderBuffer, target_width: usize, x: usize, y: usize) {
        for row in 0..self.height {
            if y + row >= target.len() / target_width {
                break;
            }

            let src_start = row * self.width;
            let src_end = src_start + self.width;
            let dst_start = (y + row) * target_width + x;

            if dst_start + self.width <= target.len() {
                target[dst_start..dst_start + self.width]
                    .copy_from_slice(&self.buffer[src_start..src_end]);

                #[cfg(feature = "fancy")]
                target.style_mut()[dst_start..dst_start + self.width]
                    .copy_from_slice(&self.buffer.style_ref()[src_start..src_end]);
            }
        }
    }

    pub fn hstack(nodes: &[Self], spacing: usize) -> Self {
        if nodes.is_empty() {
            return Self::new(0, 0, 0);
        }

        if nodes.len() == 1 {
            return nodes[0].clone();
        }

        let baseline = nodes.iter().map(|n| n.baseline).max().unwrap_or(0);
        let height = nodes
            .iter()
            .map(|n| {
                let below = n.height.saturating_sub(n.baseline);
                baseline + below
            })
            .max()
            .unwrap_or(0);

        let total_width: usize =
            nodes.iter().map(|n| n.width).sum::<usize>() + spacing * (nodes.len() - 1);
        let mut buffer = RenderBuffer::new(total_width, height);

        let mut x = 0;
        for node in nodes {
            let y = baseline.saturating_sub(node.baseline);
            node.blit_into(&mut buffer, total_width, x, y);
            x += node.width + spacing;
        }

        Self {
            width: total_width,
            height,
            baseline,
            buffer,
        }
    }

    /// if uniform_height is true, the top and bottom elements will take the same height
    pub fn vstack(
        top: &Self,
        bottom: &Self,
        line_char: char,
        pad: usize,
        #[allow(unused)] ctx: &RenderCtx,
    ) -> Self {
        let max_height = top.height.max(bottom.height);
        let inner_w = top.width.max(bottom.width);
        let w = inner_w + 2 * pad;
        let h = if UNIFORM_FRACTION_HEIGHT {
            2 * max_height
        } else {
            top.height + bottom.height
        } + 1;

        let mut buffer = cfg_select! {
            feature = "fancy" => RenderBuffer::new_styled(w, h, ctx.current_style),
            _                 => RenderBuffer::new(w, h)
        };

        let top_x = pad + (inner_w.saturating_sub(top.width)) / 2;
        let bot_x = pad + (inner_w.saturating_sub(bottom.width)) / 2;
        let baseline = if UNIFORM_FRACTION_HEIGHT {
            max_height
        } else {
            top.height
        };

        if UNIFORM_FRACTION_HEIGHT {
            // let y = (max_height - top.height) / 2;
            let y = baseline - top.height;
            top.blit_into(&mut buffer, w, top_x, y);

            bottom.blit_into(&mut buffer, w, bot_x, baseline + 1);
        } else {
            top.blit_into(&mut buffer, w, top_x, 0);
            let bot_x = pad + (inner_w.saturating_sub(bottom.width)) / 2;
            bottom.blit_into(&mut buffer, w, bot_x, baseline + 1);
        }

        for x in 0..w {
            buffer.data_mut()[baseline * w + x] = line_char;
        }

        Self {
            width: w,
            height: h,
            baseline,
            buffer,
        }
    }

    pub fn infix(lhs: &Self, op: char, rhs: &Self) -> Self {
        let baseline = lhs.baseline.max(rhs.baseline);
        let lhs_y = baseline - lhs.baseline;
        let rhs_y = baseline - rhs.baseline;

        let width = lhs.width + 3 + rhs.width;
        let height = (lhs_y + lhs.height).max(rhs_y + rhs.height);

        let mut buffer = RenderBuffer::new(width, height);

        lhs.blit_into(&mut buffer, width, 0, lhs_y);
        buffer[baseline * width + lhs.width + 1] = op;
        rhs.blit_into(&mut buffer, width, lhs.width + 3, rhs_y);

        Self {
            width,
            height,
            baseline,
            buffer,
        }
    }

    pub fn superscript(base: &Self, exp: &Self) -> Self {
        // check if we can use special subscript unicode chars for this superscript
        if exp.height == 1 {
            let mut converted = Vec::with_capacity(exp.buffer.len());
            let mut all_convertible = true;

            for &c in exp.buffer.data_ref() {
                if let Some(sup_c) = to_superscript_char(c) {
                    converted.push(sup_c);
                } else {
                    all_convertible = false;
                    break;
                }
            }

            if all_convertible {
                let width = base.width + converted.len();
                let mut buffer = RenderBuffer::new(width, base.height);
                base.blit_into(&mut buffer, width, 0, 0);

                let target_row = 0;
                let dst_start = target_row * width + base.width;
                let dst_end = dst_start + converted.len();
                buffer[dst_start..dst_end].copy_from_slice(&converted);

                return Self {
                    width,
                    height: base.height,
                    baseline: base.baseline,
                    buffer,
                };
            }
        }

        let height = exp.height + base.height;
        let width = exp.width + base.width;
        let baseline = base.baseline + exp.height;

        let mut buffer = RenderBuffer::new(width, height);
        base.blit_into(&mut buffer, width, 0, exp.height);
        exp.blit_into(&mut buffer, width, base.width, 0);

        Self {
            width,
            height,
            baseline,
            buffer,
        }
    }

    pub fn subscript(base: &Self, sub: &Self) -> Self {
        // check if we can use special subscript unicode chars for this subscript
        if sub.height == 1 {
            let mut converted = Vec::with_capacity(sub.buffer.len());
            let mut all_convertible = true;

            for c in sub.buffer.data_ref() {
                if let Some(sub_c) = to_subscript_char(*c) {
                    converted.push(sub_c);
                } else {
                    all_convertible = false;
                    break;
                }
            }

            if all_convertible {
                let width = base.width + converted.len();

                let mut buffer = RenderBuffer::new(width, base.height);
                base.blit_into(&mut buffer, width, 0, 0);

                let target_row = base.baseline;
                let dst_start = target_row * width + base.width;
                let dst_end = dst_start + converted.len();
                buffer[dst_start..dst_end].copy_from_slice(&converted);

                return Self {
                    width,
                    height: base.height,
                    baseline: base.baseline,
                    buffer,
                };
            }
        }

        // fallback

        let sub_y = base.baseline + 1;
        let height = (sub_y + sub.height).max(base.height);
        let width = base.width + sub.width;

        let mut buffer = RenderBuffer::new(width, height);
        base.blit_into(&mut buffer, width, 0, 0);
        sub.blit_into(&mut buffer, width, base.width, sub_y);

        Self {
            width,
            height,
            baseline: base.baseline,
            buffer,
        }
    }

    pub fn both_scripts(base: &Self, sub: &Self, sup: &Self) -> Self {
        let sup_h = sup.height;
        let sub_h = sub.height;
        let sub_y = base.baseline + 1;
        let height = sup_h + sub_h + base.height;
        let width = base.width + sub.width.max(sup.width);
        let baseline = base.baseline + sup_h;

        let mut buffer = RenderBuffer::new(width, height);
        base.blit_into(&mut buffer, width, 0, sup_h);
        sup.blit_into(&mut buffer, width, base.width, 0);
        sub.blit_into(&mut buffer, width, base.width, sub_y + sup_h);

        Self {
            width,
            height,
            baseline,
            buffer,
        }
    }

    pub fn prime_suffix(base: &Self, n: usize) -> Self {
        let s: String = std::iter::repeat_n('\'', n).collect();
        let p = Self::from_str(&s);
        Self::hstack(&[base.clone(), p], 0)
    }

    #[cfg(feature = "fancy")]
    pub fn apply_style(&mut self, style: crate::style::Style) {
        for s in self.buffer.style_mut().iter_mut() {
            *s = s.merge(style);
        }
    }

    /// If fill is true, the middle line will also use left and right chars
    pub fn stretchy_delim(inner: &Self, left: char, right: char, fill: bool) -> Self {
        // one-liner expressions
        if inner.height <= 1 {
            let mut result = Self::new(inner.width + 2, 1, 0);
            result.buffer[0] = left;
            inner.blit_into(&mut result.buffer, result.width, 1, 0);
            result.buffer[result.width - 1] = right;
            return result;
        }

        let h = inner.height;
        let w = inner.width + 4;
        let baseline = inner.baseline;

        let mut buffer = RenderBuffer::new(w, h);
        inner.blit_into(&mut buffer, w, 2, 0);

        let (tl, tr, bl, br, mid_l, mid_r) = match (left, right) {
            ('(', ')') => ('⎛', '⎞', '⎝', '⎠', '⎜', '⎟'),
            ('[', ']') => ('⎡', '⎤', '⎣', '⎦', '⎢', '⎥'),
            ('{', '}') => ('⎧', '⎫', '⎩', '⎭', '⎪', '⎪'),
            ('|', '|') => ('⎪', '⎪', '⎪', '⎪', '⎪', '⎪'),

            _ if fill => (left, right, left, right, left, right),
            _ => (left, right, left, right, '│', '│'),
        };

        buffer[0] = tl;
        buffer[w - 1] = tr;
        buffer[(h - 1) * w] = bl;
        buffer[(h - 1) * w + w - 1] = br;

        for y in 1..h - 1 {
            if left == '{' && y == baseline {
                buffer[y * w] = '⎨';
            } else {
                buffer[y * w] = mid_l;
            }
            if right == '}' && y == baseline {
                buffer[y * w + w - 1] = '⎬';
            } else {
                buffer[y * w + w - 1] = mid_r;
            }
        }

        Self {
            width: w,
            height: h,
            baseline,
            buffer,
        }
    }

    pub fn stretchy_delim_left(inner: &Self, top: char, middle: char, bottom: char) -> Self {
        let h = inner.height;
        let w = inner.width + 2;
        let baseline = inner.baseline;

        let mut buffer = RenderBuffer::new(w, h);
        inner.blit_into(&mut buffer, w, 2, 0);

        let mid_l = middle;

        buffer[0] = top;
        buffer[(h - 1) * w] = bottom;

        for y in 1..h - 1 {
            buffer[y * w] = mid_l;
        }

        Self {
            width: w,
            height: h,
            baseline,
            buffer,
        }
    }

    #[allow(dead_code)]
    pub fn stretchy_delim_right(inner: &Self, top: char, middle: char, bottom: char) -> Self {
        let h = inner.height;
        let w = inner.width + 2;
        let baseline = inner.baseline;

        let mut buffer = RenderBuffer::new(w, h);
        inner.blit_into(&mut buffer, w, 0, 0);

        buffer[w - 1] = top;
        buffer[(h - 1) * w + w - 1] = bottom;

        for y in 1..h - 1 {
            buffer[y * w + w - 1] = middle;
        }

        Self {
            width: w,
            height: h,
            baseline,
            buffer,
        }
    }

    pub fn sqrt_inner(inner: &Self) -> Self {
        let h = inner.height + 1;
        let w = inner.width + 3;
        let baseline = inner.baseline + 1;

        let mut buffer = RenderBuffer::new(w, h);
        inner.blit_into(&mut buffer, w, 3, 1);

        buffer[1] = '┌';

        for item in buffer.data_mut().iter_mut().take(w).skip(2) {
            *item = '─';
        }

        for y in 1..h {
            buffer[y * w + 1] = '│';
        }

        buffer[(h - 1) * w] = '╲';

        Self {
            width: w,
            height: h,
            baseline,
            buffer,
        }
    }

    pub fn limits(base: &Self, lower: &Self, upper: &Self) -> Self {
        let max_h = upper.height.max(lower.height);
        let w = base.width.max(lower.width).max(upper.width) + 1;
        // let h = upper.height + base.height + lower.height;
        let h = base.height + 2 * max_h;

        let mut buffer = RenderBuffer::new(w, h);

        // let ux = (w.saturating_sub(upper.width)) / 2;
        upper.blit_into(&mut buffer, w, 0, max_h - upper.height);

        // let bx = (w.saturating_sub(base.width)) / 2;
        base.blit_into(&mut buffer, w, 0, max_h);

        // let lx = (w.saturating_sub(lower.width)) / 2;
        lower.blit_into(&mut buffer, w, 0, max_h + base.height);

        Self {
            width: w,
            height: h,
            baseline: max_h + base.baseline,
            buffer,
        }
    }

    pub fn matrix(name: &str, rendered_rows: &[Vec<RenderNode>]) -> Result<RenderNode, ParseError> {
        let (left_delim, right_delim) = match name {
            "matrix" => (' ', ' '),
            "bmatrix" => ('[', ']'),
            "pmatrix" => ('(', ')'),
            _ => return Err(ParseError(format!("unknown matrix environment: {name}"))),
        };

        let num_rows = rendered_rows.len();
        if num_rows == 0 || rendered_rows[0].is_empty() {
            return Ok(Self::new(0, 0, 0));
        }

        let num_cols = rendered_rows[0].len();

        // depth = height - baseline
        let mut row_max_depths = vec![0; num_rows];
        let mut row_max_baselines = vec![0; num_rows];
        let mut max_item_width = 0;

        // analyze dimensions per row to preserve baselines
        for (i, row) in rendered_rows.iter().enumerate() {
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

        // define a uniform cell size for the matrix grid
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
        let vspacing = 1;

        let mut matrix_layout_height = num_rows * cell_height + (num_rows - 1) * vspacing;
        let matrix_layout_width = num_cols * cell_width + (num_cols - 1) * hspacing;

        // make the total height odd so the baseline sits right in the center
        if matrix_layout_height.is_multiple_of(2) {
            matrix_layout_height += 1;
        }

        let baseline = matrix_layout_height / 2;
        let mut buffer = RenderBuffer::new(matrix_layout_width, matrix_layout_height);

        for (i, row) in rendered_rows.iter().enumerate() {
            let row_content_height = row_max_baselines[i] + row_max_depths[i];
            // let row_padding_top = (cell_height - row_content_height) / 2;
            let row_padding_top = (active_cell_height - row_content_height) / 2;
            let row_cell_baseline = row_padding_top + row_max_baselines[i];

            for (j, item) in row.iter().enumerate() {
                let cell_x = j * (cell_width + hspacing);
                let cell_y = i * active_cell_height;

                // horizontally center the item in the cell
                let item_x_in_cell = (cell_width - item.width) / 2;
                // vertically align the item's baseline with row's
                let item_y_in_cell = row_cell_baseline - item.baseline;

                let center_x = cell_x + item_x_in_cell;
                let center_y = cell_y + item_y_in_cell;

                item.blit_into(&mut buffer, matrix_layout_width, center_x, center_y);
            }
        }

        let matrix = Self {
            width: matrix_layout_width,
            height: matrix_layout_height,
            baseline,
            buffer,
        };

        Ok(RenderNode::stretchy_delim(
            &matrix,
            left_delim,
            right_delim,
            true,
        ))
    }

    /// Place an accent above `inner`: a single mark centred over the argument
    /// (`\hat`, `\tilde`, `\vec`, ...) or, when `stretch` is set, a mark run
    /// spanning its full width (`\overline`, `\widehat`, ...). The accent adds
    /// one row on top and the baseline follows the argument down.
    pub fn accent(inner: &Self, mark: char, stretch: bool) -> Self {
        let width = inner.width.max(1);
        let height = inner.height + 1;
        let mut buffer = RenderBuffer::new(width, height);

        if stretch {
            for cell in buffer.data_mut().iter_mut().take(width) {
                *cell = mark;
            }
        } else {
            buffer[width / 2] = mark;
        }

        inner.blit_into(&mut buffer, width, 0, 1);

        Self {
            width,
            height,
            baseline: inner.baseline + 1,
            buffer,
        }
    }

    #[allow(unused)]
    #[cfg(feature = "fancy")]
    pub fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        let mut prev = crate::style::Style::new();

        for y in 0..self.height {
            let row_start = y * self.width;
            for x in 0..self.width {
                let i = row_start + x;
                let style = self.buffer.styles[i];

                if style != prev {
                    if !prev.is_empty() {
                        f.write_str("\x1b[0m")?;
                    }
                    if !style.is_empty() {
                        style.write_ansi_prefix(f)?;
                    }
                    prev = style;
                }

                f.write_char(self.buffer.data[i])?;
            }

            f.write_char('\n')?;
        }

        if !prev.is_empty() {
            f.write_str("\x1b[0m")?;
        }

        Ok(())
    }

    #[cfg(feature = "fancy")]
    pub fn write_ansi_boxed(&self, f: &mut impl fmt::Write) -> fmt::Result {
        use unicode_width::UnicodeWidthChar;

        let mut row_widths = vec![0; self.height];
        let mut max_width = 0;

        for y in 0..self.height {
            let row_start = y * self.width;
            let mut row_width = 0;
            for x in 0..self.width {
                let ch = self.buffer.data[row_start + x];
                row_width += ch.width().unwrap_or(0);
            }
            row_widths[y] = row_width;
            if row_width > max_width {
                max_width = row_width;
            }
        }

        let border = "─".repeat(max_width + 2);
        f.write_str("┌")?;
        f.write_str(&border)?;
        f.write_str("┐\n")?;

        for y in 0..self.height {
            f.write_str("│ ")?; // Left border

            let row_start = y * self.width;
            let mut prev = crate::style::Style::new();

            for x in 0..self.width {
                let i = row_start + x;
                let style = self.buffer.styles[i];

                if style != prev {
                    if !prev.is_empty() {
                        f.write_str("\x1b[0m")?;
                    }
                    if !style.is_empty() {
                        style.write_ansi_prefix(f)?;
                    }
                    prev = style;
                }

                f.write_char(self.buffer.data[i])?;
            }

            if !prev.is_empty() {
                f.write_str("\x1b[0m")?;
            }

            let padding = max_width - row_widths[y];
            if padding > 0 {
                write!(f, "{:padding$}", "", padding = padding)?;
            }

            f.write_str(" │\n")?; // Right border
        }

        f.write_str("└")?;
        f.write_str(&border)?;
        f.write_str("┘")?;

        Ok(())
    }
}

impl fmt::Display for RenderNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..self.height {
            let start = y * self.width;
            let end = start + self.width;

            for &c in &self.buffer[start..end] {
                write!(f, "{c}")?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}

#[rustfmt::skip]
fn to_superscript_char(c: char) -> Option<char> {
    Some (match c {
        '0' => '⁰', '1' => '¹', '2' => '²', '3' => '³', '4' => '⁴',
        '5' => '⁵', '6' => '⁶', '7' => '⁷', '8' => '⁸', '9' => '⁹',
        'a' => 'ᵃ', 'b' => 'ᵇ', 'c' => 'ᶜ', 'd' => 'ᵈ', 'e' => 'ᵉ',
        'f' => 'ᶠ', 'g' => 'ᵍ', 'h' => 'ʰ', 'i' => 'ⁱ', 'j' => 'ʲ',
        'k' => 'ᵏ', 'l' => 'ˡ', 'm' => 'ᵐ', 'n' => 'ⁿ', 'o' => 'ᵒ',
        'p' => 'ᵖ', 'r' => 'ʳ', 's' => 'ˢ', 't' => 'ᵗ', 'u' => 'ᵘ',
        'v' => 'ᵛ', 'w' => 'ʷ', 'x' => 'ˣ', 'y' => 'ʸ', 'z' => 'ᶻ',
        'A' => 'ᴬ', 'B' => 'ᴮ', 'D' => 'ᴰ', 'E' => 'ᴱ', 'G' => 'ᴳ',
        'H' => 'ᴴ', 'I' => 'ᴵ', 'J' => 'ᴶ', 'M' => 'ᴹ', 'N' => 'ᴺ',
        'O' => 'ᴼ', 'P' => 'ᴾ', 'R' => 'ᴿ', 'T' => 'ᵀ', 'U' => 'ᵁ',
        'W' => 'ᵂ', '+' => '⁺', '-' => '⁻', '=' => '⁼', '(' => '⁽',
        ')' => '⁾',

        _ => return None,
    })
}

#[rustfmt::skip]
fn to_subscript_char(c: char) -> Option<char> {
    Some(match c {
        '0' => '₀', '1' => '₁', '2' => '₂', '3' => '₃', '4' => '₄',
        '5' => '₅', '6' => '₆', '7' => '₇', '8' => '₈', '9' => '₉',
        'a' => 'ₐ', 'e' => 'ₑ', 'h' => 'ₕ', 'i' => 'ᵢ', 'j' => 'ⱼ',
        'k' => 'ₖ', 'l' => 'ₗ', 'm' => 'ₘ', 'n' => 'ₙ', 'o' => 'ₒ',
        'p' => 'ₚ', 'r' => 'ᵣ', 's' => 'ₛ', 't' => 'ₜ', 'u' => 'ᵤ',
        'v' => 'ᵥ', 'x' => 'ₓ', 'y' => 'ᵧ', '+' => '₊', '-' => '₋', '=' => '₌',
        '(' => '₍', ')' => '₎',

        _ => return None,
    })
}
