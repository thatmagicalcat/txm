use std::collections::HashMap;
use std::fmt::Debug;

use crate::UNIFORM_FRACTION_HEIGHT;
use crate::layout::RenderNode;

#[derive(Debug, Default, Clone, Copy)]
pub struct RenderCtx {
    pub depth: usize,
}

pub trait Glyph: Debug + Send + Sync {
    fn required_args(&self) -> usize {
        0
    }

    fn has_optional(&self) -> bool {
        false
    }

    fn has_limits(&self) -> bool {
        false
    }

    fn render(&self, args: &[RenderNode], _opts: &[RenderNode], _ctx: &mut RenderCtx)
    -> RenderNode;
}

pub struct SymbolRegistry {
    map: HashMap<String, Box<dyn Glyph>>,
}

impl SymbolRegistry {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: impl Into<String>, glyph: impl Glyph + 'static) {
        self.map.insert(name.into(), Box::new(glyph));
    }

    pub fn get(&self, name: &str) -> Option<&dyn Glyph> {
        self.map.get(name).map(|g| g.as_ref())
    }
}

#[derive(Debug)]
pub struct LimitGlyph;

impl Glyph for LimitGlyph {
    fn render(
        &self,
        _args: &[RenderNode],
        _opts: &[RenderNode],
        _ctx: &mut RenderCtx,
    ) -> RenderNode {
        RenderNode::from_str("lim")
    }

    fn required_args(&self) -> usize {
        0
    }

