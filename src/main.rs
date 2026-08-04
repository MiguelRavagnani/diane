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
            if input.capture {
                app.pane = input.entry_mode();

                let mut terminal = ratatui::init();

                run(&mut terminal, &mut app)?;
                ratatui::restore();
            } else {
                println!("Here we run TUI!");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use assert_cmd::Command;

    #[test]
    fn runs() {
        let mut cmd = Command::cargo_bin("diane").unwrap();
        cmd.assert().success();
    }
}
