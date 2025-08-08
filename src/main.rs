/*
 * Based on original at
 * https://github.com/llimllib/personal_code/blob/master/homedir/.local/bin/review
*/
use clap::{ArgAction, Parser};
use regex::Regex;
use std::process;
use std::process::Command;

use log::{debug, error, info, trace, warn, LevelFilter};
use simple_logger::SimpleLogger; // Import the logging macros
                                 /*
                                     Rust log levels:

                                         Error = 1,
                                         Warn = 2,
                                         Info = 3,
                                         Debug = 4,
                                         Trace = 5,
                                 */

const DEFAULT_SYSTEM_PROMPT: &str = "Please review this PR as if you were a senior engineer.

## Focus Areas
- Architecture and design decisions
- Potential bugs and edge cases
- Performance considerations
- Security implications
- Code maintainability and best practices
- Test coverage

## Review Format
- Start with a brief summary of the PR purpose and changes
- List strengths of the implementation
- Identify issues and improvement opportunities (ordered by priority)
- Provide specific code examples for suggested changes where applicable

Please be specific, constructive, and actionable in your feedback. Output the review in markdown format.";

const REVIEW_EXAMPLES: &str = r#"Review Examples::
    Review unstaged changes
        llm_code_review

    Review with additional context
        llm_code_review --context "Focus your review on possible authentication bypasses"

    Review with context from a file
        llm_code_review --context "\$(cat PR_DESCRIPTION.md)"

    Set system prompt to be something other than the default
        llm_code_review --system-prompt "\$(cat .github/copilot-instrunctions.md)"
        llm_code_review --system-prompt "Review this code. Talk like a pirate."

    Review staged changes
        llm_code_review --cached

    Review changes between HEAD and main
        llm_code_review main

    Review changes between two branches
        llm_code_review main feature-branch
            OR
        revillm_code_reviewew main..feature-branch

    Review only changes since branch diverged from main
        llm_code_review main...feature-branch

    Review a remote branch
        llm_code_review origin/main..origin/feature-branch

    Limit review to specific files
        llm_code_review main -- src/components/

    Adjust context lines
        llm_code_review -U5 main

Dot Notation:
  - Two dots (A..B): Direct comparison between A and B
  - Three dots (A...B): Compare common ancestor of A and B with B"#;

/// Ask an LLM to review code changes. This tool passes arguments directly to 'git diff',
/// allowing you to use any git diff syntax or options.
#[derive(Parser, Debug, Default)]
#[command(
    name = "llm_code_review",
    version = "1.0",
    author = "Matthew Nielsen <xunker@pyxidis.org>",
    after_help = REVIEW_EXAMPLES,
    about)]
struct Cli {
    /// Add additional context for the review, appended to the system prompt
    #[arg(short, long, value_name = "TEXT")]
    context: Option<String>,

    /// Override the default system prompt
    #[arg(short = 's', long = "system-prompt", value_name = "TEXT")]
    system_prompt: Option<String>,

    /// Print the current default system prompt and exit
    #[arg(short = 'S', long = "show-system-prompt", action = ArgAction::SetTrue)]
    show_system_prompt: bool,

    /// Number of lines given as context to the LLM
    #[arg(short = 'U', long = "unified", default_value_t = 3)]
    unified_context: usize,

    /// Enable verbose output
    #[arg(short, long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Enable debug output (very verbose mode, imples --verbose)
    #[arg(short = 'D', long, action = ArgAction::SetTrue)]
    debug: bool,

    /// Force context to be reduced, for testing
    #[arg(long, action = ArgAction::SetTrue)]
    force_reduced: bool,

    /// Arguments that will be passed in to `git diff`
    #[arg(value_name = "remaining_args", allow_hyphen_values = true)]
    remaining_args: Vec<String>,
}

