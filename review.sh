#!/usr/bin/env bash
#
# Generates a Code Review prompt that works with any AI/LLMs that can operate
# through a "text chat" style interface.
#
# Based on original at
# https://github.com/llimllib/personal_code/blob/master/homedir/.local/bin/review
#
# LICENSE: unlicense. This is free and unencumbered software released into the
# public domain. See unlicense.org for full license.
set -euo pipefail

default_system_prompt="Please review this PR as if you were a senior engineer.

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

Please be specific, constructive, and actionable in your feedback. Output the review in markdown format."

function usage {
  cat <<EOF
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
  llm_code_review --context "This is a Rails 8 app, running inside Microsoft Azure."

  # Review with context from a file
  review --context "\$(cat PR_DESCRIPTION.md)"

  # Set system promp to be something other than the default
  review --system-prompt "\$(cat .github/copilot-instrunctions.md)"
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
EOF
  exit "${1:-0}"
}

git_args=()
has_unified_context=false
context_value=3 # used to be 10
additional_context=""
system_prompt=""
output_filename=""

# Process only our custom arguments, pass everything else to git
while [[ $# -gt 0 ]]; do
  case "$1" in
    -v|--verbose)
      set -x
      shift
      ;;
    -s|--system-prompt)
      shift
      if [[ $# -gt 0 ]]; then
        system_prompt="$1"
        shift
      else
        error "Missing value for --system-prompt option"
      fi
      ;;
    -c|--context)
      shift
      if [[ $# -gt 0 ]]; then
        additional_context="$1"
        shift
      else
        error "Missing value for --context option"
      fi
      ;;
    -U[0-9]*)
      # Form: -U10
      has_unified_context=true
      context_value="${1#-U}"
      git_args+=("$1")
      shift
      ;;
    -U)
      # Form: -U 10
      has_unified_context=true
      shift
      if [[ $# -gt 0 && "$1" =~ ^[0-9]+$ ]]; then
        context_value="$1"
        # normalize to `-U10` to ease our checking later on
        git_args+=("-U$1")
        shift
      else
        error "Missing value for -U option"
      fi
      ;;
    --unified=*)
      # Form: --unified=10
      has_unified_context=true
      context_value="${1#--unified=}"
      git_args+=("$1")
      shift
      ;;
    -h|--help)
      usage
      ;;
    *)
      # Store all other arguments to pass to git diff
      git_args+=("$1")
      shift
      ;;
  esac
done

readonly RED='\033[0;31m'
readonly BLUE='\033[0;34m'
readonly RESET='\033[0m'

info() {
  printf "${BLUE}• %s${RESET}\n" "$1" >&2
}

error() {
  printf "${RED}❌ %s${RESET}\n" "$1" >&2
  usage 1
}

# Default unified context if none specified. The idea here is to increase the
# context (git defaults to 3 lines) so that the LLM has more context for its
# review. Later on we'll check if this generates too much output and shorten it
# if so
if [[ "$has_unified_context" == false ]]; then
  git_args=("-U$context_value" "${git_args[@]}")
fi

# Run git diff
diff_output=$(git diff "${git_args[@]}" 2>/dev/null || error "Git diff command failed. Check your arguments.")

if [[ -z "$diff_output" ]]; then
  error "No changes found to review."
fi

# I wish there were a simple consistent method to count tokens, but there isn't
# as far as I can tell, so we're gonna use a poor estimation and keep safely
# inside the context limit
max_tokens=50000  # Claude's limit is 100k, this should be a safe amount
chars_per_token=4 # simple approximation

# Estimate token count and reduce context if needed
char_count=${#diff_output}
estimated_tokens=$((char_count / chars_per_token))

if [[ $estimated_tokens -gt $max_tokens ]]; then
  # Calculate reduced context
  reduced_context=$((context_value * max_tokens / estimated_tokens))
  reduced_context=$((reduced_context > 0 ? reduced_context : 1))

  info "Reducing context to $reduced_context lines to fit token limits"

  # Replace unified context in git args
  new_git_args=()
  for arg in "${git_args[@]}"; do
    if [[ "$arg" =~ ^-U[0-9]+$ ]]; then
      new_git_args+=("-U$reduced_context")
    elif [[ "$arg" =~ ^--unified=[0-9]+$ ]]; then
      new_git_args+=("--unified=$reduced_context")
    else
      new_git_args+=("$arg")
    fi
  done

  if [[ $((${#diff_output} / chars_per_token)) -gt max_tokens ]]; then
    error "Diff is too large to process even with minimal context. Try reviewing a smaller set of changes."
  fi

  # Re-run git diff with reduced context
  diff_output=$(git diff "${new_git_args[@]}" 2>/dev/null || error "Git diff command failed with reduced context.")
fi

prompt=""

if [[ -n "$system_prompt" ]]; then
  prompt="$system_prompt"
else
  # use default prompt instead
  prompt=$default_system_prompt
fi

# Add the additional context if provided
if [[ -n "$additional_context" ]]; then
  prompt="$prompt

## Additional Context
$additional_context"
fi

assembled_review_prompt="$prompt

# PR Code

$diff_output"

echo "$assembled_review_prompt"
