use anyhow::Result;

slint::include_modules!();

pub fn gui() -> Result<()> {
    let main_window = MainWindow::new()?;

    Ok(main_window.run()?)
}
