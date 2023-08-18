# Wayfarer
> A simple TUI application to display Journey savefiles.


## How to install

Simply install it with cargo.

```sh
cargo install --git https://github.com/valeth/wayfarer
```

Make sure the path cargo installs to is in your system's `PATH`.

Then simply run it with

```sh
wayfarer
```

Use `--help` to see a list of available options.


## TUI Keybindings

| Key         | Mode   | Description                                         |
|:-----------:|:------:| --------------------------------------------------- |
| ESC         | Any    | Return to "normal mode" or cancel action            |
| Ctrl + q    | Any    | Quits the application                               |
| q           | Normal | Quits the application                               |
| e           | Normal | Enter edit mode                                     |
| o           | Normal | Open a new file                                     |
| r           | Normal | Reload the current file                             |
| w           | Normal | Toggle file watcher mode (requires "watch" feature) |
| h, j, k, l  | Edit   | Move inside the current section                     |
| H, J, K, L  | Edit   | Move between sections                               |
| n, p        | Edit   | Cycle through entry values                          |
| s           | Edit   | Save current edit                                   |
| Enter       | Edit   | Begin editing entry                                 |
| Enter       | Insert | Commit entry edit                                   |
