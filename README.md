# kcom
tui for easy directory-change, based on `tui-rs`

## Usage

1. Build with cargo
2. Put the binary e.g. in `~/bin`
3. Create allias e.g. in .bashrc: `alias kc='~/bin/kcom; source ~/.kcom.cmd'`
4. Run in a terminal: `kc`
5. Go to desired directory:
    * Change selection: Up/Down/PGup/PGdown/Mousewheel
    * Enter directory: Enter/Left-Mouse-Button
    * Go to any directory in current path: Left-Mouse-Button on breadcrumb
    * Add filter: any alphanumeric
    * Reset filter: ESC
    * Quit: Alt-q/Right-Mouse-Button
6. The shell has switched to the new directory
