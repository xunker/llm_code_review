// use llm_code_review::review::{run, Cli, get_git_diff, DEFAULT_SYSTEM_PROMPT};
use llm_code_review::review::*;
use clap::Parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        // Use ::parse_from because the default value set
        // via #[arg(default_value_t = 3)] in the Cli struct isn't applied
        // during Cli::default().
        let cli = Cli::parse_from(&["llm_code_review"]);

        assert_eq!(cli.unified_context, 3);
        assert!(!cli.verbose);
        assert!(!cli.debug);
        assert!(!cli.show_system_prompt);
    }

    #[test]
    fn test_prompt_assembly_with_context() {
        let mut cli = Cli::default();
        cli.context = Some("Extra context".to_string());
        let prompt = DEFAULT_SYSTEM_PROMPT.to_string()
            + &format!("\n## Additional Context\n{}\n", cli.context.as_ref().unwrap());
        assert!(prompt.contains("Extra context"));
    }
}
