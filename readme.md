LLM Code Review
===============

A model-agnostic way to prepare a code review for an AI/LLM. Very alpha quality.

Based on [an original script](https://github.com/llimllib/personal_code/blob/master/homedir/.local/bin/review) by Bill Mill.

# Building

First, install Rust. On MacOS, it's easy with Homebrew: `brew install rust`.

Then, build the executable with `cargo build`, and run with `target/debug/llm_code_review`.

To build a "release" version (without debugging symbols), use `cargo build --release` which will
place an executable at `target/release/llm_code_review` instead.

## Other helpful commands

`cargo build && target/debug/llm_code_review` to compile and run while testing.

Add packages with `cargo add <package name>`.

# Administrivia

(C) 2025 by Matthew Nielsen (xunker@pyxidis.org).

Released under the terms of [GPL
V3](https://www.gnu.org/licenses/gpl-3.0.en.html) license.