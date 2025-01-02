use anyhow::Result;
use slint::ComponentHandle;

use crate::{daemon::client::DaemonClient, monitor::Monitor as Monitor_};

slint::include_modules!();

pub async fn gui() -> Result<()> {
    let app = App::new()?;

    let cloned_app = app.as_weak();
    _ = tokio::spawn(async move {
        let mut daemon = DaemonClient::connect().await?;

        loop {
            let monitor = Monitor_::try_new(&mut daemon).await?;
            cloned_app
                .upgrade_in_event_loop(move |app| {
                    app.set_monitor(slint_generatedApp::Monitor {
                        bat_threshold: *monitor.bat_threshold as i32,
                        cpu_fan_speed: *monitor.cpu_fan_speed as i32,
                        cpu_temp: *monitor.cpu_temp as i32,
                        fan_mode: monitor.fan_mode as i32,
                        gpu_fan_speed: *monitor.gpu_fan_speed as i32,
                        gpu_temp: *monitor.gpu_temp as i32,
                    });
                })
                .unwrap();
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        #[allow(unreachable_code)]
        Ok::<_, anyhow::Error>(())
    });

    Ok(app.run()?)
}
