#![cfg(feature = "ratatui")]

use ratatui_core::{
    buffer::Buffer,
    layout::{HorizontalAlignment, Rect, VerticalAlignment},
    style::{Color, Style},
    widgets::Widget,
};

#[test]
fn math_renders_centered_text_and_reports_size() {
    let math = txm::ratatui::Math::new("abcd").expect("math creation failed");
    let math = math
        .style(Style::default().fg(Color::Red))
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center);
    let area = Rect::new(2, 1, 2, 1);
    let mut buffer = Buffer::empty(area);

    assert_eq!(math.size().width, 4);
    assert_eq!(math.size().height, 1);

    (&math).render(area, &mut buffer);

    assert_eq!(buffer[(2, 1)].symbol(), "b");
    assert_eq!(buffer[(3, 1)].symbol(), "c");
    assert_eq!(buffer[(2, 1)].fg, Color::Red);
}

#[test]
fn math_reports_display_width_for_wide_characters() {
    let math = txm::ratatui::Math::new("你你").expect("math creation failed");

    assert_eq!(math.size().width, 4);
    assert_eq!(math.size().height, 1);
}

#[test]
fn math_renders_default_left_alignment_when_content_fits() {
    let math = txm::ratatui::Math::new("ab").expect("math creation failed");
    let area = Rect::new(0, 0, 4, 1);
    let mut buffer = Buffer::empty(area);

    (&math).render(area, &mut buffer);

    assert_eq!(buffer[(0, 0)].symbol(), "a");
    assert_eq!(buffer[(1, 0)].symbol(), "b");
}

#[test]
fn math_returns_error_for_invalid_input() {
    assert!(txm::ratatui::Math::new("{x").is_err());
}
