use pressflow::config::{model::Config, validate::validate};

#[test]
fn example_config_is_valid() {
    let content = include_str!("../examples/config.example.toml");
    let config: Config = toml::from_str(content).expect("valid TOML");
    let report = validate(&config);
    assert!(
        report.errors.is_empty(),
        "unexpected errors: {:?}",
        report.errors
    );
    assert!(
        report.warnings.is_empty(),
        "unexpected warnings: {:?}",
        report.warnings
    );
}

#[test]
fn fail_config_reports_expected_errors() {
    let content = r#"
[appearance]
window_opacity = 1.5
columns = 0
corner_radius = -4.0
border_width = -1.0

[behavior]
watchdog_timeout_secs = 0
cheatsheet_delay_ms = 25000
"#;
    let config: Config = toml::from_str(content).expect("valid TOML");
    let report = validate(&config);

    let keys: Vec<&str> = report.errors.iter().map(|e| e.key.as_str()).collect();
    assert_eq!(
        keys,
        vec![
            "appearance.window_opacity",
            "appearance.columns",
            "appearance.corner_radius",
            "appearance.border_width",
            "behavior.watchdog_timeout_secs",
            "behavior.cheatsheet_delay_ms",
        ]
    );
}
