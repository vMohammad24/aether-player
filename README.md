# Aether Player

This project is yet to be stable and is still under active development.

## LICENSE

Distributed under the **GNU GPLv3** License. See `LICENSE` for more information.

## DEVELOP

If you are on windwos and need to develop this project, you will need to install libmpv:

1. Download the latest release of libmpv-dev
2. Extract the contents of the zip file into `%appdata%/mpv/lib`, rename libmpv-2.dll.a to mpv.lib
3. Extract the libmpv=2.dll file from the zip into `src-tauri/target/debug`
4. Put that path into your system environment variable `PATH`
5. Restart your terminal/IDE to make sure the new PATH is loaded
6. You should now be able to run `bun tuari dev` without linkning issues
