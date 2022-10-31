# `lillinput`

[![crates.io]](https://crates.io/crates/lillinput)
[![license]](LICENSE)
[![build status]](https://github.com/diego-plan9/lillinput/actions/workflows/default.yml)

## About

`lillinput` is a small utility written in Rust for connecting [`libinput`]
gestures into:
* commands for the [`i3`] tiling window manager IPC interface
* shell commands

Since version `0.3.0`, the project is split into two crates:

### Command-line application

> `libinput-cli`: the command line application.

For the documentation related to the command-line application, please check the
[README.md for the `libinput-cli` crate].

### Library

> `libinput`: the library providing the building blocks.

For the documentation related to the library, please check the
[README.md for the `libinput` crate].

### Project status

Please be aware that this project is in beta, and was started for scratching
a specific itch - allowing three-finger swipe for changing between workspaces
in `i3` under a personal setup. It aims to stay small (hence the [name]) and
biased towards custom needs (and a bit of a Rust playground).

## Compiling

### Dependencies

The following system-level libraries are required by this crate dependencies:

```
sudo aptitude install libudev-dev libinput-dev
```

## Related projects

This create relies heavily on the wonderful work on the [`input`] and [`i3ipc`]
crates (among others) - kudos to their maintainers for making them available.

Outside rust, the following projects provide a more complete solution for using
`libinput` gestures:

* [`libinput-gestures`]
* [`fusuma`]
* [`geebar-libinput`]

## Contributing

Any contribution is welcome, please issue or PR away!

## License

This project is licensed under [BSD-3-Clause].

[BSD-3-Clause]: LICENSE
[`i3`]: https://i3wm.org/
[`libinput`]: https://www.freedesktop.org/wiki/Software/libinput/
[name]: https://en.wikipedia.org/wiki/Lilliput_and_Blefuscu
[README.md for the `libinput-cli` crate]: crates/lillinput-cli
[README.md for the `libinput` crate]: crates/lillinput

[`i3ipc`]: https://github.com/tmerr/i3ipc-rs
[`input`]: https://github.com/Smithay/input.rs

[`libinput-gestures`]: https://github.com/bulletmark/libinput-gestures
[`fusuma`]: https://github.com/iberianpig/fusuma
[`geebar-libinput`]: https://github.com/Coffee2CodeNL/gebaar-libinput

[crates.io]: https://img.shields.io/crates/v/lillinput
[license]: https://img.shields.io/crates/l/lillinput
[build status]: https://github.com/diego-plan9/lillinput/actions/workflows/default.yml/badge.svg
