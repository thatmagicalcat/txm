use std::{
    env,
    io::{self, IsTerminal, Write},
    process, thread,
    time::Duration,
};

use unicode_width::UnicodeWidthStr;

struct Flag {
    name: &'static str,
    desc: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrideStyle {
    Lgbtq,
    Gay,
    Trans,
    Lesbian,
}

struct PrideConfig {
    style: PrideStyle,
    animate: bool,
}

struct Config {
    unboxed: bool,
    expression: String,
    pride: Option<PrideConfig>,
}

enum Cli {
    Help,
    Version,
    Run(Config),
}

#[cfg(test)]
mod tests {
    use super::{PrideStyle, pride_palette};

    #[test]
    fn pride_palette_contains_expected_colors_for_trans() {
        let colors = pride_palette(PrideStyle::Trans);
        assert_eq!(colors[0], (91, 206, 250));
    }

    #[test]
    fn pride_palette_contains_expected_colors_for_gay() {
        let colors = pride_palette(PrideStyle::Gay);
        assert_eq!(colors[0], (0, 0, 0));
    }
}

fn main() {
    let flags = [
        Flag {
            name: "--help",
            desc: "Print this help message and exit",
        },
        Flag {
            name: "--version",
            desc: "Print version information and exit",
        },
        Flag {
            name: "--unboxed",
            desc: "Render without the decorative box border",
        },
        Flag {
            name: "--pride-style <lgbtq|gay|trans|lesbian>",
            desc: "Choose the pride palette used by --pride",
        },
    ];

    let program: String = env::args().next().unwrap_or_default();

    match parse_args() {
        Ok(Cli::Help) => print!("{}", help(&program, &flags)),
        Ok(Cli::Version) => println!("txm {}", env!("CARGO_PKG_VERSION")),
        Ok(Cli::Run(config)) => match txm::render(&config.expression) {
            Ok(rendered) => {
                let output = if config.unboxed {
                    rendered
                } else {
                    boxed(&rendered)
                };

                if let Some(pride) = config.pride.as_ref() {
                    if pride.animate && io::stdout().is_terminal() {
                        animate_pride(&output, pride);
                    } else {
                        print!("{}", render_pride(&output, pride, 0));
                    }
                } else {
                    print!("{output}");
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        Err(msg) if msg == "missing expression" => {
            print!("{}", help(&program, &flags));
        }
        Err(msg) => {
            eprintln!("error: {msg}");
            eprintln!("{}", help(&program, &flags));
            process::exit(2);
        }
    }
}

fn parse_args() -> Result<Cli, String> {
    let args = env::args().skip(1);
    let mut unboxed = false;
    let mut pride_enabled = false;
    let mut pride_style = PrideStyle::Lgbtq;
    let mut expression: Option<String> = None;

    let mut iter = args.peekable();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--help" => return Ok(Cli::Help),
            "--version" => return Ok(Cli::Version),
            "--unboxed" => unboxed = true,
            "--pride-style" => {
                let value = iter.next().ok_or_else(|| "missing value for --pride-style".to_string())?;
                pride_style = match value.to_ascii_lowercase().as_str() {
                    "lgbtq" => PrideStyle::Lgbtq,
                    "gay" => PrideStyle::Gay,
                    "trans" => PrideStyle::Trans,
                    "lesbian" => PrideStyle::Lesbian,
                    other => return Err(format!("unknown pride style '{other}'")),
                };
                pride_enabled = true; 
            }
            s if s.starts_with("--") => return Err(format!("unknown flag '{s}'")),
            s => {
                if expression.replace(s.to_string()).is_some() {
                    return Err(format!("unexpected extra argument '{s}'"));
                }
            }
        }
    }

    // Construct the PrideConfig after processing all arguments
    let pride = if pride_enabled {
        Some(PrideConfig {
            style: pride_style,
            animate: true,
        })
    } else {
        None
    };

    let expression = expression.ok_or_else(|| "missing expression".to_string())?;
    Ok(Cli::Run(Config {
        unboxed,
        expression,
        pride,
    }))
}

fn help(program: &str, flags: &[Flag]) -> String {
    let max_len = flags.iter().map(|f| f.name.len()).max().unwrap_or(0);

    let opts: String = flags
        .iter()
        .map(|f| {
            let gap = " ".repeat(max_len - f.name.len() + 2);
            format!("  {}{gap}{}", f.name, f.desc)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Usage: {program} [OPTIONS] [EXPRESSION]

Terminal Math Rendering Engine - renders LaTeX math expressions in your terminal.

OPTIONS:
{opts}

EXAMPLES:
  {program} \"E = mc^2\"
  {program}  \"\\lim_{{x\\,\\to\\,\\infty}}\\,\\int_0^x{{\\frac{{\\sin\\, t^2}}{{1 + t^4}}}}\\, dt = L\"
"
    )
}

fn boxed(rendered: &str) -> String {
    let lines: Vec<&str> = rendered.lines().collect();
    let width = lines
        .iter()
        .map(|line| UnicodeWidthStr::width(*line))
        .max()
        .unwrap_or(0);
    let border = "─".repeat(width + 2);
    let mut out = format!("┌{border}┐\n│ {} │\n", " ".repeat(width));
    for line in lines {
        let padding = width - UnicodeWidthStr::width(line);
        out.push_str(&format!("│ {line}{} │\n", " ".repeat(padding)));
    }
    out.push_str(&format!("│ {} │\n└{border}┘\n", " ".repeat(width)));
    out
}

fn render_pride(output: &str, pride: &PrideConfig, frame: usize) -> String {
    let palette = pride_palette(pride.style);
    let lines: Vec<&str> = output.lines().collect();
    let mut out = String::new();

    // 🌊 Wave Animation Parameters
    let amplitude = 3.0; // How far left/right the flag waves (in character slots)
    let frequency = 0.4; // How tight the wave curls vertically per line
    let speed = 0.25; // How fast the flag ripples over time

    for (line_idx, line) in lines.iter().enumerate() {
        // Calculate the horizontal sine wave offset for this specific row
        let wave_time = frame as f32 * speed;
        let row_phase = line_idx as f32 * frequency;
        let wave_offset = ((wave_time + row_phase).sin() * amplitude).round() as isize;

        // Add a base padding so negative wave offsets don't crash or hit the screen edge
        let safety_margin = amplitude as isize;
        let total_leading_spaces = (safety_margin + wave_offset) as usize;

        // Push the leading spaces to create the physical wave displacement
        out.push_str(&" ".repeat(total_leading_spaces));

        let mut col_idx = 0usize;
        for ch in line.chars() {
            if ch == ' ' {
                out.push(' ');
            } else {
                let (r, g, b) = palette[(frame + line_idx * 2 + col_idx) % palette.len()];
                out.push_str(&format!("\x1b[38;2;{r};{g};{b}m{ch}\x1b[0m"));
            }
            col_idx += 1;
        }
        if line_idx + 1 < lines.len() {
            out.push('\n');
        }
    }

    if output.ends_with('\n') {
        out.push('\n');
    }

    out
}

fn animate_pride(output: &str, pride: &PrideConfig) {
    let mut frame = 0usize;
    loop {
        print!("\x1b[H\x1b[2J{}", render_pride(output, pride, frame));
        io::stdout().flush().ok();
        thread::sleep(Duration::from_millis(80));
        frame = (frame + 1) % 24;
    }
}

fn pride_palette(style: PrideStyle) -> Vec<(u8, u8, u8)> {
    match style {
        PrideStyle::Lgbtq => vec![
            (227, 0, 34),
            (255, 140, 0),
            (255, 237, 0),
            (0, 128, 38),
            (0, 77, 255),
            (117, 7, 135),
        ],
        PrideStyle::Gay => vec![
            (7, 141, 112),
            (38, 206, 170),
            (152, 232, 193),
            (255, 255, 255),
            (123, 193, 237),
            (75, 128, 202),
            (61, 26, 120),
        ],
        PrideStyle::Trans => vec![
            (91, 206, 250),
            (245, 169, 184),
            (255, 255, 255),
        ],
        PrideStyle::Lesbian => vec![
            (213, 45, 0),    // Dark Orange
            (239, 118, 39),   // Medium Orange
            (255, 154, 86),   // Light Orange
            (255, 255, 255),  // White
            (209, 98, 164),   // Light Pink
            (181, 19, 110),   // Medium Pink
            (163, 2, 98),     // Dark Magenta
        ],
    }
}
