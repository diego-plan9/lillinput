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

```bash
$ lillinput --help
```

```
...
USAGE:
    lillinput [OPTIONS]

OPTIONS:
    -c, --config-file <CONFIG_FILE>
            Configuration file

    -e, --enabled-action-types <ENABLED_ACTION_TYPES>
            enabled action types [possible values: i3, command]

        --four-finger-swipe-down <FOUR_FINGER_SWIPE_DOWN>
            actions the four-finger swipe down

        --four-finger-swipe-left <FOUR_FINGER_SWIPE_LEFT>
            actions the four-finger swipe left

        --four-finger-swipe-right <FOUR_FINGER_SWIPE_RIGHT>
            actions the four-finger swipe right

        --four-finger-swipe-up <FOUR_FINGER_SWIPE_UP>
            actions the four-finger swipe up

    -h, --help
            Print help information

    -s, --seat <SEAT>
            libinput seat

    -t, --threshold <THRESHOLD>
            minimum threshold for displacement changes

        --three-finger-swipe-down <THREE_FINGER_SWIPE_DOWN>
            actions the three-finger swipe down

        --three-finger-swipe-left <THREE_FINGER_SWIPE_LEFT>
            actions the three-finger swipe left

        --three-finger-swipe-right <THREE_FINGER_SWIPE_RIGHT>
            actions the three-finger swipe right

        --three-finger-swipe-up <THREE_FINGER_SWIPE_UP>
            actions the three-finger swipe up

    -v, --verbose
            Level of verbosity (additive, can be used up to 3 times)

    -V, --version
            Print version information
```

### Configuring the swipe actions

Each `--{number}-finger-swipe-{direction}` argument accepts one or several
"actions", in the form `{type}:{command}`. For example, the following
invocation specifies two actions for the "three finger swipe up": moving to the
next workspace in `i3`, and creating a file.

```bash
$ lillinput --three-finger-swipe-up "i3:workspace next" --three-finger-swipe-up "command:touch /tmp/myfile"
```

Currently, the available action types are `i3` and `command`.

### Using a configuration file

The configuration from the application can be read from a configuration file.
By default, the following sources will be read in order:

1. `/etc/lillinput.toml`
2. `${XDG_HOME}/lillinput/lillinput.toml`
3. `${CWD}/lillinput.toml`

Alternatively, a different file can be specified via the `--config-file`
argument. If specified, any other command line arguments will take precedence
over values read from the configuration file.

The format of the configuration can be found in the [sample configuration file]:

```toml
verbose = 0
seat = "seat01"
threshold = 20.0
enabled_action_types = ["i3"]

[actions]
three-finger-swipe-right = ["i3:workspace next"]
three-finger-swipe-left = ["i3:workspace prev"]
three-finger-swipe-up = []
three-finger-swipe-down = []
four-finger-swipe-right = []
four-finger-swipe-left = []
four-finger-swipe-up = []
four-finger-swipe-down = []
```

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
[sample configuration file]: lillinput.toml.sample

[`i3ipc`]: https://github.com/tmerr/i3ipc-rs
[`input`]: https://github.com/Smithay/input.rs

[`libinput-gestures`]: https://github.com/bulletmark/libinput-gestures
[`fusuma`]: https://github.com/iberianpig/fusuma
[`geebar-libinput`]: https://github.com/Coffee2CodeNL/gebaar-libinput

[crates.io]: https://img.shields.io/crates/v/lillinput
[license]: https://img.shields.io/crates/l/lillinput
[build status]: https://github.com/diego-plan9/lillinput/actions/workflows/default.yml/badge.svg
