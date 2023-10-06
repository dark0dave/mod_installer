# Infinity Engine Mod Installer
      /\/\   ___   __| | (_)_ __  ___| |_ __ _| | | ___ _ __
     /    \ / _ \ / _` | | | '_ \/ __| __/ _` | | |/ _ \ '__|
    / /\/\ \ (_) | (_| | | | | | \__ \ || (_| | | |  __/ |
    \/    \/\___/ \__,_| |_|_| |_|___/\__\__,_|_|_|\___|_|

Automatically install mods from a prepopulated weidu.log file.

## Demo
![](docs/mod_installer.webm)

## Usage
```sh
Usage: mod_installer [OPTIONS] --log-file <LOG_FILE> \
  --game-directory <GAME_DIRECTORY> \
  --weidu-binary <WEIDU_BINARY> \
  --mod-directories <MOD_DIRECTORIES>

Options:
  --log-file <LOG_FILE>                    Full path to target log [env: LOG_FILE=]
  -g, --game-directory <GAME_DIRECTORY>    Full path to game directory [env: GAME_DIRECTORY=]
  -w, --weidu-binary <WEIDU_BINARY>        Full Path to weidu binary [env: WEIDU_BINARY=]
  -m, --mod-directories <MOD_DIRECTORIES>  Full Path to mod directories [env: MOD_DIRECTORIES=]
  -l, --language <LANGUAGE>                Game Language [default: en_US]
  -h, --help                               Print help
  -V, --version                            Print version
```

## Log levels

Additional information can be shown with:
```sh
RUST_LOG=INFO mod_installer [OPTIONS]
```

For line by line debuging:
```sh
RUST_LOG=DEBUG mod_installer [OPTIONS]
```

To print everything including weidu logs:
```sh
RUST_LOG=TRACE mod_installer [OPTIONS]
```
