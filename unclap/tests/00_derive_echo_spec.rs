use unclap::{Argument, ArgumentExt, Flag};

#[derive(Argument)]
#[allow(dead_code)]
enum EchoMode {
    #[argument(variant(named))]
    Help,
    #[argument(variant(named))]
    Version,
    Print(EchoPrintSpec),
}

#[derive(Argument, Default)]
struct EchoPrintSpec {
    #[argument(variant(flag = "-n"))]
    no_trailing_newline: Flag,
    #[argument(variant(flag = "-e"))]
    enable_backslash_escapes: Flag,
    strings: std::vec::Vec<String>,
}

#[test]
fn test_basic_echo() {
    let spec = EchoMode::Print(EchoPrintSpec {
        no_trailing_newline: Flag::default(),
        enable_backslash_escapes: Flag::default(),
        strings: vec![String::from("hello"), String::from("world")],
    });

    let output = spec
        .to_command("echo")
        .output()
        .expect("expected echo to run successfully");
    let printed = std::str::from_utf8(&output.stdout)
        .expect("expected valid utf-8 in output")
        .trim_end(); // use a specific test for no_trailing_newline.
                     // Not sure about windows compatibility there.
    assert_eq!(printed, "hello world");

    let version_status = EchoMode::Version
        .to_command("echo")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("expected echo to run successfully");
    assert!(
        version_status.success(),
        "expected `echo --version` to exit with success"
    );

    let help_status = EchoMode::Help
        .to_command("echo")
        .stdout(std::process::Stdio::null())
        .status()
        .expect("expected echo to run successfully");
    assert!(
        help_status.success(),
        "expected `echo --help` to exit with success"
    );
}
