brew install rust

`cargo build` to compile, `target/debug/llm_code_review` to run

./reviewer -c "git diff HEAD~1" -s "Custom prompt" -v