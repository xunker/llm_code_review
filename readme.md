brew install rust

`cargo build && target/debug/llm_code_review` to compile and run

./reviewer -c "git diff HEAD~1" -s "Custom prompt" -v

--

# misc

`RUST_LOG=info` to set log level

`cargo add <package name>`