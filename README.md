# Infinity Engine Mod Installer
      /\/\   ___   __| | (_)_ __  ___| |_ __ _| | | ___ _ __
     /    \ / _ \ / _` | | | '_ \/ __| __/ _` | | |/ _ \ '__|
    / /\/\ \ (_) | (_| | | | | | \__ \ || (_| | | |  __/ |
    \/    \/\___/ \__,_| |_|_| |_|___/\__\__,_|_|_|\___|_|

Automatically install mods from a prepopulated weidu.log file.

## Demo
[mod_installer.webm](https://github.com/dark0dave/mod_installer/assets/52840419/98127744-850e-43a1-a9be-adc078b2a829)

## Usage
```sh
Usage: mod_installer [OPTIONS] --log-file <LOG_FILE> \
  --game-directory <GAME_DIRECTORY> \
  --weidu-binary <WEIDU_BINARY> \
  --mod-directories <MOD_DIRECTORIES>

Options:
  --log-file <LOG_FILE>
      Full path to target log [env: LOG_FILE=]
  -g, --game-directory <GAME_DIRECTORY>
      Full path to game directory [env: GAME_DIRECTORY=]
  -w, --weidu-binary <WEIDU_BINARY>
      Full Path to weidu binary [env: WEIDU_BINARY=]
  -m, --mod-directories <MOD_DIRECTORIES>
      Full Path to mod directories [env: MOD_DIRECTORIES=]
  -l, --language <LANGUAGE>
      Game Language [default: en_US]
  -d, --depth <DEPTH>
      Depth to walk folder structure [env: DEPTH=] [default: 3]
  -s, --skip-installed
      Compare against installed weidu log, note this is best effort [env: SKIP_INSTALLED=]
  -a, --abort-on-warnings
      If a warning occurs in the weidu child process exit [env: ABORT_ON_WARNINGS=]
  -t, --timeout <TIMEOUT>
      Timeout time per mod in seconds, default is 1 hour [env: TIMEOUT=] [default: 3600]
  -h, --help
      Print help
  -V, --version
      Print version
```

## Log levels

Additional information can be shown with:
```sh
RUST_LOG=INFO mod_installer [OPTIONS]
```

For line by line debugging:
```sh
RUST_LOG=DEBUG mod_installer [OPTIONS]
```

To print everything including weidu logs:
```sh
RUST_LOG=TRACE mod_installer [OPTIONS]
```
