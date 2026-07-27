use diane::app::App;
use diane::cli::GetMatches;
use diane::config::Config;

use std::{error::Error, io};

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = match envy::prefixed("DIANE_").from_env::<Config>() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("diane: bad configuration: {e}");
            std::process::exit(1);
        }
    };

    let _app = App::new(&cfg);
    let _matches = GetMatches();
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
