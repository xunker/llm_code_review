use clap::Parser;
use llm_code_review::review::{run, Cli};

fn main() {
    let cli = Cli::parse();
    run(cli);
}
