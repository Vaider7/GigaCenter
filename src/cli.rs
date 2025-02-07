#[cfg(feature = "gui")]
use anyhow::Result;
#[cfg(feature = "gui")]
use clap::ArgMatches;

use clap::{value_parser, Arg, ArgAction, ArgGroup, Command, ValueEnum};

pub fn cli() -> Command {
    #[allow(unused_mut, reason = "Mutable access with `gui` feature")]
    let mut group = ArgGroup::new("exclusive")
        .args(["show", "fan_mode", "bat_threshold", "daemon"])
        .multiple(false);

    #[cfg(not(feature = "gui"))]
    {
        group = group.required(true);
    }

    let mut cli = Command::new("gigacenter")
        .version("0.1.0")
        .propagate_version(true)
        .about("Manage your Gigabyte laptop fan speed and battery threshold on Linux")
        .styles(get_styles())
        .group(group)
        .arg(
            Arg::new("show")
                .short('s')
                .long("show")
                .help("Show current machine state (fan speed, temperature, etc.)")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("fan_mode")
                .short('f')
                .long("fan-mode")
                .value_name("FAN_MODE")
                .action(ArgAction::Set)
                .num_args(0..=1)
                .help("Get/Set fan speed mode")
                .value_parser(["normal", "eco", "power", "turbo"]),
        )
        .arg(
            Arg::new("bat_threshold")
                .short('b')
                .long("bat-threshold")
                .value_name("THRESHOLD")
                .action(ArgAction::Set)
                .num_args(0..=1)
                .help("Get/Set battery threshold. Takes values from 60 to 100 (in percent)")
                .value_parser(value_parser!(u8).range(60..=100)),
        )
        .arg(
            Arg::new("daemon")
                .short('d')
                .long("daemon")
                .value_name("DAEMON_COMMAND")
                .value_parser(value_parser!(DaemonCommands)),
        )
        .arg(
            Arg::new("logs")
                .short('l')
                .long("enable-logs")
                .help("Enable logs")
                .action(ArgAction::SetTrue),
        )
        .after_help(
            "NOTE: Currently it's tested for Aorus 16X. For other models, use it at your own risk!",
        );

    #[allow(unused_mut, reason = "Mutable access with `gui` feature")]
    let mut base_after_help =
        "NOTE: Currently it's tested for Aorus 16X. For other models, use it at your own risk!"
            .to_owned();

    #[cfg(not(feature = "gui"))]
    {
        cli = cli.arg_required_else_help(true);
        base_after_help
            .push_str("\nThis is CLI only version. To have GUI, build the app with `gui` feature or download appropriate package on Github page");
    }
    cli = cli.after_help(base_after_help);
    cli.build();
    cli
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DaemonCommands {
    /// Run daemon
    Run,
    /// Install systemd service needed to use gigacenter without root permissions
    Install,
    /// Remove binary and systemd service
    Remove,
}

/// Thanks to https://stackoverflow.com/a/76916424
pub fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::styled()
        .usage(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .header(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Yellow))),
        )
        .literal(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .invalid(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .error(
            anstyle::Style::new()
                .bold()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Red))),
        )
        .valid(
            anstyle::Style::new()
                .bold()
                .underline()
                .fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Green))),
        )
        .placeholder(
            anstyle::Style::new().fg_color(Some(anstyle::Color::Ansi(anstyle::AnsiColor::Cyan))),
        )
}
