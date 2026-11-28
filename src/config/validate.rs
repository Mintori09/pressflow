use std::{
    fmt::Display,
    fs,
    io::{self, IsTerminal},
    path::PathBuf,
    process,
};

use indexmap::IndexMap;

use crate::config::{
    key_binding::KeyBinding,
    model::{ChordNode, Config},
};

const VALID_PREFIXES: &[&str] = &[
    "app:", "cmd:", "snippet:", "key:", "wm:", "media:", "volume:", "http://", "https://",
];

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub key: String,
    pub message: String,
    pub help: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ValidateReport {
    pub warnings: Vec<Diagnostic>,
    pub errors: Vec<Diagnostic>,
}

impl ValidateReport {
    fn error(&mut self, key: impl Into<String>, message: impl Into<String>, help: Option<&str>) {
        self.errors.push(Diagnostic {
            key: key.into(),
            message: message.into(),
            help: help.map(str::to_string),
        });
    }

    fn warning(&mut self, key: impl Into<String>, message: impl Into<String>, help: Option<&str>) {
        self.warnings.push(Diagnostic {
            key: key.into(),
            message: message.into(),
            help: help.map(str::to_string),
        });
    }
}

fn check_between<T: PartialOrd + Display>(
    report: &mut ValidateReport,
    key: &str,
    val: T,
    min: T,
    max: T,
    example: T,
) {
    if val < min || val > max {
        report.error(
            key,
            format!("{key} must be between {min} and {max} (got {val})"),
            Some(&format!(
                "Set a value between {min} and {max} (e.g. {example})."
            )),
        );
    }
}

fn check_min<T: PartialOrd + Display>(
    report: &mut ValidateReport,
    key: &str,
    val: T,
    min: T,
    example: T,
) {
    if val < min {
        report.error(
            key,
            format!("{key} must be >= {min} (got {val})"),
            Some(&format!("Set a value >= {min} (e.g. {example}).")),
        );
    }
}

pub fn validate(config: &Config) -> ValidateReport {
    let mut report = ValidateReport::default();
    let app = &config.appearance;

    check_between(
        &mut report,
        "appearance.window_opacity",
        app.window_opacity,
        0.0,
        1.0,
        0.85,
    );
    check_min(&mut report, "appearance.columns", app.columns, 1, 4);
    check_min(
        &mut report,
        "appearance.corner_radius",
        app.corner_radius,
        0.0,
        12.0,
    );
    check_min(
        &mut report,
        "appearance.border_width",
        app.border_width,
        0.0,
        1.0,
    );
    check_min(
        &mut report,
        "behavior.watchdog_timeout_secs",
        config.behavior.watchdog_timeout_secs,
        1,
        10,
    );
    check_between(
        &mut report,
        "behavior.cheatsheet_delay_ms",
        config.behavior.cheatsheet_delay_ms,
        0,
        10_000,
        1000,
    );

    validate_nodes(&config.keys, "keymap", &mut report);

    report
}

fn validate_nodes(
    nodes: &IndexMap<KeyBinding, ChordNode>,
    path: &str,
    report: &mut ValidateReport,
) {
    for (key, node) in nodes {
        let node_path = format!("{path}.{key:?}");
        match node {
            ChordNode::Action(action) => {
                if action.command.trim().is_empty() {
                    report.error(
                        &node_path,
                        format!("{node_path}: action command cannot be empty"),
                        None,
                    );
                } else if !VALID_PREFIXES.iter().any(|p| action.command.starts_with(p)) {
                    report.error(
                        &node_path,
                        format!("{node_path}: unknown action prefix in '{}'", action.command),
                        Some("Use a valid prefix: app:, cmd:, snippet:, key:, wm:, media:, volume:, http://, https://"),
                    );
                }

                if action.label.trim().is_empty() {
                    report.error(
                        &node_path,
                        format!("{node_path}: action label cannot be empty"),
                        None,
                    );
                }
            }
            ChordNode::Group(group) => {
                if group.label.trim().is_empty() {
                    report.error(
                        &node_path,
                        format!("{node_path}: group label cannot be empty"),
                        None,
                    );
                }
                if group.children.is_empty() {
                    report.warning(
                        &node_path,
                        format!("{node_path}: group has no children"),
                        None,
                    );
                } else {
                    validate_nodes(&group.children, &node_path, report);
                }
            }
        }
    }
}

