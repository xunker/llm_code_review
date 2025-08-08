use clap::{Parser, ArgAction};
use regex::Regex;
use std::process;
use std::process::Command;

use simple_logger::SimpleLogger;
use log::{debug, error, info, trace, warn, LevelFilter}; // Import the logging macros
/*
    Rust log levels:

    #[repr(usize)]
    pub enum Level {
        Error = 1,
        Warn = 2,
        Info = 3,
        Debug = 4,
        Trace = 5,
    }
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

/// Processes code review options
#[derive(Parser, Debug, Default)]
#[command(
    name = "reviewer",
    version = "1.0",
    author = "Your Name <you@example.com>",
    after_help = &format!("Default System Prompt:\n\n{}", Regex::new(r"(?m)^").unwrap().replace_all(DEFAULT_SYSTEM_PROMPT, "  ")),
    about)]
struct Cli {
    /// Add additional context for the review, appended to the system prompt allowing you to use any git diff syntax or options
    #[arg(short, long, value_name = "TEXT")]
    context: Option<String>,

    /// Use something other than the default system prompt
    #[arg(short = 's', long = "system-prompt", value_name = "TEXT")]
    system_prompt: Option<String>,

    /// Number of lines given as context to the LLM
    #[arg(short = 'U', long = "unified", default_value_t=10)]
    unified_context: usize,

    /// Enable verbose output
    #[arg(short, long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Enable debug output (very verbose mode, imples --verbose)
    #[arg(short = 'D', long, action = ArgAction::SetTrue)]
    debug: bool,

    /// Arguments that will be passed in to `git diff`
    #[arg(value_name = "remaining_args", allow_hyphen_values = true)]
    remaining_args: Vec<String>,
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

    SimpleLogger::new()
        .with_level(log_level)
        .init()
        .unwrap();
    // trace!("This message will be shown if the level is Trace or lower.");
    // debug!("This message will be shown if the level is Debug or lower.");
    // info!("This message will be shown if the level is Info or lower.");
    // warn!("This message will be shown if the level is Warn or lower.");
    // error!("This message will be shown if the level is Error or lower.");


    if !cli.remaining_args.is_empty() {
        println!("Remaining arguments: {:?}", &cli.remaining_args);
    }

    if cli.verbose {
        println!("Verbose mode enabled.");
    }

    if cli.debug {
        println!("Debug mode enabled.");
    }

    if let Some(ctx) = &cli.context {
        println!("Context provided: {}", ctx);
    }

    if let Some(prompt) = &cli.system_prompt {
        println!("Using system prompt: {}", prompt);
    }

    println!("Using unified_context: {}", &cli.unified_context);

    let git_args = &format!("-U{} {}", &cli.unified_context, &cli.remaining_args.join(" "));

    let mut binding = Command::new("git");
    let command = binding.arg("diff").arg(git_args.trim());
    debug!("Running command: {:?}", command);
    let output = command.output().expect("");
    let diff_output = &format!("{}", String::from_utf8_lossy(&output.stdout) );

    if !output.status.success() {
        println!("Git diff command failed. Check your arguments:");
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }
    if diff_output.len() == 0 {
        println!("No changes found to review.");
        process::exit(0);
    }

    // I wish there were a simple consistent method to count tokens, but there isn't
    // as far as I can tell, so we're gonna use a poor estimation and keep safely
    // inside the context limit
    let max_tokens = 50_000; // Claude's limit is 100k, this should be a safe amount
    let chars_per_token = 4; // simple approximation

    // Estimate token count and reduce context if needed
    let char_count = diff_output.len();
    let estimated_tokens = char_count / chars_per_token;

    // if estimated_tokens > max_tokens {
    if true {
        // Calculate reduced context
        let reduced_context = &cli.unified_context * max_tokens / estimated_tokens;
        let reduced_context = if reduced_context > 0 {
            reduced_context
        } else {
            1
        };

        info!("Reducing context to {} lines to fit token limits", reduced_context);

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

        if (diff_output.len()/chars_per_token) > max_tokens {
            error!("Diff is too large to process even with minimal context. Try reviewing a smaller set of changes.");
            process::exit(1);
        }


        // # Re-run git diff with reduced context
        // diff_output=$(git diff "${new_git_args[@]}" 2>/dev/null || error "Git diff command failed with reduced context.")
        // fi

        // Re-run git diff with reduced context
        let mut binding2 = Command::new("git");
        // let mut command = binding2.arg("diff").arg(new_git_args.join().trim());
        let mut command = binding2.arg("diff"); //.arg(new_git_args.join().trim());
        let git_args_split: Vec<&str> = git_args.split_whitespace().collect();
        for git_arg in git_args_split.iter() {
            command = command.arg(git_arg);
        }
        debug!("Running command: {:?}", command);
        let output = command.output().expect("");
        let diff_output = &format!("{}", String::from_utf8_lossy(&output.stdout) );
        println!("diff_output");
        println!("{}", diff_output);
        if !output.status.success() {
            println!("Git diff command failed with reduced context:");
            println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
            process::exit(1);
        }
    }

    // prompt=""

    // if [[ -n "$system_prompt" ]]; then
    // prompt="$system_prompt"
    // else
    // # use default prompt instead
    // prompt=$default_system_prompt
    // fi

    // # Add the additional context if provided
    // if [[ -n "$additional_context" ]]; then
    // prompt="$prompt

    // ## Additional Context
    // $additional_context"
    // fi

    // assembled_review_prompt="$prompt

    // # PR Code

    // $diff_output"

    // echo "$assembled_review_prompt"

}
