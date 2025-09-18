
LLM Code Review
===============

A model-agnostic way to prepare a code review for an AI/LLM. Very alpha quality.

Based on [an original script](https://github.com/llimllib/personal_code/blob/master/homedir/.local/bin/review) by Bill Mill.


There are two versions available here. One is a Bash script, a modification of the original by Bill
Mill. It is found in [`review.sh`](/.review.sh) in this repo.

# Table of Contents

<!-- TOC -->

- [LLM Code Review](#llm-code-review)
- [Table of Contents](#table-of-contents)
  - [Bash Script Version](#bash-script-version)
  - [Rust Version](#rust-version)
    - [Building](#building)
    - [Installing](#installing)
    - [Usage](#usage)
    - [Other helpful commands](#other-helpful-commands)
  - [Workflow Example](#workflow-example)
  - [Administrivia](#administrivia)

<!-- /TOC -->

## Bash Script Version

Run `review.sh --help` to see usage details.

```
review [--verbose] [--context TEXT] [--help] [git-diff-arguments...]

Ask an LLM to review code changes. This tool passes arguments directly to 'git diff',
allowing you to use any git diff syntax or options.

Options:
  -c, --context       TEXT  Add additional context for the review, appended to the system prompt
  -s, --system-prompt TEXT  Use something other than the default system prompt
  -h, --help          Show this help message
  -v, --verbose       Enable verbose output

Review Examples:
  # Review unstaged changes
  review

  # Review with additional context
  review --context "Focus your review on possible authentication bypasses"

  # Review with context from a file
  review --context "$(cat PR_DESCRIPTION.md)"

  # Set system promp to be something other than the default
  review --system-prompt "$(cat .github/copilot-instrunctions.md)"
  review --system-prompt "Review this code. Talk like a pirate."

  # Review staged changes
  review --cached

  # Review changes between HEAD and main
  review main

  # Review changes between two branches
  review main feature-branch
  # OR
  review main..feature-branch

  # Review only changes since branch diverged from main
  review main...feature-branch

  # Review a remote branch
  review origin/main..origin/feature-branch

  # Limit review to specific files
  review main -- src/components/

  # Adjust context lines
  review -U5 main

Dot Notation:
  - Two dots (A..B): Direct comparison between A and B
  - Three dots (A...B): Compare common ancestor of A and B with B

Depends on:
- bat: https://github.com/sharkdp/bat (optional)
```

## Rust Version

A Rust 2024 version is found in `src/`. This is a work-in-progress.

### Building

First, install Rust. On MacOS, it's easy with Homebrew: `brew install rust`.

To build a standard "release" executable, then run `make`. This will build an executable as
`target/release/llm_code_review`.

### Installing

The executable can then be installed using `make install`, which will copy the file to `~/bin/`. You
can safely skip this step and copy the file whereever you prefer to any location in your $PATH.

### Usage

Run `llm_code_review --help` to see usage details.

```
Ask an LLM to review code changes. This tool passes arguments directly to 'git diff', allowing you to use any git diff syntax or options

Usage: llm_code_review [OPTIONS] [remaining_args]...

Arguments:
  [remaining_args]...  Arguments that will be passed in to `git diff`

Options:
  -c, --context <TEXT>             Add additional context for the review, appended to the system prompt
  -s, --system-prompt <TEXT>       Override the default system prompt
  -S, --show-system-prompt         Print the current default system prompt and exit
  -F, --output-format <FORMAT>     Request review output be in a specific format [possible values: markdown, asciidoc, mediawiki]
  -U, --unified <UNIFIED_CONTEXT>  Number of lines given as context to the LLM [default: 3]
  -v, --verbose                    Enable verbose output
  -D, --debug                      Enable debug output (very verbose mode, imples --verbose)
      --force-reduced              Force context to be reduced, for testing
  -h, --help                       Print help
  -V, --version                    Print version

Review Examples::
    Review unstaged changes
        llm_code_review

    Review with additional context
        llm_code_review --context "Focus your review on possible authentication bypasses"
        llm_code_review --context "This is a Rails 8 app, running inside Microsoft Azure."

    Review with context from a file
        llm_code_review --context "$(cat PR_DESCRIPTION.md)"

    Set system prompt to be something other than the default
        llm_code_review --system-prompt "$(cat .github/copilot-instrunctions.md)"
        llm_code_review --system-prompt "Please review this code while talking like a pirate."

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
  - Three dots (A...B): Compare common ancestor of A and B with B
```

### Other helpful commands

`cargo build && target/debug/llm_code_review` to compile and run while testing.

Add packages with `cargo add <package name>`.

## Workflow Example

Any of the below examples can use `llm_code_review` or `review.sh`.

To build a review prompt based on my _current working state_ vs main branch, and copy the prompt to
the clipboard:

```bash
# MacOS
llm_code_review main | pbcopy

# Linux - using "xclip"
llm_code_review main | xclip

# Windows
llm_code_review main | clip
```

To use a custom system prompt instead of the default system prompt:

```bash
llm_code_review --system-prompt "$(cat .github/copilot-instrunctions.md)"
```

## Administrivia

(C) 2025 by Matthew Nielsen (xunker@pyxidis.org).

Released under the terms of [GPL
V3](https://www.gnu.org/licenses/gpl-3.0.en.html) license unless otherwise specified.