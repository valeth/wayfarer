# Wayfarer
> A simple TUI application to display Journey savefiles.


## How to install

Simply install it with cargo.

```sh
cargo install --git https://github.com/valeth/wayfarer
```

Make sure the path cargo installs to is in your system's `PATH`.

```sh
wayfarer tui <path-to-savefile>

# or

wayfarer show <path-to-savefile>
```


## TUI Keybindings

| Key | Mode   | Description                                         |
|:---:|:------:| --------------------------------------------------- |
| ESC | Any    | Return to "normal mode"                             |
| q   | Normal | Quits the application                               |
| o   | Normal | Open a new file                                     |
| w   | Normal | Toggle file watcher mode (requires "watch" feature) |