    fn has_limits(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct UnicodeGlyph(pub char);

impl Glyph for UnicodeGlyph {
    fn render(
        &self,
        _args: &[RenderNode],
        _opts: &[RenderNode],
        _ctx: &mut RenderCtx,
    ) -> RenderNode {
        RenderNode::from_char(self.0)
    }
}

#[derive(Debug)]
pub struct TextGlyph(pub &'static str);

impl Glyph for TextGlyph {
    fn render(
        &self,
        _args: &[RenderNode],
        _opts: &[RenderNode],
        _ctx: &mut RenderCtx,
    ) -> RenderNode {
        RenderNode::from_str(self.0)
    }
}

#[derive(Debug)]
pub struct BinomGlyph;

impl Glyph for BinomGlyph {
    fn required_args(&self) -> usize {
        2
    }

    fn render(
        &self,
        args: &[RenderNode],
        _opts: &[RenderNode],
        _ctx: &mut RenderCtx,
    ) -> RenderNode {
        let inner = RenderNode::vstack(&args[0], &args[1], ' ', 0, UNIFORM_FRACTION_HEIGHT);
        RenderNode::stretchy_delim(&inner, '(', ')', false)
    }
}

#[derive(Debug)]
pub struct FracGlyph;

impl Glyph for FracGlyph {
    fn required_args(&self) -> usize {
        2
    }

    fn render(&self, args: &[RenderNode], _opts: &[RenderNode], ctx: &mut RenderCtx) -> RenderNode {
        let pad = if ctx.depth == 0 { 1 } else { 0 };
        RenderNode::vstack(&args[0], &args[1], '─', pad, UNIFORM_FRACTION_HEIGHT)
    }
}

#[derive(Debug)]
pub struct SqrtGlyph;

impl Glyph for SqrtGlyph {
    fn required_args(&self) -> usize {
        1
    }

    fn has_optional(&self) -> bool {
        true
    }

    fn render(&self, args: &[RenderNode], opts: &[RenderNode], _ctx: &mut RenderCtx) -> RenderNode {
        let radicand = RenderNode::sqrt_inner(&args[0]);
        if let Some(root) = opts.first() {
            let w = root.width + radicand.width;
            let h = root.height.max(radicand.height);
            let mut data = vec![' '; w * h];

            root.blit_into(&mut data, w, 0, 0);
            radicand.blit_into(&mut data, w, root.width, 0);

            RenderNode {
                width: w,
                height: h,
                baseline: radicand.baseline,
                data,
            }
        } else {
            radicand
        }
    }
}

#[derive(Debug)]
pub struct IntegralGlyph;

impl Glyph for IntegralGlyph {
    fn has_limits(&self) -> bool {
        true
    }

    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[RenderNode],
        _opts: &[RenderNode],
        _ctx: &mut RenderCtx,
    ) -> RenderNode {
        // Render a fixed-length integral symbol
        if args.is_empty() {
            RenderNode {
                width: 2, // symbol + space
                height: 3,
                baseline: 1,
                data: vec!['⎛', ' ', '⎟', ' ', '⎠', ' '],
            }
        } else {
            // no stretching required
            if args[0].height <= 3 {
                let w = args[0].width + 2; // symbol + space
                let mut data = vec![' '; w * 3];

                data[0] = '⎛';
                data[w] = '⎟';
                data[2 * w] = '⎠';

                // center one-liner expressions
                let y = if args[0].height == 1 { 1 } else { 0 };
                args[0].blit_into(&mut data, w, 2, y);

                return RenderNode {
                    width: w,
                    height: 3,
                    baseline: 1,
                    data,
                };
            }

            RenderNode::stretchy_delim_left(&args[0], '⎛', '⎟', '⎠')
        }
    }
}

/// A font-alphabet command (`\mathbf`, `\mathbb`, `\mathrm`, ...): takes one
/// argument and remaps each character of it through `.0`. Characters with no
/// variant in the target alphabet (spaces, operators, digits in italic) pass
/// through unchanged.
#[derive(Debug)]
pub struct AlphabetGlyph(pub fn(char) -> char);

impl Glyph for AlphabetGlyph {
    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[RenderNode],
        _opts: &[RenderNode],
        _ctx: &mut RenderCtx,
    ) -> RenderNode {
        let src = &args[0];
        RenderNode {
            width: src.width,
            height: src.height,
            baseline: src.baseline,
            data: src.data.iter().map(|&c| (self.0)(c)).collect(),
        }
    }
}

fn shift(c: char, base: u32, off: u32) -> char {
    char::from_u32(base + off).unwrap_or(c)
}

/// Mathematical bold (𝐀-𝐳, 𝟎-𝟗).
pub fn to_bold(c: char) -> char {
    match c {
        'A'..='Z' => shift(c, 0x1D400, c as u32 - 'A' as u32),
        'a'..='z' => shift(c, 0x1D41A, c as u32 - 'a' as u32),
        '0'..='9' => shift(c, 0x1D7CE, c as u32 - '0' as u32),
        _ => c,
    }
}

/// Blackboard bold / double-struck (ℝ, ℍ, ℂ, ...).
pub fn to_bb(c: char) -> char {
    match c {
        // Letters that live in the Letterlike Symbols block, not the contiguous run.
        'C' => 'ℂ',
        'H' => 'ℍ',
        'N' => 'ℕ',
        'P' => 'ℙ',
        'Q' => 'ℚ',
        'R' => 'ℝ',
        'Z' => 'ℤ',
        'A'..='Z' => shift(c, 0x1D538, c as u32 - 'A' as u32),
        'a'..='z' => shift(c, 0x1D552, c as u32 - 'a' as u32),
        '0'..='9' => shift(c, 0x1D7D8, c as u32 - '0' as u32),
        _ => c,
    }
}

/// Upright roman (`\mathrm`, `\mathup`): terminal glyphs are already upright,
/// so this is the identity and simply lets the argument render normally.
pub fn to_upright(c: char) -> char {
    c
}

/// Mathematical italic (𝐴-𝑧).
pub fn to_italic(c: char) -> char {
    match c {
        'h' => 'ℎ', // U+1D455 is reserved; Planck constant stands in.
        'A'..='Z' => shift(c, 0x1D434, c as u32 - 'A' as u32),
        'a'..='z' => shift(c, 0x1D44E, c as u32 - 'a' as u32),
        _ => c,
    }
}

/// Sans-serif (𝖠-𝗓, 𝟢-𝟫).
pub fn to_sans(c: char) -> char {
    match c {
        'A'..='Z' => shift(c, 0x1D5A0, c as u32 - 'A' as u32),
        'a'..='z' => shift(c, 0x1D5BA, c as u32 - 'a' as u32),
        '0'..='9' => shift(c, 0x1D7E2, c as u32 - '0' as u32),
        _ => c,
    }
}

#[derive(Debug)]
pub struct AbsGlyph;

impl Glyph for AbsGlyph {
    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[RenderNode],
        _opts: &[RenderNode],
        _ctx: &mut RenderCtx,
    ) -> RenderNode {
        RenderNode::abs(&args[0])
    }
}

/// An accent command (`\hat`, `\tilde`, `\bar`, `\vec`, `\overline`, ...):
/// takes one argument and draws `mark` above it. `stretch` spans the mark
/// across the whole width (wide accents); otherwise it is centred.
#[derive(Debug)]
pub struct AccentGlyph {
    pub mark: char,
    pub stretch: bool,
}

impl Glyph for AccentGlyph {
    fn required_args(&self) -> usize {
        1
    }

    fn render(
        &self,
        args: &[RenderNode],
        _opts: &[RenderNode],
        _ctx: &mut RenderCtx,
    ) -> RenderNode {
        RenderNode::accent(&args[0], self.mark, self.stretch)
    }
}
