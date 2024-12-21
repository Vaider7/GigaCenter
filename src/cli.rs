use clap::{Arg, ArgAction, ArgGroup, Command, ValueEnum, value_parser};

use crate::fan_speed::FanMode;

pub fn cli() -> Command {
    let mut cli = Command::new("")
        .version("0.1.0")
        .propagate_version(true)
        .about("Manage your Gigabyte laptop fan speed and battery threshold on Linux")
        .styles(get_styles())
        .group(
            ArgGroup::new("exclusive")
                .args(["show", "fan_mode", "bat_threshold", "daemon"])
                .required(true)
                .multiple(false),
        )
        .arg_required_else_help(true)
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
                .help("Set fan speed mode")
                .value_parser(value_parser!(FanMode)),
        )
        .arg(
            Arg::new("bat_threshold")
                .short('b')
                .long("bat-threshold")
                .value_name("THRESHOLD")
                .help("Set battery threshold. Takes values from 60 to 100 (in percent)")
                .value_parser(value_parser!(u8).range(60..=100)),
        )
        .arg(
            Arg::new("daemon")
                .short('d')
                .long("daemon")
                .value_name("DAEMON_COMMAND")
                .value_parser(value_parser!(DaemonCommands)),
        );

    cli.build();
    cli
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DaemonCommands {
    /// Run daemon
    Run,
    /// Install systemd service needed to use Gigabyte Linux without root permissions
    Install,
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
