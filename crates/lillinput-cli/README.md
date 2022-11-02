# `lillinput-cli`

[![crates.io]](https://crates.io/crates/lillinput-cli)

<img src="../../doc/assets/logo.svg" width="64px" alt="lillinput logo">

`lillinput-cli` is the commandline application for connecting [`libinput`]
gestures into:
* commands for the [`i3`] tiling window manager `IPC` interface
* shell commands

## Usage

Upon invocation, `lillinput-cli` will listen to `libinput` events until
stopped. By default, the `i3` action will be enabled, with the `workspace next`
configured for the "three finger right swipe" gesture, and the `workspace prev`
for the "three finger left swipe" gesture.

The full list of options can be retrieved via:

```bash
$ lillinput-cli --help
```

```
...
USAGE:
    lillinput-cli [OPTIONS]

OPTIONS:
    -c, --config-file <CONFIG_FILE>
            Configuration file

    -e, --enabled-action-types <ENABLED_ACTION_TYPES>
            enabled action types [possible values: i3, command]

        --four-finger-swipe-down <FOUR_FINGER_SWIPE_DOWN>
            actions the four-finger swipe down

        --four-finger-swipe-left <FOUR_FINGER_SWIPE_LEFT>
            actions the four-finger swipe left

        --four-finger-swipe-left-down <FOUR_FINGER_SWIPE_LEFT_DOWN>
            actions the four-finger swipe left-down

        --four-finger-swipe-left-up <FOUR_FINGER_SWIPE_LEFT_UP>
            actions the four-finger swipe left-up

        --four-finger-swipe-right <FOUR_FINGER_SWIPE_RIGHT>
            actions the four-finger swipe right

        --four-finger-swipe-right-down <FOUR_FINGER_SWIPE_RIGHT_DOWN>
            actions the four-finger swipe right-down

        --four-finger-swipe-right-up <FOUR_FINGER_SWIPE_RIGHT_UP>
            actions the four-finger swipe right-up

        --four-finger-swipe-up <FOUR_FINGER_SWIPE_UP>
            actions the four-finger swipe up

    -h, --help
            Print help information

    -q, --quiet
            Less output per occurrence

    -s, --seat <SEAT>
            libinput seat

    -t, --threshold <THRESHOLD>
            minimum threshold for displacement changes

        --three-finger-swipe-down <THREE_FINGER_SWIPE_DOWN>
            actions the three-finger swipe down

        --three-finger-swipe-left <THREE_FINGER_SWIPE_LEFT>
            actions the three-finger swipe left

        --three-finger-swipe-left-down <THREE_FINGER_SWIPE_LEFT_DOWN>
            actions the three-finger swipe left-down

        --three-finger-swipe-left-up <THREE_FINGER_SWIPE_LEFT_UP>
            actions the three-finger swipe left-up

        --three-finger-swipe-right <THREE_FINGER_SWIPE_RIGHT>
            actions the three-finger swipe right

        --three-finger-swipe-right-down <THREE_FINGER_SWIPE_RIGHT_DOWN>
            actions the three-finger swipe right-down

        --three-finger-swipe-right-up <THREE_FINGER_SWIPE_RIGHT_UP>
            actions the three-finger swipe right-up

        --three-finger-swipe-up <THREE_FINGER_SWIPE_UP>
            actions the three-finger swipe up

    -v, --verbose
            More output per occurrence

    -V, --version
            Print version information
```

### Configuring the swipe actions

Each `--{number}-finger-swipe-{direction}` argument accepts one or several
"actions", in the form `{type}:{command}`. For example, the following
invocation specifies two actions for the "three finger swipe up" gesture:
moving to the next workspace in `i3`, and creating a file.

```bash
$ lillinput -e i3 -e command --three-finger-swipe-up "i3:workspace next" --three-finger-swipe-up "command:touch /tmp/myfile"
```

Currently, the available action types are `i3` and `command`.

### Using a configuration file

The configuration from the application can be read from a configuration file.
By default, the following sources will be read in order:

1. `/etc/lillinput.toml`
2. `${XDG_HOME}/lillinput/lillinput.toml`
3. `${CWD}/lillinput.toml`

Alternatively, a different file can be specified via the `--config-file`
argument. The configuration files can be partial (as in declaring just specific
options rather than the full range of options), and each option can be
overridden individually by later config files or command line arguments,
falling back to their default values if not provided.

The format of the configuration can be found in the [sample configuration file]:

```toml
verbose = "INFO"
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

## License

This project is licensed under [BSD-3-Clause].

[BSD-3-Clause]: ../../LICENSE
[`i3`]: https://i3wm.org/
[`libinput`]: https://www.freedesktop.org/wiki/Software/libinput/
[sample configuration file]: ../../lillinput.toml.sample

[`i3ipc`]: https://github.com/tmerr/i3ipc-rs
[`input`]: https://github.com/Smithay/input.rs

[crates.io]: https://img.shields.io/crates/v/lillinput-cli
