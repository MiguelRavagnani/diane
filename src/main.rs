use diane::app::App;
use diane::cli::Cli;
use diane::config::Config;
use diane::ui::run;

use clap::Parser;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = match envy::prefixed("DIANE_").from_env::<Config>() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("diane: bad configuration: {e}");
            std::process::exit(1);
        }
    };

    let mut app = App::new(&cfg);

    let input = Cli::parse();

    match input.input_text {
        Some(text) => {
            app.push_record(text);
            app.flush_journal()?;
        }
        None => {
            app.pane = Some(input.initial_pane());
            app.retrieve_journal()?;
            app.retrieve_archive()?;
            let mut terminal = ratatui::init();
            let result = run(&mut terminal, &mut app);
            ratatui::restore();
            result?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;
    use tempfile::TempDir;

    #[test]
    fn records_from_args() {
        let root = TempDir::new().unwrap();

        Command::cargo_bin("diane")
            .unwrap()
            .env("DIANE_ROOT", root.path())
            .arg("wrote a smoke test")
            .assert()
            .success();

        let journal = root.path().join("journal");
        let entry = std::fs::read_dir(&journal)
            .expect("journal dir should exist under DIANE_DIANE_ROOT")
            .next()
            .expect("journal should have one file")
            .unwrap();
        let text = std::fs::read_to_string(entry.path()).unwrap();
        assert!(text.contains("wrote a smoke test"), "got: {text}");
    }
}
