# README Usage Gap Report

## Goal

Create a Korean Markdown report that explains how to use Thought Castle based on the current README, audits README claims against the current Rust implementation, and states which steps still require manual management.

## Scope

- Compare README commands with `src/main.rs` command routing.
- Check the installed Homebrew binary against the current source build.
- Record implementation gaps and manual responsibilities in Korean.
- Commit and push the completed archive-only work.

## Acceptance Checks

- The report exists as a Korean Markdown file.
- The report lists supported automatic sync providers and manual capture providers.
- The report identifies any README/code/package mismatch.
- `cargo test` passes before push.
