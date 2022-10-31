# lillinput

[![crates.io]](https://crates.io/crates/lillinput)

## Overview

The `lillinput` library provides the building blocks for connecting
`libinput` events with different actions:

```mermaid
flowchart LR
    libinput

    subgraph ::controllers
    Controller([Controller])
    DefaultController
    end

    subgraph ::processor
    Processor([Processor])
    DefaultProcessor
    end

    subgraph actions
    Action([Action])
    I3Action -.-> i3
    CommandAction -.-> command
    end

    libinput -.-> |input::event::Event| Controller
    Controller --> |::events::ActionEvent | Processor
    Processor --> |runs| Action

    DefaultController --> DefaultProcessor
    DefaultProcessor --> I3Action
    DefaultProcessor --> CommandAction
```

## License

This project is licensed under [BSD-3-Clause].

[BSD-3-Clause]: ../../LICENSE

[`i3ipc`]: https://github.com/tmerr/i3ipc-rs
[`input`]: https://github.com/Smithay/input.rs

[crates.io]: https://img.shields.io/crates/v/lillinput
