use llm_code_review::review::{run, Cli};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    run(cli);
}
