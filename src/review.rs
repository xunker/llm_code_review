use clap::builder::PossibleValuesParser;
use clap::{ArgAction, Parser};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn, LevelFilter};
use regex::Regex;
use simple_logger::SimpleLogger;
use std::process::{self, Command}; // Import the logging macros
                                   /*
                                       Rust log levels:

                                           Error = 1,
                                           Warn = 2,
                                           Info = 3,
                                           Debug = 4,
                                           Trace = 5,
                                   */

pub const DEFAULT_SYSTEM_PROMPT: &str = include_str!("rsc/default_system_prompt.txt");

pub const REVIEW_EXAMPLES: &str = include_str!("rsc/review_examples.txt");

/// Ask an LLM to review code changes. This tool passes arguments directly to 'git diff',
/// allowing you to use any git diff syntax or options.
#[derive(Parser, Debug, Default)]
#[command(
    name = "llm_code_review",
    version = "1.0",
    author = "Matthew Nielsen <xunker@pyxidis.org>",
    after_help = REVIEW_EXAMPLES,
    about)]
pub struct Cli {
    /// Add additional context for the review, appended to the system prompt
    #[arg(short, long, value_name = "TEXT")]
    pub context: Option<String>,

    /// Override the default system prompt
    #[arg(short = 's', long = "system-prompt", value_name = "TEXT")]
    system_prompt: Option<String>,

    /// Print the current default system prompt and exit
    #[arg(short = 'S', long = "show-system-prompt", action = ArgAction::SetTrue)]
    pub show_system_prompt: bool,

    /// Request review output be in a specific format
    #[arg(short = 'F', long = "output-format", value_name = "FORMAT", value_parser = PossibleValuesParser::new(["markdown", "asciidoc", "mediawiki"]))]
    pub output_format: Option<String>,

    /// Number of lines given as context to the LLM
    #[arg(short = 'U', long = "unified", default_value_t = 3)]
    pub unified_context: usize,

    /// Enable verbose output
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub verbose: bool,

    /// Enable debug output (very verbose mode, imples --verbose)
    #[arg(short = 'D', long, action = ArgAction::SetTrue)]
    pub debug: bool,

    /// Force context to be reduced, for testing
    #[arg(long, action = ArgAction::SetTrue)]
    force_reduced: bool,

    /// Arguments that will be passed in to `git diff`
    #[arg(value_name = "remaining_args", allow_hyphen_values = true)]
    remaining_args: Vec<String>,
}

#[derive(Debug)]
enum OutputFormat {
    Markdown,
    AsciiDoc,
    MediaWiki,
}

impl OutputFormat {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "markdown" => Some(OutputFormat::Markdown),
            "asciidoc" => Some(OutputFormat::AsciiDoc),
            "mediawiki" => Some(OutputFormat::MediaWiki),
            _ => None,
        }
    }
}

pub fn get_git_diff(git_args: &String) -> String {
    let mut command_binding = Command::new("git");
    let command = command_binding.arg("diff");

    // Split git_args by space and append them each to command as arg()
    let git_args_split: Vec<&str> = git_args.trim().split_whitespace().collect();
    for git_arg in git_args_split.iter() {
        command.arg(git_arg);
    }

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

fn reduce_context_if_needed(
    git_args: &[String],
    unified_context: usize,
    force_reduced: bool,
    diff_output: &str,
    max_tokens: usize,
    chars_per_token: usize,
) -> Option<Vec<String>> {
    // Estimate token count and reduce context if needed
    let estimated_tokens = diff_output.len() / chars_per_token;

    if estimated_tokens <= max_tokens && !force_reduced {
        return None;
    }

    let reduced_context = (unified_context * max_tokens / estimated_tokens).max(1);

    info!(
        "Reducing context to {} lines to fit token limits",
        reduced_context
    );

    let new_git_args: Vec<String> = git_args
        .iter()
        .map(|arg| {
            if arg.starts_with("-U") {
                format!("-U{}", reduced_context)
            } else if arg.starts_with("--unified=") {
                format!("--unified={}", reduced_context)
            } else {
                arg.clone()
            }
        })
        .collect();

    let new_estimated_tokens = diff_output.len() / chars_per_token;
    if new_estimated_tokens > max_tokens {
        error!("Diff is too large to process even with minimal context. Try reviewing a smaller set of changes.");
        process::exit(1);
    }

    Some(new_git_args)
}

pub fn run(cli: Cli) {
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
        // Indent the each line of the prompt by two spaces
        println!(
            "Default System Prompt:\n\n{}",
            Regex::new(r"(?m)^")
                .unwrap()
                .replace_all(DEFAULT_SYSTEM_PROMPT, "  ")
        );
        process::exit(0);
    }

    // I wish there were a simple consistent method to count tokens, but there isn't
    // as far as I can tell, so we're gonna use a poor estimation and keep safely
    // inside the context limit
    let max_tokens = 50_000; // Claude's limit is 100k, this should be a safe amount
    let chars_per_token = 4; // simple approximation

    let git_args_vec: Vec<String> = vec![
        format!("-U{}", cli.unified_context),
        cli.remaining_args.join(" "),
    ];

    let mut diff_output = get_git_diff(&git_args_vec.join(" "));

    if let Some(new_args) = reduce_context_if_needed(
        &git_args_vec,
        cli.unified_context,
        cli.force_reduced,
        &diff_output,
        max_tokens,
        chars_per_token,
    ) {
        diff_output = get_git_diff(&new_args.join(" "));
    }

    let mut prompt = DEFAULT_SYSTEM_PROMPT.to_string();

    if let Some(custom_system_prompt) = &cli.system_prompt {
        prompt = custom_system_prompt.to_string();
    }

    // Add instructions about the output format to the end of the main prompt
    let format = &cli
        .output_format
        .as_ref()
        .and_then(|s| OutputFormat::from_str(s));
    if let Some(output_format) = format {
        prompt.push_str(&format!("\nOutput the review in {:?} format.\n", output_format));
    }

    // Add the additional context if provided
    if let Some(ctx) = &cli.context {
        prompt.push_str(&format!("\n## Additional Context\n{}\n", ctx));
    }

    let assembled_review_prompt = format!("{}\n\n# PR Code\n\n{}", prompt, diff_output);

    println!("{}", assembled_review_prompt);
}
