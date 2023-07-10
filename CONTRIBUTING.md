# How to contribute

Thanks for your interest in contributing to `lillinput`. We follow [GitHub flow]
in this repository.

## Reporting a bug or suggesting a features

Did you find a bug, or have an idea, suggestion, or feature for the project?
Please create an [issue] in the issue tracker. This allows for discussing the
specifics and getting information on how to fix or implement it, as well as
pointing to existing related topics.

The issue tracker is also a good mechanism for finding issues requested by
other individuals or users: it can be a good source of finding out potential
contributions and get an insight on the project's development practices.

## Contributing code

Once you have identified or created an issue:

1. Fork the `master` branch, and work on your changes.
2. Create a [pull request] that contains your work. Please ensure to include
   tests covering your new functionality - this will make the review easier
   and better future-proof your changes.
3. Once issued, the PR will be reviewed by the maintainers. This is meant to
   be a way of discussing and providing feedback - a two-way conversation.
4. Eventually, the pull request will be approved, and your changes will be
   part of the repo.

## Coding conventions

We follow a minimal set of conventions (enforced by the [default workflow]):

* `cargo fmt` as the formatting standard
* `cargo clippy` as the linter
* for all other needs, we default to usual Rust conventions

[GitHub flow]: https://docs.github.com/en/get-started/quickstart/github-flow
[issue]: https://github.com/diego-plan9/lillinput/issues
[pull request]: https://github.com/diego-plan9/lillinput/compare
[default workflow]: https://github.com/diego-plan9/lillinput/blob/master/.github/workflows/default.yml