pub(crate) struct Colors {
    pub(crate) red: &'static str,
    yellow: &'static str,
    green: &'static str,
    cyan: &'static str,
    bold: &'static str,
    dim: &'static str,
    pub(crate) reset: &'static str,
}

const ANSI: Colors = Colors {
    red: "\x1b[1;31m",
    yellow: "\x1b[1;33m",
    green: "\x1b[1;32m",
    cyan: "\x1b[36m",
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    reset: "\x1b[0m",
};

const PLAIN: Colors = Colors {
    red: "",
    yellow: "",
    green: "",
    cyan: "",
    bold: "",
    dim: "",
    reset: "",
};

impl Colors {
    pub(crate) fn detect() -> Self {
        if io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
            ANSI
        } else {
            PLAIN
        }
    }
}

fn find_key_span<'a>(content: &'a str, key_path: &str) -> Option<(usize, usize, usize, &'a str)> {
    let (target_section, target_key) = key_path.rsplit_once('.')?;
    let mut current_section = "";

    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].trim();
            continue;
        }
        if current_section == target_section
            && let Some(eq_pos) = line.find('=')
        {
            let lhs = line[..eq_pos].trim();
            if lhs == target_key {
                let col = line.find(target_key)? + 1;
                return Some((idx + 1, col, target_key.len(), line));
            }
        }
    }
    None
}

fn print_diagnostic(is_error: bool, diag: &Diagnostic, content: &str, c: &Colors) {
    let (badge, color) = if is_error {
        ("×", c.red)
    } else {
        ("⚠", c.yellow)
    };

    let loc = find_key_span(content, &diag.key)
        .map(|(line_no, col, _, _)| {
            format!(
                "{dim}[{line_no}:{col}]{reset} ",
                dim = c.dim,
                reset = c.reset
            )
        })
        .unwrap_or_default();

    eprintln!(
        "  {color}{badge}{reset} {loc}{bold}{}{reset}",
        diag.message,
        reset = c.reset,
        bold = c.bold
    );

    if let Some(help) = &diag.help {
        eprintln!(
            "    {cyan}help:{reset} {dim}{help}{reset}",
            cyan = c.cyan,
            dim = c.dim,
            reset = c.reset
        );
    }
}

impl Config {
    pub fn validate(path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let c = Colors::detect();

        let content = fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!(
                "{}×{} Failed to read '{}': {e}",
                c.red,
                c.reset,
                path.display()
            );
            process::exit(1);
        });

        let config: Config = toml::from_str(&content).unwrap_or_else(|e| {
            eprintln!(
                "{}×{} Invalid TOML in '{}':\n{e}",
                c.red,
                c.reset,
                path.display()
            );
            process::exit(1);
        });

        let report = validate(&config);

        for warning in &report.warnings {
            print_diagnostic(false, warning, &content, &c);
        }
        for error in &report.errors {
            print_diagnostic(true, error, &content, &c);
        }

        if !report.errors.is_empty() {
            eprintln!(
                "\n{}Found {} error(s) and {} warning(s){} in {}{}{}",
                c.red,
                report.errors.len(),
                report.warnings.len(),
                c.reset,
                c.bold,
                path.display(),
                c.reset
            );
            process::exit(1);
        }

        if report.warnings.is_empty() {
            println!(
                "{}✓{} Config is valid: {}{}{}",
                c.green,
                c.reset,
                c.bold,
                path.display(),
                c.reset
            );
        } else {
            println!(
                "\n{}✓{} Config is valid ({}{} warning(s){}): {}{}{}",
                c.green,
                c.reset,
                c.yellow,
                report.warnings.len(),
                c.reset,
                c.bold,
                path.display(),
                c.reset
            );
        }

        Ok(config)
    }
}
