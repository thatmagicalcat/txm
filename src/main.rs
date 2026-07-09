use std::env;

use glyph::*;
use parser::Parser;
use render::render;
use token::tokenize;

mod ast;
mod glyph;
mod layout;
mod parser;
mod render;
mod token;

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
        "sup", "inf", "max", "min", "arg", "Pr", "mod",
    ] {
        r.register(*name, TextGlyph(name));
    }

    r.register("binom", BinomGlyph);
    r.register("frac", FracGlyph);
    r.register("sqrt", SqrtGlyph);

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
        ("int", '∫'),
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

    r
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: txm [LaTeX input]");
        return;
    }

    let reg = build_registry();
    render_boxed(&args[1], &reg);
}

fn render_boxed(input: &str, reg: &SymbolRegistry) {
    let tokens = tokenize(input);
    let mut parser = Parser::new(&tokens, reg);
    let expr = parser.parse_expr().unwrap();
    // dbg!(&expr);
    let mut ctx = RenderCtx::default();
    let layout = render(&expr, reg, &mut ctx);

    let w = layout.width + 4;
    let h = layout.height + 4;
    let mut box_data = vec![' '; w * h];

    box_data[0] = '┌';
    box_data[w - 1] = '┐';
    box_data[(h - 1) * w] = '└';
    box_data[(h - 1) * w + w - 1] = '┘';

    for x in 1..w - 1 {
        box_data[x] = '─';
        box_data[(h - 1) * w + x] = '─';
    }

    for y in 1..h - 1 {
        box_data[y * w] = '│';
        box_data[y * w + w - 1] = '│';
    }

    layout.blit_into(&mut box_data, w, 2, 2);

    for y in 0..h {
        for x in 0..w {
            print!("{}", box_data[y * w + x]);
        }
        println!();
    }
}
