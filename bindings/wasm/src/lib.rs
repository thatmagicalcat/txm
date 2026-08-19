use libtxm;
use wasm_bindgen::prelude::*;

/// Render a LaTeX math expression to Unicode art (one newline-separated row per line).
#[wasm_bindgen]
pub fn render(input: &str) -> Result<String, JsError> {
    libtxm::render(input).map_err(|e| JsError::new(&e.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_basic() {
        let out = libtxm::render("E = mc^2").unwrap();
        assert!(out.contains('m'));
        assert!(out.contains('c'));
    }

    #[test]
    fn render_integral() {
        let out = libtxm::render(r"\int_0^\infty e^{-x^2}\,dx").unwrap();
        assert!(out.contains('∞'));
        assert!(out.contains('x'));
    }

    #[test]
    fn render_fraction() {
        let out = libtxm::render(r"\frac{1}{2}").unwrap();
        assert!(out.contains('1'));
        assert!(out.contains('2'));
    }

    #[test]
    fn render_error_on_unclosed_brace() {
        assert!(libtxm::render(r"\frac{1").is_err());
    }
}
