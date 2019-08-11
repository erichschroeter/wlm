_window-layout-manager_ is a utility program that can automatically move and resize windows based on a profile.

# Constructing a profile

A profile is a list of windows and their respective properties.

Creating a profile file can be done by executing the `ls` command with the `--as-profile` flag.
This will output [JSON](http://json.org/) text that can be saved to a file and modified.

## Bash / CommandPrompt

    window-layout-manager ls --as-profile > myprofile.json

## PowerShell

Redirecting the output in [PowerShell](https://docs.microsoft.com/en-us/powershell/) requires special attention due to the default encoding used by PowerShell.
The JSON profile file needs to be UTF-8 compatible.

_**NOTE:** If the window title has non-ASCII characters this method will likely not work._

    window-layout-manager ls --as-profile | Out-File -Encoding ASCII myprofile.json

# Applying a profile

    window-layout-manager apply myprofile.json

# Customizing a profile

As of _version 1.0_, profiles are pretty bare-bones, offering support for specifying the following properties of a window:

* Position (_x_ and _y_ coordinates of the upper-left corner of the window)
* Dimensions (_width_ and _height_ of the window)

More properties may be added in the future.
