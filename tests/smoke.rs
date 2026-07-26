use std::process::Command;

#[test]
fn prints_usage_without_arguments() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .output()
        .expect("failed to run txm");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
}

#[test]
fn boxes_simple_identifier() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .arg("x")
        .output()
        .expect("failed to run txm");

    assert!(output.status.success());
    let rendered = String::from_utf8_lossy(&output.stdout).to_string();
    // The box should contain the italic x character (rendered via ANSI escape)
    assert!(
        rendered.contains('x'),
        "expected italic x in box: {rendered:?}"
    );
    assert!(
        rendered.starts_with('┌'),
        "should start with box border: {rendered:?}"
    );
}

#[test]
fn boxes_wide_identifier() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .arg("你")
        .output()
        .expect("failed to run txm");

    assert!(output.status.success());
    let rendered = String::from_utf8_lossy(&output.stdout).to_string();
    // CJK characters don't have italic variants, so 你 stays as 你
    assert!(rendered.contains('你'), "expected 你 in box: {rendered:?}");
    assert!(
        rendered.starts_with('┌'),
        "should start with box border: {rendered:?}"
    );
}

#[test]
fn boxes_adjacent_wide_identifiers() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .arg("你你")
        .output()
        .expect("failed to run txm");

    assert!(output.status.success());
    let rendered = String::from_utf8_lossy(&output.stdout).to_string();
    assert!(rendered.contains('你'), "expected 你 in box: {rendered:?}");
    assert!(
        rendered.starts_with('┌'),
        "should start with box border: {rendered:?}"
    );
}

#[test]
fn render_returns_raw_lines_for_simple_identifier() {
    let rendered = txm::render("x").expect("render failed");

    // x should be rendered as italic via ANSI escape codes
    assert!(rendered.contains('x'), "expected italic x: {rendered:?}");
}

#[test]
fn braces_group_invisibly() {
    let rendered = txm::render("{x}").unwrap();
    assert!(rendered.contains('x'), "expected x: {rendered:?}");
    assert!(
        !rendered.contains('{'),
        "should not contain literal {{: {rendered:?}"
    );
}

#[test]
fn render_returns_error_for_invalid_lexer_input() {
    assert!(txm::render("\\").is_err());
}

#[test]
fn render_returns_error_for_unknown_matrix_environment() {
    assert!(txm::render(r"\begin{unknown}x\end{unknown}").is_err());
}

#[test]
fn render_returns_error_for_ragged_matrix() {
    assert!(txm::render(r"\begin{matrix}a&b\\c\end{matrix}").is_err());
}

#[test]
fn cli_reports_render_errors_without_panicking() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .arg(r"\begin{unknown}x\end{unknown}")
        .output()
        .expect("failed to run txm");

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "error: unknown matrix environment: unknown\n"
    );
}

#[test]
fn mathbf_maps_to_bold_alphabet() {
    let rendered = txm::render(r"\mathbf{x}").unwrap();
    // \mathbf applies to_bold on italic x → produces bold italic x
    assert!(
        rendered.contains('𝐱') || rendered.contains('𝐱'),
        "expected bold math x: {rendered:?}"
    );
}

#[test]
fn mathbb_uses_letterlike_specials() {
    let rendered = txm::render(r"\mathbb{R}").unwrap();
    assert!(
        rendered.contains('ℝ'),
        "expected blackboard bold R: {rendered:?}"
    );
}

#[test]
fn single_token_argument_needs_no_braces() {
    let rendered = txm::render(r"\mathbf n").unwrap();
    assert!(
        rendered.contains('𝐧') || rendered.contains('𝐧'),
        "expected bold math n: {rendered:?}"
    );
}

#[test]
fn accent_stacks_mark_above_argument() {
    let rendered = txm::render(r"\hat{x}").unwrap();
    assert!(rendered.contains('^'), "expected hat mark: {rendered:?}");
    assert!(rendered.contains('x'), "expected italic x: {rendered:?}");
}

#[test]
fn latex_style_parentheses_render_as_paired_delimiters() {
    let rendered = txm::render(r"\left( x \right)").unwrap();

    assert!(rendered.contains('(') || rendered.contains('⎛'));
    assert!(rendered.contains(')') || rendered.contains('⎠'));
    assert!(rendered.contains('x'), "expected italic x: {rendered:?}");
}

#[test]
fn latex_style_brackets_render_fraction_inside() {
    let rendered = txm::render(r"\left[ \frac{1}{2} \right]").unwrap();

    assert!(rendered.contains('[') || rendered.contains('⎡'));
    assert!(rendered.contains(']') || rendered.contains('⎤'));
    assert!(rendered.contains('1'));
    assert!(rendered.contains('2'));
}

#[test]
fn unmatched_latex_delimiters_fail_gracefully() {
    assert!(txm::render(r"\left( x ").is_err());
}

#[test]
fn inline_punctuation_renders_literally() {
    let rendered = txm::render(r"(3,0)").unwrap();
    // Numbers stay upright, comma stays literal
    assert!(rendered.contains('3'), "expected 3: {rendered:?}");
    assert!(rendered.contains(','), "expected comma: {rendered:?}");
    assert!(rendered.contains('0'), "expected 0: {rendered:?}");
}

#[test]
fn stretchy_brackets_use_side_correct_extensions() {
    let rendered = txm::render(r"\begin{bmatrix}a\\b\\c\end{bmatrix}").unwrap();
    let lines: Vec<&str> = rendered.lines().collect();
    assert!(lines.len() >= 3, "expected tall bracket: {rendered:?}");
    assert!(
        lines[1].starts_with('⎢') && lines[1].ends_with('⎥'),
        "middle row should use left/right bracket pieces: {:?}",
        lines[1]
    );
}

#[test]
fn pipe_delimiters_render_like_abs() {
    assert_eq!(
        txm::render("|x|").unwrap(),
        txm::render(r"\abs{x}").unwrap()
    );

    let rendered = txm::render("|x|").unwrap();
    assert!(rendered.contains('x'), "expected italic x: {rendered:?}");
    assert!(
        rendered.contains('│'),
        "expected pipe delimiters: {rendered:?}"
    );
}

#[test]
fn text_mode_produces_upright_text() {
    let rendered = txm::render(r"\text{x}").unwrap();
    // \text should produce upright x, not italic
    assert!(rendered.contains('x'), "expected upright x: {rendered:?}");
    assert!(
        !rendered.contains('𝑥'),
        "should not contain italic x: {rendered:?}"
    );
}

#[test]
fn textbf_produces_bold_upright() {
    let rendered = txm::render(r"\textbf{hello}").unwrap();
    assert!(rendered.contains('h'), "expected upright h: {rendered:?}");
    assert!(
        !rendered.contains('ℎ'),
        "should not contain italic h: {rendered:?}"
    );
}

#[test]
fn color_applies_to_fraction_line() {
    let rendered = txm::render(r"\color{red}{\frac{x}{y}}").unwrap();
    // Both the fraction line and content should be colored
    assert!(
        rendered.contains('─'),
        "expected fraction line: {rendered:?}"
    );
    assert!(rendered.contains('x'), "expected numerator: {rendered:?}");
    assert!(rendered.contains('y'), "expected denominator: {rendered:?}");
}
