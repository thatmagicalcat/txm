use std::{env, fs, process};
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
        Flag {
            name: "--file <FILE>",
            desc: "Render specified file (- for stdin)",
        },
    ];

    let program: String = env::args().next().unwrap_or_default();

    match parse_args() {
        Ok(Cli::Help) => print!("{}", help(&program, &flags)),
        Ok(Cli::Version) => println!("txm {}", env!("CARGO_PKG_VERSION")),
        Ok(Cli::Run(config)) => match txm::render(&config.expression) {
            Ok(rendered) => {
                if config.unboxed {
                    print!("{rendered}");
                } else {
                    cfg_select! {
                        feature = "fancy" => println!("{rendered}"),
                        _  => boxed(&rendered, &mut std::io::stdout()),
                    }
                }
            }
            Err(e) => {
                eprintln!("error: {e}");
                std::process::exit(1);
            }
        },
        Err(msg) if msg == "missing expression" => print!("{}", help(&program, &flags)),
        Err(msg) => {
            eprintln!("error: {msg}");
            eprintln!("\nFor more information, try '--help'.");
            process::exit(2);
        }
    }
}

fn parse_args() -> Result<Cli, String> {
    let mut args = env::args().skip(1);
    let mut unboxed = false;
    let mut expression: Option<String> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => return Ok(Cli::Help),
            "--version" => return Ok(Cli::Version),
            "--unboxed" => unboxed = true,
            "--file" => {
                // grab the next token for the file path
                let path_arg = args
                    .next()
                    .ok_or_else(|| "missing file path after '--file'".to_string())?;

                let path = if path_arg == "-" {
                    "/dev/stdin"
                } else {
                    &path_arg
                };

                let contents = fs::read_to_string(path)
                    .map_err(|err| format!("unable to open file '{path_arg}': {err}"))?;

                if expression.replace(contents).is_some() {
                    return Err(format!("unexpected extra argument '{path_arg}'"));
                }
            }
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

Terminal Math Rendering Engine - renders LaTeX math expressions in your terminal.

OPTIONS:
{opts}

EXAMPLES:
  {program} \"E = mc^2\"
  {program}  \"\\lim_{{x\\,\\to\\,\\infty}}\\,\\int_0^x{{\\frac{{\\sin\\, t^2}}{{1 + t^4}}}}\\, dt = L\"
"
    )
}

#[allow(unused)]
fn boxed(rendered: &str, f: &mut impl std::io::Write) {
    let lines: Vec<&str> = rendered.lines().collect();
    let width = lines
        .iter()
        .map(|line| line.width()) // Use the extension trait method directly
        .max()
        .unwrap_or(0);

    let border = "─".repeat(width + 2);

    let _ = writeln!(f, "┌{border}┐");
    let _ = writeln!(f, "│ {:width$} │", "", width = width);

    for line in lines {
        let padding = width - line.width();
        let _ = writeln!(f, "│ {line}{:padding$} │", "", padding = padding);
    }

    let _ = writeln!(f, "│ {:width$} │", "", width = width);
    let _ = writeln!(f, "└{border}┘");
}
