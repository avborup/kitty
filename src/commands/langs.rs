use color_eyre::owo_colors::OwoColorize;

use crate::App;

pub async fn langs(app: &App) -> crate::Result<()> {
    if app.config.languages.is_empty() {
        println!("No languages found. Have you set up your kitty.yml config file?");
        return Ok(());
    }

    let mut langs = app
        .config
        .languages
        .iter()
        .map(|lang| (lang.to_string(), lang.file_ext()))
        .collect::<Vec<_>>();

    langs.sort();

    println!("{:9}  {}", "Name".bright_cyan(), "Extension".bright_cyan());
    for (name, ext) in langs {
        println!("{:9}  {}", name, ext);
    }

    Ok(())
}
