//! Linux gigabyte is a tool for managing fan speed and battery threshold (current support only Aorus 16X)
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_from_env(Env::default().filter_or("RUST_LOG", "info"));
    let cli = cli();
    let this_exe = std::env::current_exe()?;
    let mut args = std::env::args().peekable();
    let euid = unsafe { geteuid() };
    _ = args.next_if(|first| *first == this_exe.to_string_lossy());

    let matches = cli.get_matches_from(args);
    debug!("Matches ready");
    if let Some(daemon_cmd) = matches.get_one::<DaemonCommands>("daemon") {
        if euid != 0 {
            rerun_as_root()
        }
        match daemon_cmd {
            DaemonCommands::Run => {
                start_daemon().await.context("Start daemon")?;
            }
            DaemonCommands::Install => todo!(),
        }
        std::process::exit(0);
    }

    let mut ec = match Handler::new().await {
        Ok(ec) => ec,
        Err(err) => {
            if euid != 0 {
                rerun_as_root()
            } else {
                bail!("Failed to run Gigabyte Linux: {err}");
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
    if matches.get_one::<bool>("show").is_some_and(|arg| *arg) {
        let monitor = Monitor::try_new(&mut ec)
            .await
            .context("Creating monitor")?;
        println!("{}", monitor);
    }
    Ok(())
}

fn rerun_as_root() -> ! {
    warn!("Command need to be run as root. Try rerun via `pkexec`");
    let this_exe = std::env::current_exe().unwrap();
    let args = std::env::args();
    let output = Command::new("pkexec")
        .arg(this_exe)
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
