The High Level Design (HLD) intends to present a 10,000 ft view of the components in this program.

```mermaid
flowchart

program[Program] --> windowManager[WindowManager]
windowManager[WindowManager] -- 0..* --> windowState[WindowState]
program[Program] --> config[Config]

subgraph JSON Config
    config[Config] --> window[Window]
end

subgraph Microsoft Windows
    hwnd[HWND]
end

subgraph Linux
    x11[X11 Handle]
    wayland[Wayland Handle]
end

windowState[WindowState] -.-> winapi[winapi crate]
winapi[winapi crate] --> hwnd[HWND]

windowState[WindowState] -.-> x11-dl["x11-dl crate <br>(FUTURE)"]
x11-dl["x11-dl crate <br>(FUTURE)"] --> x11[X11 Handle]

windowState[WindowState] -.-> wayland-client["wayland-client crate <br>(FUTURE)"]
wayland-client["wayland-client crate <br>(FUTURE)"] --> wayland[Wayland Handle]
```

## JSON Config

The `Config` class is intended to encapsulate a layout profile loaded by the program.
A user may have multiple configurations to layout `Window`s in different positions based on what they are doing.

```mermaid
classDiagram
    class Config{
        windows Window[]
    }

    class WindowFlag{
        <<Enum>>
        MINIMIZED
        MAXIMIZED
    }

    class Window{
        title String
        process String
        flags WindowFlag[]
        origin Point : // upper left
        percent_width f64
        percent_height f64
        dimensions Tuple : // x, y, z
    }

    Window "many" o-- Config : contains
    WindowFlag "many" o-- Window : contains
```

### Window class

The `Window` class encapsulates the information to control how a program window will be laid out.
The `title` and `process` are the only attributes used to select a window from the list of windows available to be controlled.

* `title`: the title of window to control
* `process`: the process of window to control
* `flags`: _(optional)_ list of flags to control the window
* `origin`: _(optional)_ the upper left of the window
* `percent_width`: _(optional)_ percentage of the width of the screen to expand/contract to
* `percent_height`: _(optional)_ percentage of the height of the screen to expand/contract to
* `dimensions`: _(optional)_ hard-coded pixel sizes of the window
    * `x`: width of the window
    * `y`: height of the window
    * `z`: _(optional)_ controls the order of the window in scenarios where other windows overlap
