# lillinput

[![crates.io]](https://crates.io/crates/lillinput)
[![license]](LICENSE)
[![build status]](https://github.com/diego-plan9/lillinput/actions/workflows/default.yml)


## About

`lillinput` is a small utility written in Rust for connecting [`libinput`]
gestures into:
* commands for the [`i3`] tiling window manager IPC interface
* shell commands

### Project status

Please be aware that this project is in beta, and was started for scratching
a specific itch - allowing three-finger swipe for changing between workspaces
in `i3` under a personal setup. It aims to stay small (hence the [name]) and
biased towards custom needs (and a bit of a Rust playground).

## Usage

Upon invocation, `lillinput` will listen to `libinput` events until stopped. By
default, the `i3` action will be enabled, with the `workspace next` configured
for the "three finger right swipe" gesture, and the `workspace prev` for the
"three finger left swipe" gesture.

The full list of options can be retrieved via:

```
lillinput --help
```

```
...
USAGE:
    lillinput [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Level of verbosity (additive, can be used up to 3 times)
    -V, --version    Print version information

OPTIONS:
    -c, --config-file <CONFIG_FILE>         Configuration file
    -e, --enabled-action-types <ENABLED_ACTION_TYPES>...
            enabled action types [possible values: i3, command]
    -s, --seat <SEAT>                       libinput seat
        --swipe-down-3 <SWIPE_DOWN_3>...    actions the three-finger swipe down
        --swipe-down-4 <SWIPE_DOWN_4>...    actions the four-finger swipe down
        --swipe-left-3 <SWIPE_LEFT_3>...    actions the three-finger swipe left
        --swipe-left-4 <SWIPE_LEFT_4>...    actions the four-finger swipe left
        --swipe-right-3 <SWIPE_RIGHT_3>...  actions the three-finger swipe right
        --swipe-right-4 <SWIPE_RIGHT_4>...  actions the four-finger swipe right
        --swipe-up-3 <SWIPE_UP_3>...        actions the three-finger swipe up
        --swipe-up-4 <SWIPE_UP_4>...        actions the four-finger swipe up
    -t, --threshold <THRESHOLD>             minimum threshold for displacement changes
```

### Configuring the swipe actions

Each `--swipe-{foo}` argument accepts one or several "actions", in the form
`{type}:{command}`. For example, the following invocation specifies two actions
for the "three finger swipe up": moving to the next workspace in `i3`, and
creating a file.

```
lillinput --swipe-up-3 "i3:workspace next" --swipe-up-3 "command:touch /tmp/myfile"
```

Currently, the available action types are `i3` and `command`.

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

[BSD-3-Clause]: LICENSE.txt
[`i3`]: https://i3wm.org/
[`libinput`]: https://www.freedesktop.org/wiki/Software/libinput/
[name]: https://en.wikipedia.org/wiki/Lilliput_and_Blefuscu

[`i3ipc`]: https://github.com/tmerr/i3ipc-rs
[`input`]: https://github.com/Smithay/input.rs

[`libinput-gestures`]: https://github.com/bulletmark/libinput-gestures
[`fusuma`]: https://github.com/iberianpig/fusuma
[`geebar-libinput`]: https://github.com/Coffee2CodeNL/gebaar-libinput

[crates.io]: https://img.shields.io/crates/v/lillinput
[license]: https://img.shields.io/crates/l/lillinput
[build status]: https://github.com/diego-plan9/lillinput/actions/workflows/default.yml/badge.svg
