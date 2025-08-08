use clap::{Parser, ArgAction};
use regex::Regex;
use std::process;
use std::process::Command;


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
    unified_context: i32,

    /// Enable verbose output
    #[arg(short, long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Arguments that will be passed in to `git diff`
    #[arg(value_name = "remaining_args", allow_hyphen_values = true)]
    remaining_args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    if !cli.remaining_args.is_empty() {
        println!("Remaining arguments: {:?}", &cli.remaining_args);
    }

    if cli.verbose {
        println!("Verbose mode enabled.");
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
    let output = command.output().expect("");
    let git_changes = &format!("{}", String::from_utf8_lossy(&output.stdout) );

    if !output.status.success() {
        println!("Git diff command failed. Check your arguments:");
        println!("Stderr: {}", String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }
    if git_changes.len() == 0 {
        println!("No changes found to review.");
        process::exit(0);
    }

    // # I wish there were a simple consistent method to count tokens, but there isn't
    // # as far as I can tell, so we're gonna use a poor estimation and keep safely
    // # inside the context limit
    // max_tokens=50000  # Claude's limit is 100k, this should be a safe amount
    // chars_per_token=4 # simple approximation

    // # Estimate token count and reduce context if needed
    // char_count=${#diff_output}
    // estimated_tokens=$((char_count / chars_per_token))

    // if [[ $estimated_tokens -gt $max_tokens ]]; then
    // # Calculate reduced context
    // reduced_context=$((context_value * max_tokens / estimated_tokens))
    // reduced_context=$((reduced_context > 0 ? reduced_context : 1))

    // info "Reducing context to $reduced_context lines to fit token limits"

    // # Replace unified context in git args
    // new_git_args=()
    // for arg in "${git_args[@]}"; do
    //     if [[ "$arg" =~ ^-U[0-9]+$ ]]; then
    //     new_git_args+=("-U$reduced_context")
    //     elif [[ "$arg" =~ ^--unified=[0-9]+$ ]]; then
    //     new_git_args+=("--unified=$reduced_context")
    //     else
    //     new_git_args+=("$arg")
    //     fi
    // done

    // if [[ $((${#diff_output} / chars_per_token)) -gt max_tokens ]]; then
    //     error "Diff is too large to process even with minimal context. Try reviewing a smaller set of changes."
    // fi

    // # Re-run git diff with reduced context
    // diff_output=$(git diff "${new_git_args[@]}" 2>/dev/null || error "Git diff command failed with reduced context.")
    // fi

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
