Contribution Guidelines
====

Repository
----

Code is organized in a [git] repository.
It adheres to [Semantic Versioning] and changes are recorded in file [`CHANGELOG.md`](./CHANGELOG.md) according to the format [Keep a Changelog].

[git]: https://git-scm.com/
[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
[Semantic Versioning]: https://semver.org/spec/v2.0.0.html


Build
----

This project is implemented in Rust and uses [just] as a task runner.
Install it with the following command.

~~~~bash
cargo install just
~~~~

All available tasks are defined inside [`justfile`](./justfile) and their names should be self-explanatory.

~~~~bash
# List available tasks
just

# Check source code format
just check-format

# Check source code best practices
just lint

# Build the project
just build

# Build and run tests
just test

# Build the project in release mode
just build-release

# Generate Debian package archive
just deb

# Audit dependencies
just audit
~~~~

[just]: https://lib.rs/crates/just
