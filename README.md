## hacksaw lets you select areas of your screen

(on x11)

![screencast](https://user-images.githubusercontent.com/15344581/77849939-29169d80-71c7-11ea-91c4-7e95a743d54c.gif)

### Installation

#### Dependencies
Before installing, make sure you have the following libraries installed (this list is non-exhaustive):

* `xcb-shape`
* `xcb-xkb`

On systems with `apt`, you should be able to run:

```sh
apt install libxcb-shape0-dev libxcb-xkb-dev
```

#### Once you have the dependencies
Simply run ` cargo install hacksaw ` to install from crates.io.

#### Manual installation alternative
Clone this repo, `cd` into it, and run `cargo install --path .`

#### Nixpkgs
hacksaw is in the [NUR](https://github.com/nix-community/NUR) under [`nexromancers`](https://github.com/neXromancers/nixromancers) as [`nur.repos.nexromancers.pkgs.hacksaw`](https://github.com/neXromancers/nixromancers/blob/master/pkgs/tools/misc/hacksaw/generic.nix).

### Examples
#### Take a screenshot (with [shotgun](https://github.com/neXromancers/shotgun)) of a selection/window and copy to clipboard
```sh
selection=$(hacksaw)  # add hacksaw arguments inside as you wish
shotgun -g "$selection" - | xclip -t 'image/png' -selection clipboard
```

#### Take a screenshot of a selection/window and save to a file
```sh
selection=$(hacksaw)  # add hacksaw arguments inside as you wish
shotgun -g "$selection" screenshot.png
```

#### Record an area of the screen with ffmpeg
```sh
#!/bin/sh
#
# record - record an area of the screen

dir=~/medias/videos/records
current=$(date +%F_%H-%M-%S)

mkdir -p "$dir"

hacksaw -n | {
    IFS=+x read -r w h x y

    w=$((w + w % 2))
    h=$((h + h % 2))

    ffmpeg               \
        -v 16            \
        -r 30            \
        -f x11grab       \
        -s "${w}x${h}"   \
        -i ":0.0+$x,$y"  \
        -preset slow     \
        -c:v h264        \
        -pix_fmt yuv420p \
        -crf 20          \
        "$dir/$current.mp4"
}
```

#### Also: [open a terminal with the selected size and shape (on bspwm)](https://github.com/turquoise-hexagon/dots/blob/896422dd12a/wm/.local/bin/draw)

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

### Usage

```
USAGE:
    hacksaw [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information
    -n, --no-guides    Disable fighter pilot guide lines
    -V, --version      Prints version information

OPTIONS:
    -f, --format <format>
            Output format. You can use:
                  %x for x-coordinate,
                  %y for y-coordinate,
                  %w for width,
                  %h for height,
                  %i for selected window id,
                  %g as a shorthand for %wx%h+%x+%y (X geometry),
                  %% for a literal '%'.
            Other %-codes will cause an error. [default: %g]
    -g, --guide-thickness <guide-thickness>          Thickness of fighter pilot guide lines [default: 1]
    -c, --colour <line-colour>
            Hex colour of the lines (RGB or RGBA), '#' optional [default: #7f7f7f]

    -r, --remove-decorations <remove-decorations>
            Number of (nested) window manager frames to try and remove [default: 0]

    -s, --select-thickness <select-thickness>        Thickness of selection box lines [default: 1]
```
