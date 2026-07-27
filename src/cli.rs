use clap::Parser;

#[derive(Parser)]
#[command(name = "diane")]
#[command(about = "Note taking app")]
pub struct Cli {
    #[arg(value_parser = validade_text)]
    pub input_text: Option<String>,
}

fn validade_text(s: &str) -> Result<String, String> {
    let trimmed = s.trim();

    if trimmed.is_empty() {
        return Err("The note cannot be empty.".to_string());
    }
    Ok(s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validade_empty_note() {
        let result = validade_text("");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "The note cannot be empty.")
    }

    #[test]
    fn test_validade_space_only_note() {
        let result = validade_text(" ");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "The note cannot be empty.")
    }

    #[test]
    fn test_validade_accepted_note() {
        let result = validade_text(" 18:59 - valid node, nice! ");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), " 18:59 - valid node, nice! ")
    }
}
