use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    time::Duration,
};

use anyhow::Result;
use rkyv::{rancor::Error as RkyvError, Archive, Deserialize, Serialize};
use slint::{Brush, Color, ComponentHandle, ToSharedString, Weak};
use tokio::sync::Mutex;

use crate::{
    bat::BatThreshold, daemon::client::DaemonClient, fan_speed, monitor::Monitor as Monitor_,
    traits::ECHandler, WRITE_TIMEOUT_MS,
};

slint::include_modules!();

impl From<fan_speed::FanMode> for FanMode {
    fn from(value: fan_speed::FanMode) -> Self {
        match value {
            fan_speed::FanMode::Normal => FanMode::Normal,
            fan_speed::FanMode::Eco => FanMode::Eco,
            fan_speed::FanMode::Power => FanMode::Power,
            fan_speed::FanMode::Turbo => FanMode::Turbo,
            fan_speed::FanMode::Unsupported => FanMode::Unsupported,
        }
    }
}

impl From<FanMode> for fan_speed::FanMode {
    fn from(value: FanMode) -> Self {
        match value {
            FanMode::Normal => fan_speed::FanMode::Normal,
            FanMode::Eco => fan_speed::FanMode::Eco,
            FanMode::Power => fan_speed::FanMode::Power,
            FanMode::Turbo => fan_speed::FanMode::Turbo,
            FanMode::Unsupported => fan_speed::FanMode::Unsupported,
        }
    }
}

#[derive(Debug, Clone, Copy, Archive, Serialize, Deserialize)]
struct Config {
    color: u32,
}

pub async fn gui() -> Result<()> {
    let app = App::new()?;
    let daemon: &'static Mutex<DaemonClient> =
        Box::leak(Box::new(Mutex::new(DaemonClient::connect().await?)));
    let saved_config = get_config();
    if let Ok(config) = saved_config {
        let color = Color::from_argb_encoded(config.color);
        let brush = Brush::from(color);
        let app_config = AppConfig { color: brush };
        app.global::<State<'_>>().set_config(app_config);
    }

    let cloned_app = app.as_weak();
    _ = tokio::spawn(async move {
        loop {
            let mut daemon = daemon.lock().await;
            let monitor = Monitor_::try_new(&mut *daemon).await?;
            drop(daemon);
            cloned_app.upgrade_in_event_loop(move |app| {
                let prev_threshold = app.get_monitor().bat_threshold;
                if prev_threshold != *monitor.bat_threshold as i32 {
                    app.global::<State<'_>>().set_threshold_value(
                        (*monitor.bat_threshold as i32)
                            .to_string()
                            .to_shared_string(),
                    );
                }
                app.set_monitor(slint_generatedApp::Monitor {
                    bat_threshold: *monitor.bat_threshold as i32,
                    cpu_fan_speed: *monitor.cpu_fan_speed as i32,
                    cpu_temp: *monitor.cpu_temp as i32,
                    fan_mode: monitor.fan_mode.into(),
                    gpu_fan_speed: *monitor.gpu_fan_speed as i32,
                    gpu_temp: *monitor.gpu_temp as i32,
                });
            })?;
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    app.global::<State<'_>>().on_threshold_change({
        let app = app.clone_strong();
        move |mut prev, key| {
            prev.push_str(&key.text);
            let Ok(new_val) = prev.parse::<i32>() else {
                return;
            };
            if new_val > 100 {
                return;
            }
            app.global::<State<'_>>()
                .set_threshold_value(new_val.to_string().into());
        }
    });

    app.global::<State<'_>>().on_fan_mode_change({
        let weak = app.as_weak();
        move |fan_mode| {
            block_user_input(weak.clone());
            let fm = fan_speed::FanMode::from(fan_mode);
            _ = tokio::spawn({
                async move {
                    let mut daemon = daemon.lock().await;
                    _ = daemon.write_data(&fm).await;
                }
            });
        }
    });

    app.global::<State<'_>>().on_do_backspace(|string| {
        if !string.is_empty() {
            string[0..string.len() - 1].into()
        } else {
            string
        }
    });

    app.global::<State<'_>>().on_set_threshold({
        let app = app.clone_strong();
        move |value| {
            // SAFETY:
            // no panic because the value is provided from input field, where only values between 0 and 100 are allowed
            let mut threshold = value.parse::<u8>().unwrap();
            if threshold < 60 {
                threshold = 60;
            }
            block_user_input(app.as_weak());
            app.global::<State<'_>>()
                .set_threshold_value(threshold.to_string().to_shared_string());

            _ = tokio::spawn({
                async move {
                    let mut daemon = daemon.lock().await;
                    _ = daemon.write_data(&BatThreshold::new(threshold)).await;
                }
            });
        }
    });
    app.global::<State<'_>>().on_save_config(|app_config| {
        let color = app_config.color.color().as_argb_encoded();
        let to_save = Config { color };
        _ = save_config(&to_save);
    });
    Ok(app.run()?)
}

fn block_user_input(weak: Weak<App>) {
    weak.upgrade_in_event_loop(|app| {
        app.global::<State<'_>>().set_blocked(true);
    })
    .unwrap();
    _ = tokio::spawn(async move {
        let duration = Duration::from_millis(WRITE_TIMEOUT_MS as u64);
        tokio::time::sleep(duration).await;
        weak.upgrade_in_event_loop(|app| {
            app.global::<State<'_>>().set_blocked(false);
        })
    });
}

fn get_config() -> Result<Config> {
    let mut file = get_config_file()?;
    let mut content = vec![];
    _ = file.read_to_end(&mut content)?;

    let response = rkyv::access::<ArchivedConfig, RkyvError>(&content)?;
    let response = rkyv::deserialize::<_, RkyvError>(response)?;
    Ok(response)
}

#[expect(clippy::trivially_copy_pass_by_ref)]
fn save_config(config: &Config) -> Result<()> {
    let bytes = rkyv::to_bytes::<RkyvError>(config).unwrap();
    let mut file = get_config_file()?;
    file.write_all(&bytes)?;
    Ok(())
}

fn get_config_file() -> Result<File> {
    let home_dir = std::env::var("HOME")?;
    let path = home_dir + "/.config/.gigalinux";

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    Ok(file)
}
