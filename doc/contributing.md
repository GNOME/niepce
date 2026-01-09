# Contributing guide

This document will guide you into contributing to Niepce.

Please also read the [design philosophy](design_philosophy.md) to guide
about what kind of submission would be accepted.

## AI Policy

Using AI/ChatBot/LLM to contribute code or documentation is not
allowed. They often produce low quality code and represent a risk
about ownership since the training data isn't clear, non-withstanding
all the ethical issues of the technology between being used for labour
oppression, excessive energy usage, excessive water usage, and unfair
labour practice towards all the humans that are required for the
training.

## Building

See [building](building.md).

## Code

Please see the [code oragnization](code-organization.md) for details
on how things are organised.

Code is in Rust.

Each commit must pass `cargo fmt`. It's enforced at the MR level.

Each commit must avoid adding clippy warnings. The `clippy` target in
meson is available to check this.

Tests must pass. Use the `test` target in meson.

### Modules

Modules are in their own file as per [Rust
documentation](https://doc.rust-lang.org/rust-by-example/mod/split.html):

```
$ tree .
.
├── my
│   ├── inaccessible.rs
│   └── nested.rs
├── my.rs
└── split.rs
```

Don't use the form `module_name/mod.rs`

## Foreign code

Dedpending on some external code is useful. Currently Niepce depends
on a certain about of Rust crates, and on a fork of RawTherapee
rtengine which is in C++. The latter fork is a necessity to be able to
build it. It presents minimal changes.

## Submitting

Commit messages must be clear and descriptive. If they resolve a
reported issue (even partially) the issue must be linked in the commit
message.

If possible, slice a MR into smaller independent commit. The
requirement is that the commit pass the checks and doesn't break. They
shouldn't have "drive-by" fixes (ie unrelated). While these are
welcome they should be in their own commits.

Avoid merge commits. Please use `git rebase`.

## Branches

TBD (at the time of writing there is no branch)
