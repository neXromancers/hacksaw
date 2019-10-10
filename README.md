## hacksaw lets you select areas of your screen

(on x11)

![screencast](https://user-images.githubusercontent.com/15344581/49049792-67b5d580-f1d8-11e8-871c-74fc8cc72d96.gif)

### Installation

`cargo install --git https://github.com/neXromancers/hacksaw`

(crates.io coming soon)

### Features
- **Guide Lines** to check precise positions and line up before you start a selection
  - just like the popular [Guides](https://github.com/udf/slop-guides) shader for slop
- doesn't instantly quit on first keypress
  - keep typing like a pro while you screenshot your memes
  - *(tiling wm exclusive)* you can still navigate windows while in hacksaw
- select with any mouse button, not just left click!
  - except right click, that's cancel
  - restart selection by scrolling scrollwheel
- you can customise the *colour* and **width** of the lines
  - and you can customise the width of selection and guide lines **separately**!
- did i mention it's written in **RUST**
- *lightweight and fast*
  - not that i've actually run any performance comparisons to slop
- [one of Thor's favorites](https://xkcd.com/2097/)
- built for the most *advanced* and *cutting edge* platform of today, ***X11***

### Stability
- Main functionality is all there and pretty solid
- You may experience bugs when invoking hacksaw while a popup is open
- Pressing escape to exit selection is not yet implemented

### Usage

```
hacksaw 0.1.0
expectocode <expectocode@gmail.com>

USAGE:
    hacksaw [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
    -n, --no-guides    Disable fighter pilot guide lines
    -V, --version      Prints version information

OPTIONS:
    -f, --format <format>
            Output format. You can use %x for x-coordinate, %y for y-coordinate, %w for width, %h for height, %i for
            selected window id, %g as a shorthand for %wx%h+%x+%y (the default, X geometry) and %% for a literal '%'.
            Other %-codes will cause an error. [default: %g]
    -g, --guide-thickness <guide_thickness>          Thickness of fighter pilot guide lines [default: 1]
    -c, --colour <line_colour>
            Hex colour of the lines (RGB or RGBA), '#' optional [default: #7f7f7f]

    -r, --remove-decorations <remove_decorations>
            Number of (nested) window manager frames to try and remove [default: 0]

    -s, --select-thickness <select_thickness>        Thickness of selection box lines [default: 1]
```
