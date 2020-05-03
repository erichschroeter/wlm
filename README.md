![Continuous Integration](https://github.com/erichschroeter/window-layout-manager/workflows/Continuous%20Integration/badge.svg)

**wlm** is a command-line tool to move and resize windows based on a config.

# Rationale

**wlm** started as an open source replacement to http://www.stefandidak.com/windows-layout-manager/ and a way for me to get more experience with [Rust](https://www.rust-lang.org/).
The intent is to provide cross-platform support, with Windows being the first to be implemented.

# Installation

    cargo install wlm

# How to Use

## Create a config

A config is a list of windows and their respective properties.

    wlm init

By default, this will create a `default.json` in your platform-specific location.
This is determined by the [directories](https://crates.io/crates/directories) library.
Multiple configs can be created by using the `--file` argument.

## View available window information

Once a window is in a state you like, get its information via the `ls` command.

    wlm ls

## Modify a config

The following example will create a new window in your config:

    wlm add --process "chrome.exe" -x 0 -y 0 -w 800 -H 600

And now that, that window exists it can be modified via the following example:

    wlm config windows.0.process "chrome.exe"
    wlm config windows.0.x 0
    wlm config windows.0.y 0
    wlm config windows.0.w 800
    wlm config windows.0.h 600

To view your existing config:

    wlm config

## Apply a config

Now to apply the config:

    wlm apply

To apply a different config, simply specify it with the `--file` argument.

# TODO

- [x] Add support for default config location
    - `$HOME/.config/wlm/default.json`
- [x] Add support for modifying existing config via CLI
    - e.g. `wlm config windows.2.process "chrome.exe"`
- [ ] Add support for specifying monitor
- [ ] Add support for percentage-based configs
    - e.g. similar to how tiling window managers work or Windows snapping
- [ ] Add support for a daemon/service that will auto apply config on new window events
