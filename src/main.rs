use std::{env, process};

use unicode_width::UnicodeWidthStr;

struct Flag {
    name: &'static str,
    desc: &'static str,
}

struct Config {
    unboxed: bool,
    expression: String,
}

enum Cli {
    Help,
    Version,
    Run(Config),
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
    ];

    let program: String = env::args().next().unwrap_or_default();

    match parse_args() {
        Ok(Cli::Help) => print!("{}", help(&program, &flags)),
        Ok(Cli::Version) => println!("txm {}", env!("CARGO_PKG_VERSION")),
        Ok(Cli::Run(config)) => {
            let rendered = txm::render(&config.expression);
            if config.unboxed {
                print!("{rendered}");
            } else {
                print!("{}", boxed(&rendered));
            }
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
    let mut expression: Option<String> = None;

    for arg in args {
        match arg.as_str() {
            "--help" => return Ok(Cli::Help),
            "--version" => return Ok(Cli::Version),
            "--unboxed" => unboxed = true,
            s if s.starts_with("--") => return Err(format!("unknown flag '{s}'")),
            s => {
                if expression.replace(s.to_string()).is_some() {
                    return Err(format!("unexpected extra argument '{s}'"));
                }
            }
        }
    }

    let expression = expression.ok_or_else(|| "missing expression".to_string())?;
    Ok(Cli::Run(Config {
        unboxed,
        expression,
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

Terminal math renderer - renders LaTeX math expressions in your terminal.

OPTIONS:
{opts}

EXAMPLES:
  {program} \"E = mc^2\"
  {program} --unboxed \"\\sum_{{n=1}}^{{\\infty}} \\frac{{1}}{{n^2}}\"
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
