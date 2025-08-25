LLM Code Review
===============

A model-agnostic way to prepare a code review for an AI/LLM. Very alpha quality.

Based on [an original script](https://github.com/llimllib/personal_code/blob/master/homedir/.local/bin/review) by Bill Mill.

# Two versions

## Bash script

There are two versions available here. One is a Bash script, a modification of the original by Bill
Mill. It is found in [`review.sh`](/.review.sh) in this repo.

## Rust

A Rust 2024 version is found in `src/`. This is a work-in-progress.

## Building

First, install Rust. On MacOS, it's easy with Homebrew: `brew install rust`.

To build a standard "release" executable, then run `make`. This will build an executable as
`target/release/llm_code_review`.

## Installing

The executable can then be installed using `make install`, which will copy the file to `~/bin/`. You
can safely skip this step and copy the file whereever you prefer to any location in your $PATH.

## Running

Run `llm_code_review` to usage details.

## Other helpful commands

`cargo build && target/debug/llm_code_review` to compile and run while testing.

Add packages with `cargo add <package name>`.

# Administrivia

(C) 2025 by Matthew Nielsen (xunker@pyxidis.org).

Released under the terms of [GPL
V3](https://www.gnu.org/licenses/gpl-3.0.en.html) license unless otherwise specified.