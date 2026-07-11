use crate::glyph::{
    AbsGlyph, AccentGlyph, AlphabetGlyph, BinomGlyph, FracGlyph, IntegralGlyph, LimitGlyph,
    RenderCtx, SqrtGlyph, SymbolRegistry, TextGlyph, UnicodeGlyph, to_bb, to_bold, to_italic,
    to_sans, to_upright,
};
use crate::parser::Parser;
use crate::render::render as render_expr;
use crate::token::tokenize;

use std::sync::OnceLock;

mod ast;
mod error;
mod glyph;
mod layout;
mod parser;
mod render;
mod token;

pub use error::ParseError;

const UNIFORM_FRACTION_HEIGHT: bool = false;
const COMPACT_SIMPLE_FRACTIONAL_EXPONENTS: bool = false;

/// Renders a math expression to plain text lines.
///
/// On success, the returned string is newline-terminated and contains one line
/// per rendered row. Returns `ParseError` for lexer, parser, or render errors.
pub fn render(input: &str) -> Result<String, ParseError> {
    let tokens = tokenize(input)?;
    let reg = registry();
    let mut parser = Parser::new(&tokens, reg);
    let expr = parser.parse_expr()?;
    let mut ctx = RenderCtx::default();
    let layout = render_expr(&expr, reg, &mut ctx)?;

    Ok(layout.to_string())
}

fn registry() -> &'static SymbolRegistry {
    static REGISTRY: OnceLock<SymbolRegistry> = OnceLock::new();
    REGISTRY.get_or_init(build_registry)
}

fn build_registry() -> SymbolRegistry {
    let mut r = SymbolRegistry::new();

    for (cmd, ch) in [
        ("alpha", 'α'),
        ("beta", 'β'),
        ("gamma", 'γ'),
        ("delta", 'δ'),
        ("epsilon", 'ε'),
        ("zeta", 'ζ'),
        ("eta", 'η'),
        ("theta", 'θ'),
        ("iota", 'ι'),
        ("kappa", 'κ'),
        ("lambda", 'λ'),
        ("mu", 'μ'),
        ("nu", 'ν'),
        ("xi", 'ξ'),
        ("omicron", 'ο'),
        ("pi", 'π'),
        ("rho", 'ρ'),
        ("sigma", 'σ'),
        ("tau", 'τ'),
        ("upsilon", 'υ'),
        ("phi", 'φ'),
        ("chi", 'χ'),
        ("psi", 'ψ'),
        ("omega", 'ω'),
    ] {
        r.register(cmd, UnicodeGlyph(ch));
    }

    for (cmd, ch) in [
        ("Gamma", 'Γ'),
        ("Delta", 'Δ'),
        ("Theta", 'Θ'),
        ("Lambda", 'Λ'),
        ("Xi", 'Ξ'),
        ("Pi", 'Π'),
        ("Sigma", 'Σ'),
        ("Phi", 'Φ'),
        ("Psi", 'Ψ'),
        ("Omega", 'Ω'),
    ] {
        r.register(cmd, UnicodeGlyph(ch));
    }

    for name in &[
        "sin", "cos", "tan", "cot", "sec", "csc", "arcsin", "arccos", "arctan", "sinh", "cosh",
        "tanh", "log", "ln", "lg", "det", "dim", "hom", "ker", "exp", "deg", "gcd", "lcm", "lim",
        "sup", "inf", "max", "min", "arg", "Pr", "mod", "adj",
    ] {
        r.register(*name, TextGlyph(name));
    }

    r.register("binom", BinomGlyph);
    r.register("frac", FracGlyph);
    r.register("sqrt", SqrtGlyph);
    r.register("lim", LimitGlyph);
    r.register("int", IntegralGlyph);

    for (cmd, ch) in [
        ("infty", '∞'),
        ("partial", '∂'),
        ("nabla", '∇'),
        ("forall", '∀'),
        ("exists", '∃'),
        ("neg", '¬'),
        ("emptyset", '∅'),
        ("triangle", '△'),
        ("angle", '∠'),
        ("therefore", '∴'),
        ("because", '∵'),
        ("cdot", '·'),
        ("times", '×'),
        ("div", '÷'),
        ("pm", '±'),
        ("mp", '∓'),
        ("circ", '∘'),
        ("bullet", '∙'),
        ("star", '⋆'),
        ("le", '≤'),
        ("ge", '≥'),
        ("ne", '≠'),
        ("approx", '≈'),
        ("equiv", '≡'),
        ("sim", '∼'),
        ("simeq", '≃'),
        ("cong", '≅'),
        ("propto", '∝'),
        ("perp", '⊥'),
        ("parallel", '∥'),
        ("to", '→'),
        ("rightarrow", '→'),
        ("Rightarrow", '⇒'),
        ("leftarrow", '←'),
        ("Leftarrow", '⇐'),
        ("mapsto", '↦'),
        ("implies", '⇒'),
        ("iff", '⇔'),
        ("in", '∈'),
        ("notin", '∉'),
        ("subset", '⊂'),
        ("supset", '⊃'),
        ("subseteq", '⊆'),
        ("supseteq", '⊇'),
        ("cup", '∪'),
        ("cap", '∩'),
        ("sum", '∑'),
        ("prod", '∏'),
        ("lvert", '|'),
        ("rvert", '|'),
        ("langle", '⟨'),
        ("rangle", '⟩'),
        ("lfloor", '⌊'),
        ("rfloor", '⌋'),
        ("lceil", '⌈'),
        ("rceil", '⌉'),
        ("quad", ' '),
        ("dots", '⋯'),
    ] {
        r.register(cmd, UnicodeGlyph(ch));
    }

    r.register("|", AbsGlyph);

    for (cmd, map) in [
        ("mathbf", to_bold as fn(char) -> char),
        ("mathbb", to_bb),
        ("mathrm", to_upright),
        ("mathit", to_italic),
        ("mathsf", to_sans),
    ] {
        r.register(cmd, AlphabetGlyph(map));
    }

    for (cmd, mark) in [
        ("hat", '^'),
        ("tilde", '~'),
        ("bar", '‾'),
        ("vec", '→'),
        ("dot", '˙'),
        ("ddot", '¨'),
        ("acute", '´'),
        ("grave", '`'),
        ("check", 'ˇ'),
        ("breve", '˘'),
    ] {
        r.register(
            cmd,
            AccentGlyph {
                mark,
                stretch: false,
            },
        );
    }

    for (cmd, mark) in [("overline", '─'), ("widehat", '^'), ("widetilde", '~')] {
        r.register(
            cmd,
            AccentGlyph {
                mark,
                stretch: true,
            },
        );
    }

    r
}
