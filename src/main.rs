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

use std::{
    process::{Command, Stdio},
    str::FromStr,
};

use anyhow::{bail, Context, Result};
use bat::BatThreshold;
use cli::{cli, DaemonCommands};
use common::{Handler, EXIT_MSG};
use daemon::server::start_daemon;
use ec::*;
use env_logger::{init_from_env, Env};
use fan_speed::FanMode;
use libc::geteuid;
use log::{debug, info, warn};
use monitor::Monitor;
use traits::ECHandler;

fn main() -> Result<()> {
    let cli = cli();
    let args = std::env::args().collect::<Vec<_>>();
    let euid = unsafe { geteuid() };
    let matches = cli.get_matches_from(args);
    if matches.get_flag("logs") {
        init_from_env(Env::default().filter_or("RUST_LOG", "info"));
    }
    debug!("Matches ready");

    #[cfg(feature = "gui")]
    if std::env::args().len() == 1 {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()?;
        runtime.block_on(async { crate::ui::gui().await })?;
        std::process::exit(0);
    }

    if let Some(daemon_cmd) = matches.get_one::<DaemonCommands>("daemon") {
        if euid != 0 {
            rerun_as_root()
        }
        match daemon_cmd {
            DaemonCommands::Run => {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                runtime.block_on(async { start_daemon().await.context("Start daemon") })?;
            }
            DaemonCommands::Install => {
                #[cfg(feature = "self-packed")]
                crate::daemon::server::install_daemon()?;
                #[cfg(not(feature = "self-packed"))]
                {
                    log::error!(
                        "Build gigacenter with self-packed feature to use `--daemon install` command"
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
                        "Build gigacenter with self-packed feature to use `--daemon install` command"
                    );
                    std::process::exit(1);
                }
            }
        }
        std::process::exit(0);
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
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

        if let Some(fan_mode) = matches.get_one::<String>("fan_mode") {
            // SAFETY: fan_mode from cli guaranteed to be parsed be FanMode struct
            let fm = FanMode::from_str(fan_mode).unwrap();
            _ = ec.write_data(&fm).await?;
            info!("Fan mode set to {fan_mode}");
        } else if matches.contains_id("fan_mode") {
            let fan_mode = FanMode::current_mode(&mut ec).await?;
            println!("{fan_mode}");
        }

        if let Some(threshold) = matches.get_one::<u8>("bat_threshold") {
            _ = ec.write_data(&BatThreshold::new(*threshold)).await?;
            info!("Battery threshold set to {}", *threshold);
        } else if matches.contains_id("bat_threshold") {
            let fan_mode = BatThreshold::current_state(&mut ec).await?;
            println!("{}", *fan_mode);
        }
        if matches.get_flag("show") {
            let monitor = Monitor::try_new(&mut ec)
                .await
                .context("Creating monitor")?;
            println!("{}", monitor);
        }
        Ok::<_, anyhow::Error>(())
    })?;
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