fn get_git_diff(git_args: &String) -> String {
    let mut command_binding = Command::new("git");
    let command = command_binding.arg("diff").arg(git_args.trim());

    debug!("Running command: {:?}", command);
    let output = command.output().expect("");
    let diff_output = format!("{}", String::from_utf8_lossy(&output.stdout));

    if !output.status.success() {
        error!("Git diff command failed. Check your arguments:");
        error!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }
    if diff_output.len() == 0 {
        println!("No changes found to review.");
        process::exit(0);
    }

    return diff_output;
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.verbose {
        LevelFilter::Info
    } else if cli.debug {
        LevelFilter::Trace
    } else {
        LevelFilter::Warn
    };

    SimpleLogger::new().with_level(log_level).init().unwrap();

    trace!("unified_context: {}", &cli.unified_context);
    if !cli.remaining_args.is_empty() {
        trace!("remaining_args: {:?}", &cli.remaining_args);
    }
    if cli.verbose {
        info!("Verbose mode enabled.");
    }
    if cli.debug {
        trace!("Debug mode enabled.");
    }

    if cli.show_system_prompt {
        println!(
            "Default System Prompt:\n\n{}",
            Regex::new(r"(?m)^")
                .unwrap()
                .replace_all(DEFAULT_SYSTEM_PROMPT, "  ")
        );
        process::exit(0);
    }

    let git_args = &format!(
        "-U{} {}",
        &cli.unified_context,
        &cli.remaining_args.join(" ")
    );

    let mut diff_output = get_git_diff(git_args);

    // I wish there were a simple consistent method to count tokens, but there isn't
    // as far as I can tell, so we're gonna use a poor estimation and keep safely
    // inside the context limit
    let max_tokens = 50_000; // Claude's limit is 100k, this should be a safe amount
    let chars_per_token = 4; // simple approximation

    // Estimate token count and reduce context if needed
    let char_count = diff_output.len();
    let estimated_tokens = char_count / chars_per_token;

    if (estimated_tokens > max_tokens) || cli.force_reduced {
        debug!(
            "estimated_tokens > max_tokens! `{} > {}`. Need to reduce context from {}!",
            estimated_tokens, max_tokens, &cli.unified_context
        );

        // Calculate reduced context

        let reduced_context = &cli.unified_context * max_tokens / estimated_tokens;
        let reduced_context = if reduced_context > 0 {
            reduced_context
        } else {
            1
        };

        info!(
            "Reducing context to {} lines to fit token limits",
            reduced_context
        );

        // Replace unified context in git args
        let mut new_git_args: Vec<String> = vec![];
        let git_args_split: Vec<&str> = git_args.split_whitespace().collect();
        for git_arg in git_args_split.iter() {
            if Regex::new(r"^-U[0-9]+$").unwrap().is_match(git_arg) {
                new_git_args.push(format!("-U{}", reduced_context));
            } else if Regex::new(r"^--unified=[0-9]+$").unwrap().is_match(git_arg) {
                new_git_args.push(format!("--unified={}", reduced_context));
            } else {
                new_git_args.push(git_arg.to_string());
            }
        }

        if (diff_output.len() / chars_per_token) > max_tokens {
            trace!(
                "diff_output.len()/chars_per_token) > max_tokens : `({}/{} == {}) > {}`",
                diff_output.len(),
                chars_per_token,
                diff_output.len() / chars_per_token,
                max_tokens
            );
            error!("Diff is too large to process even with minimal context. Try reviewing a smaller set of changes.");
            process::exit(1);
        }

        // Re-run git diff with reduced context
        diff_output = get_git_diff(&new_git_args.join(" "));
    }

    let mut prompt = DEFAULT_SYSTEM_PROMPT.to_string();

    if let Some(custom_system_prompt) = &cli.system_prompt {
        prompt = custom_system_prompt.to_string();
    }

    // Add the additional context if provided
    if let Some(ctx) = &cli.context {
        prompt = prompt.to_owned() + &format!("\n## Additional Context\n{}\n", ctx).to_string();
    }

    let assembled_review_prompt = format!("{}\n\n# PR Code\n\n{}", prompt, diff_output);

    println!("{}", assembled_review_prompt);
}
