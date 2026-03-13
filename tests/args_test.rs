use prism::args::*;

fn make_config(argv: Vec<&str>) -> ArgsConfig {
    ArgsConfig {
        name: "test".into(),
        version: Some("1.0.0".into()),
        description: Some("Test CLI".into()),
        flags: vec![
            (
                "verbose".into(),
                FlagDef {
                    flag_type: FlagType::Boolean,
                    short: Some('v'),
                    description: Some("Verbose output".into()),
                    default: None,
                    required: false,
                    placeholder: None,
                },
            ),
            (
                "output".into(),
                FlagDef {
                    flag_type: FlagType::String,
                    short: Some('o'),
                    description: Some("Output file".into()),
                    default: None,
                    required: false,
                    placeholder: Some("file".into()),
                },
            ),
        ],
        commands: vec![
            (
                "scan".into(),
                CommandDef {
                    description: Some("Scan networks".into()),
                    flags: vec![(
                        "target".into(),
                        FlagDef {
                            flag_type: FlagType::String,
                            short: Some('t'),
                            description: Some("Target BSSID".into()),
                            default: None,
                            required: false,
                            placeholder: None,
                        },
                    )],
                    usage: Some("[options]".into()),
                    hidden: false,
                },
            ),
            (
                "attack".into(),
                CommandDef {
                    description: Some("Run attacks".into()),
                    flags: vec![],
                    usage: Some("<type> [target]".into()),
                    hidden: false,
                },
            ),
        ],
        argv: Some(argv.into_iter().map(String::from).collect()),
        no_exit: true,
        allow_no_command: false,
        ..Default::default()
    }
}

#[test]
fn parse_command() {
    let result = args(make_config(vec!["scan"]));
    assert_eq!(result.command.as_deref(), Some("scan"));
}

#[test]
fn parse_command_with_positionals() {
    let result = args(make_config(vec!["attack", "pmkid", "target"]));
    assert_eq!(result.command.as_deref(), Some("attack"));
    assert_eq!(result.args, vec!["pmkid", "target"]);
}

#[test]
fn parse_long_flag_boolean() {
    let result = args(make_config(vec!["scan", "--verbose"]));
    assert!(result.get_bool("verbose"));
}

#[test]
fn parse_short_flag_boolean() {
    let result = args(make_config(vec!["scan", "-v"]));
    assert!(result.get_bool("verbose"));
}

#[test]
fn parse_long_flag_string() {
    let result = args(make_config(vec!["scan", "--output", "result.txt"]));
    assert_eq!(result.get_string("output"), Some("result.txt"));
}

#[test]
fn parse_short_flag_string() {
    let result = args(make_config(vec!["scan", "-o", "result.txt"]));
    assert_eq!(result.get_string("output"), Some("result.txt"));
}

#[test]
fn parse_long_flag_equals() {
    let result = args(make_config(vec!["scan", "--output=result.txt"]));
    assert_eq!(result.get_string("output"), Some("result.txt"));
}

#[test]
fn parse_command_flag() {
    let result = args(make_config(vec!["scan", "--target", "AA:BB:CC:DD:EE:FF"]));
    assert_eq!(result.get_string("target"), Some("AA:BB:CC:DD:EE:FF"));
}

#[test]
fn flag_defaults() {
    let config = ArgsConfig {
        name: "test".into(),
        flags: vec![(
            "count".into(),
            FlagDef {
                flag_type: FlagType::String,
                short: None,
                description: None,
                default: Some(FlagValue::String("10".into())),
                required: false,
                placeholder: None,
            },
        )],
        argv: Some(vec![]),
        no_exit: true,
        allow_no_command: true,
        ..Default::default()
    };
    let result = args(config);
    assert_eq!(result.get_string("count"), Some("10"));
}

#[test]
fn no_command_mode() {
    let config = ArgsConfig {
        name: "test".into(),
        flags: vec![(
            "verbose".into(),
            FlagDef {
                flag_type: FlagType::Boolean,
                short: Some('v'),
                description: None,
                default: None,
                required: false,
                placeholder: None,
            },
        )],
        argv: Some(
            vec!["-v", "file.txt"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        no_exit: true,
        allow_no_command: true,
        ..Default::default()
    };
    let result = args(config);
    assert!(result.command.is_none());
    assert!(result.get_bool("verbose"));
    assert_eq!(result.args, vec!["file.txt"]);
}

#[test]
fn double_dash_stops_parsing() {
    let config = ArgsConfig {
        name: "test".into(),
        flags: vec![(
            "verbose".into(),
            FlagDef {
                flag_type: FlagType::Boolean,
                short: Some('v'),
                description: None,
                default: None,
                required: false,
                placeholder: None,
            },
        )],
        argv: Some(
            vec!["-v", "--", "--not-a-flag"]
                .into_iter()
                .map(String::from)
                .collect(),
        ),
        no_exit: true,
        allow_no_command: true,
        ..Default::default()
    };
    let result = args(config);
    assert!(result.get_bool("verbose"));
    assert_eq!(result.args, vec!["--not-a-flag"]);
}
