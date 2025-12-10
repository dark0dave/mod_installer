  # Infinity Engine Mod Installer
[![](./docs/rust.svg)](https://www.rust-lang.org/tools/install)
[![](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)](https://github.com/dark0dave/mod_installer/releases/latest)
[![](https://img.shields.io/badge/Windows-0078D6?&style=for-the-badge&logoColor=white&logo=git-for-windows)](https://github.com/dark0dave/mod_installer/releases/latest)
[![](https://img.shields.io/badge/mac%20os-grey?style=for-the-badge&logo=apple&logoColor=white)](https://github.com/dark0dave/mod_installer/releases/latest)
[![](https://img.shields.io/github/actions/workflow/status/dark0dave/mod_installer/main.yaml?style=for-the-badge)](https://github.com/dark0dave/mod_installer/actions/workflows/main.yaml)
[![](https://img.shields.io/github/license/dark0dave/mod_installer?style=for-the-badge)](./LICENSE)

      /\/\   ___   __| | (_)_ __  ___| |_ __ _| | | ___ _ __
     /    \ / _ \ / _` | | | '_ \/ __| __/ _` | | |/ _ \ '__|
    / /\/\ \ (_) | (_| | | | | | \__ \ || (_| | | |  __/ |
    \/    \/\___/ \__,_| |_|_| |_|___/\__\__,_|_|_|\___|_|

The Infinity Engine Mod Installer is a tool designed to automate the installation of mods for Infinity Engine games such as Baldur's Gate, Icewind Dale, and Planescape: Torment. It uses a file called "weidu.log" to determine which mods to install and how to install them.

## Usage

To use the Infinity Engine Mod Installer, you need to run it from the command line. Here's the basic structure of the command:

```sh
Usage: mod_installer(.exe) <COMMAND> [OPTIONS]

Commands:
  normal, -n  Normal install for (BG1EE,BG2EE,IWDEE) (STABLE)
  eet, -e     EET install for (eet) (ALPHA)
  help        Print this message or the help of the given subcommand(s)
```
Let's break down what each part means:

* mod_installer(.exe): This is the name of the program you're running.
[OPTIONS]: These are additional settings you can use to customize how the program works (we'll explain these in detail below).

## Born2BSalty Windows guide

[![Watch the video](./docs/guide.webp)](https://raw.githubusercontent.com/dark0dave/mod_installer/main/docs/guide.webm)


## Normal Install

```sh
Usage: mod_installer {normal|-n} [OPTIONS] --log-file <LOG_FILE> --game-directory <GAME_DIRECTORY>

Options:
  -f, --log-file <LOG_FILE>
          Path to target log [env: LOG_FILE=]
  -g, --game-directory <GAME_DIRECTORY>
          Absolute Path to game directory [env: GAME_DIRECTORY=]
  -w, --weidu-binary <WEIDU_BINARY>
          Absolute Path to weidu binary [env: WEIDU_BINARY=] [default: /home/x/projects/bg_mods/tools/weidu/weidu]
  -m, --mod-directories <MOD_DIRECTORIES>
          Path to mod directories [env: MOD_DIRECTORIES=] [default: /home/x/projects/bg_mods/tools/mod_installer]
  ...
  -h, --help                             Print help
```

* --log-file <LOG_FILE>: This is where you tell the program where to find the "weidu.log" file, which contains the mods you want to install, please note this is not the same as the weidu.log file in the game directory.

* --game-directory <GAME_DIRECTORY>: This is where you tell the program where your game is installed.

* --mod-directories <MOD_DIRECTORIES>: This is where you tell the program where to find the mod files. This defaults to the current workgin directory

### Example Run

With binary or installed from cargo
```sh
mod_installer(.exe) --log-file dark0dave_weidu.log --game-directory ~/.steam/steam/steamapps/common/Baldur\'s\ Gate\ Enhanced\ Edition/ --mod-directories ~/Downloads/my_mods/
```

With cargo:
```sh
cargo run -- normal --log-file dark0dave_weidu.log --game-directory ~/.steam/steam/steamapps/common/Baldur\'s\ Gate\ Enhanced\ Edition/ --mod-directories ~/Downloads/my_mods/
```

## ALPHA: EET Install

This automation exists purely to help install EET, as you end up juggling two directories and weidu mod lists.


```sh
Usage: mod_installer {eet|-e} [OPTIONS] --bg1-game-directory <BG1_GAME_DIRECTORY> --bg1-log-file <BG1_LOG_FILE> --bg2-game-directory <BG2_GAME_DIRECTORY> --bg2-log-file <BG2_LOG_FILE> --mod-directories <MOD_DIRECTORIES>

Options:
  -1, --bg1-game-directory <BG1_GAME_DIRECTORY>
          Absolute Path to bg1ee game directory [env: BG1_GAME_DIRECTORY=]
  -y, --bg1-log-file <BG1_LOG_FILE>
          Path to bg1ee weidu.log file [env: BG1_LOG_FILE=]
  -2, --bg2-game-directory <BG2_GAME_DIRECTORY>
          Absolute Path to bg2ee game directory [env: BG2_GAME_DIRECTORY=]
  -z, --bg2-log-file <BG2_LOG_FILE>
          Path to bg2ee weidu.log file [env: BG2_LOG_FILE=]
  -w, --weidu-binary <WEIDU_BINARY>
          Absolute Path to weidu binary [env: WEIDU_BINARY=] [default: /home/x/projects/bg_mods/tools/weidu/weidu]
  -m, --mod-directories <MOD_DIRECTORIES>
          Path to mod directories [env: MOD_DIRECTORIES=] [default: /home/x/projects/bg_mods/tools/mod_installer]
  ...
  -h, --help Print help
```

* --bg1-log-file <LOG_FILE>: This is where you tell the program where to find the "weidu.log" file, which contains the mods you want to install, as the first part of the EET process. This is often referred to as the pre-EET section.

* --bg2-log-file <LOG_FILE>: Same as bg1-log-file, but for the bg2 part of EET, ie the log which is installed after the bg1 log is installed.

* --bg1-game-directory <GAME_DIRECTORY>: This is where you tell the program where your game is installed.

* --bg2-game-directory <GAME_DIRECTORY>: This is where you tell the program where your EET part (bg2) to be installed.

* --mod-directories <MOD_DIRECTORIES>: This is where you tell the program where to find the mod files. This defaults to the current workgin directory

## ALPHA: Download feature

This automation exists purely to help the install process. Currently its very limited. Users can supply the flag `--download` followed by the associated github zip url.

```sh
Usage: mod_installer -{e|n} ... --download
```

If a mod is missing users will be presented with the following prompt:

```sh
[2025-06-14T14:08:34Z INFO  mod_installer::utils] Missing mod: Component {
        tp_file: "SETUP-EEFIXPACK.TP2",
        name: "eefixpack",
        lang: "0",
        component: "0",
        component_name: "Core Fixes",
        sub_component: "",
        version: "Nightly 031225",
    }
[2025-06-14T14:08:34Z INFO  mod_installer::utils] Please provide mod url, or exit
```

Users can then input the required github zip url, and the mod_installer will download extract and install the provided mod.


#### Notes

- Only github is supported (more to come I promise!)
- No version check is performed, ie it downloads what ever is passed and that must be a zip file


## FAQ

The Infinity Engine Mod Installer looks at a "weidu.log" file that you provide. This file contains information about mods you want to install. The tool then goes through this list and installs each mod automatically. This saves you time and effort, as you don't have to manually install each mod one by one.

### Weidu Log

The Weidu log file contains a list of installed mods and is typically found in your game directory if you have previously installed mods. Here's an example of what a Weidu log might look like:

```sh
// Log of Currently Installed WeiDU Mods
// The top of the file is the 'oldest' mod
// ~TP2_File~ #language_number #component_number // [Subcomponent Name -> ] Component Name [ : Version]
~TEST_MOD_NAME_1/TEST.TP2~ #0 #0 // test mod one
```
If you're new to modding Infinity Engine games, we recommend installing mods manually first to familiarize yourself with the process. This will help you understand how mods work and how they interact with your game.

### Getting Started with Weidu Logs

If you're looking for an example `weidu.log` to get started:

Check online forums and modding communities. Experienced players and modders often share their mod lists and corresponding Weidu logs.
Look for "mod packs" or "recommended mod lists" for your specific game. These often come with pre-configured Weidu logs.
Start with a small number of popular mods and gradually build up your log as you become more comfortable with the modding process.
Some mod managers for Infinity Engine games can generate Weidu logs based on your selected mods.

Remember, Weidu is a powerful tool, but it's important to understand what you're installing. Always back up your game files before installing mods, and be aware that some mods may conflict with others.

### Demo
We have a short video that shows how the tool works:

![](docs/mod_installer.gif)

### What options can I use?

**Don't panic** you can use the help command to find all the options listed below:

* -h, --help

  What it does: This shows a help message with information about how to use the program.
  How to use it: Just add this option to your command if you need help.
  Example: mod_installer --help

#### Normal Install

```sh
mod_installer(.exe) normal --help
```

Here's a detailed explanation of all the options you can use, for a normal install:

* -f, --log-file <LOG_FILE>

  > What it does: This tells the program where to find the "weidu.log" file.

  > How to use it: Replace <LOG_FILE> with the path to your "weidu.log" file.

  > Example: --log-file C:\Games\Baldur's Gate\weidu.log


* -g, --game-directory <GAME_DIRECTORY>


  > What it does: This tells the program where your game is installed.

  > How to use it: Replace <GAME_DIRECTORY> with the path to your game folder.

  > Example: --game-directory C:\Games\Baldur's Gate


#### EET Install

```sh
mod_installer(.exe) eet --help
```
* -1, --bg1-game-directory <BG1_GAME_DIRECTORY>

  > What it does: This tells the program where your bg1ee is installed.

  > How to use it: Replace <BG1_GAME_DIRECTORY> with the path to your game folder.

  > Example: --bg1-game-directory C:\Games\Baldur's Gate Enhanced Edition

* -y, --bg1-log-file <BG1_LOG_FILE>


  > What it does: This tells the program where to find the "weidu.log" file. For the BG1/pre-EET Install.

  > How to use it: Replace <BG1_LOG_FILE> with the path to your "weidu.log" file.

  > Example: --bg1-log-file C:\pre-eet-weidu.log

* -2, --bg2-game-directory <BG2_GAME_DIRECTORY>

  > What it does: This tells the program where your bg2ee is installed.

  > How to use it: Replace <BG1_GAME_DIRECTORY> with the path to your game folder.

  > Example: --bg2-game-directory C:\Games\Baldur's Gate II Enhanced Edition

* -z, --bg2-log-file <BG2_LOG_FILE>

  > What it does: This tells the program where to find the "weidu.log" file. For the BG2EE/EET Install.

  > How to use it: Replace <BG2_LOG_FILE> with the path to your "weidu.log" file.

  > Example: --bg2-log-file C:\eet-weidu.log

#### Common Options

These flags work for either of the above commands

* -w, --weidu-binary <WEIDU_BINARY>


  > What it does: This tells the program where to find the WeiDU program.

  > How to use it: Replace <WEIDU_BINARY> with the path to your WeiDU executable.

  > Example: --weidu-binary C:\WeiDU\weidu.exe


* -m, --mod-directories <MOD_DIRECTORIES>


  > What it does: This tells the program where to find the mod files.

  > How to use it: Replace <MOD_DIRECTORIES> with the path(s) to your mod folder(s).

  > Example: --mod-directories C:\BG_Mods


* -l, --language <LANGUAGE>


  > What it does: This sets the language for the game and mods.

  > How to use it: Replace <LANGUAGE> with your preferred language code.

  > Default: en_US (English)

  > Example: --language fr_FR (for French)


* -d, --depth <DEPTH>


  > What it does: This sets how deep the program should look in folders for mod files.

  > How to use it: Replace <DEPTH> with a number.

  > Default: 5

  > Example: --depth 3


* -s, --skip-installed


  > What it does: This makes the program check what's already installed and skip those mods.

  > How to use it: Just add this option to your command if you want to use it.

  > Default: This is on by default.

  > Example: --skip-installed=false


* -a, --abort-on-warnings


  > What it does: This makes the program stop if it encounters any warnings.

  > How to use it: Just add this option to your command if you want to use it.

  > Default: This is on by default.

  > Example: --abort-on-warnings=false


* -t, --timeout <TIMEOUT>


  > What it does: This sets how long the program will wait for each mod to install before giving up.

  > How to use it: Replace <TIMEOUT> with a number of seconds.

  > Default: 3600 (1 hour)

  > Example: --timeout 7200 (2 hours)

* -i, --tick <TICK>


  > What it does: This sets how long the program will wait between polling weidu.

  > How to use it: Replace <TIMEOUT> with a number of milliseconds.

  > Default: 500 (1/2 a second)

  > Example: --tick 1000

* -u, --weidu-log-mode <WEIDU_LOG_MODE>


  > What it does: This sets how WeiDU should log its actions.

  > How to use it: Replace <WEIDU_LOG_MODE> with a WeiDU log mode.

  > Default: --autolog

  > Example: --weidu-log-mode --log

* -o, --overwrite

  > What it does: This tells the mod_installer to overwrite mod directories in the game folder.

  > How to use it: Pass the overwrite flag to the mod_installer

  > Default: false

  > Example: -o


* -x, --strict-matching


  > What it does: This makes the program more strict about matching mod versions and components.

  > How to use it: Just add this option to your command if you want to use it.

  > Default: This is off by default.

  > Example: --strict-matching

* -c, --check_last_installed


  > What it does: This has the mod_installer compare what it just installed vs the last line in the weidu.log file.

  > How to use it: Just add this option to your command if you want to use it.

  > Default: This is off by default.

  > Example: -c --check_last_installed

* -V, --version


  > What it does: This shows what version of the program you're using.

  > How to use it: Just add this option to your command if you want to check the version.

  > Example: mod_installer --version

### Configuring the Parser

See the `example_config.toml` for defaults parser uses. Here we provide a brief breakdown of what each configuration does:

Name|Category|Description|Example
----|----|:----|----
| in_progress_words | A list of words | Checks if weidu is currently running | ["installing", "creating", ...]
| useful_status_words | A list of words | Provides feedback on the weidu process | ["copied", "copying", ...]
| choice_words | A list of words | Words which check if weidu wants user input | ["choice", "choose", ...]
| choice_phrase | A list of phrases | Phrases which check if weidu wants user input | ["do you want", "would you like", ...]
| completed_with_warnings | A list of phrases | Phrases which wiedu uses if it finishes with warning | ["installed with warnings", ...]
| failed_with_error | A list of phrases | Phrases which wiedu uses if it finishes with an error | ["not installed due to errors", ...]
| finished | A list of phrases | Phrases which wiedu uses if it finishes successfully | ["successfully installed","process ended", ...]

Note: **All words/phrases are compared in lowercase ascii.**

If you wish to changes the above; or you are using a different game language (apologies for not translating all of this); have found a exemption; or just want to change the way the parser works you'll need to create your own mod_installer.toml.

We use the rust crate [`confy`](https://crates.io/crates/confy) to load configuration. Confy uses the rust crate [`directories`](https://crates.io/crates/directories) to find the the expected path for your operating system. The `directories` crate uses:

- the XDG base directory and the XDG user directory specifications on Linux
- the Known Folder API on Windows
- the Standard Directories guidelines on macOS

In order to save you some time reading all the above we will put the expected locations below:

- Windows: `{FOLDERID_RoamingAppData}\mod_installer\config`
- Macos: `$HOME/Library/Application Support/mod_installer/config.toml`
- Linux: `$XDG_CONFIG_HOME/mod_installer/config.toml` or `$HOME/.config/mod_installer/config.toml`

### Logging

You can show more install information by setting the `RUST_LOG` environment variable. Here are some of the levels you can use:

For some additional information:

```sh
RUST_LOG=INFO mod_installer [OPTIONS]
```

For detailed information about each step:
```sh
RUST_LOG=DEBUG mod_installer [OPTIONS]
```

For absolutely everything, including WeiDU logs:
```sh
RUST_LOG=TRACE mod_installer [OPTIONS]
```

For more information on logging visit the rust crate [`log`](https://crates.io/crates/log).

### Building

We offer two ways of building the mod_installer. Nix or cargo directly.

#### Cargo

```sh
cargo build --release
```

#### Nix

```sh
nix build -f default.nix
```
