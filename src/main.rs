//! Gigacenter is a tool for managing Gigabyte laptops fan speed and battery threshold (current tested Aorus 16X only)
#![cfg_attr(
    feature = "gui",
    expect(
        forbidden_lint_groups,
        reason = "Slint generated code contains warnings, so mute it until it fixed"
    )
)]
mod bat;
mod cli;
mod common;
mod daemon;
mod ec;
mod fan_speed;
mod macros;
mod monitor;
mod registers;
mod temp;
mod traits;
#[cfg(feature = "gui")]
mod ui;

use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use bat::BatThreshold;
use cli::{DaemonCommands, cli};
use common::{EXIT_MSG, Handler};
use daemon::server::start_daemon;
use ec::*;
use env_logger::{Env, init_from_env};
use fan_speed::FanMode;
use libc::geteuid;
use log::{debug, info, warn};
use monitor::Monitor;
use traits::ECHandler;
#[cfg(feature = "gui")]
use ui::gui;

#[tokio::main]
async fn main() -> Result<()> {
    init_from_env(Env::default().filter_or("RUST_LOG", "info"));
    let cli = cli();
    let args = std::env::args().collect::<Vec<_>>();
    let euid = unsafe { geteuid() };
    let matches = cli.get_matches_from(args);
    debug!("Matches ready");

    // matches.args_present() just broken for now, so the next code is such a crap
    // https://github.com/clap-rs/clap/issues/5860
    #[cfg(feature = "gui")]
    if matches.index_of("daemon").is_none()
        && matches.index_of("bat_threshold").is_none()
        && !matches.get_flag("show")
        && matches.index_of("fan_mode").is_none()
    {
        gui().await?;
        std::process::exit(0);
    }

    if let Some(daemon_cmd) = matches.get_one::<DaemonCommands>("daemon") {
        if euid != 0 {
            rerun_as_root()
        }
        match daemon_cmd {
            DaemonCommands::Run => {
                start_daemon().await.context("Start daemon")?;
            }
            DaemonCommands::Install => {
                #[cfg(feature = "self-packed")]
                crate::daemon::server::install_daemon()?;
                #[cfg(not(feature = "self-packed"))]
                {
                    log::error!(
                        "Build gigalinux with self-packed feature to use `--daemon install` command"
                    );
                    std::process::exit(1);
                }
            }
            DaemonCommands::Remove => {
                #[cfg(feature = "self-packed")]
                crate::daemon::server::remove_daemon()?;
                #[cfg(not(feature = "self-packed"))]
                {
                    log::error!(
                        "Build gigalinux with self-packed feature to use `--daemon install` command"
                    );
                    std::process::exit(1);
                }
            }
        }
        std::process::exit(0);
    }

    let mut ec = match Handler::new().await {
        Ok(ec) => ec,
        Err(err) => {
            if euid != 0 {
                rerun_as_root()
            } else {
                bail!("Failed to run gigacenter: {err}");
            }
        }
    };

    if let Some(fan_mode) = matches.get_one::<FanMode>("fan_mode") {
        _ = ec.write_data(fan_mode).await?;
        info!("Fan mode set to {fan_mode}");
    }

    if let Some(threshold) = matches.get_one::<u8>("bat_threshold") {
        _ = ec.write_data(&BatThreshold::new(*threshold)).await?;
        info!("Battery threshold set to {}", *threshold);
    }
    if matches.get_flag("show") {
        let monitor = Monitor::try_new(&mut ec)
            .await
            .context("Creating monitor")?;
        println!("{}", monitor);
    }
    info!("Done!");
    Ok(())
}

fn rerun_as_root() -> ! {
    warn!("Command need to be run as root. Try rerun via `pkexec`");
    let args = std::env::args();
    let output = Command::new("pkexec")
        .args(args)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .spawn()
        .expect("Run pkexec")
        .wait_with_output()
        .unwrap();
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{stdout}");
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{stderr}");
        eprintln!("{EXIT_MSG}");
    }
    std::process::exit(output.status.code().unwrap());
}
