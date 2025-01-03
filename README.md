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

## Installation

`mod_installer` can be installed via crates.io:

```sh
cargo install mod_installer
```

or you can grab it from the latest releases page on github, [here](https://github.com/dark0dave/mod_installer/releases/latest).

## Usage

To use the Infinity Engine Mod Installer, you need to run it from the command line. Here's the basic structure of the command:

```sh
mod_installer(.exe) [OPTIONS]
  --log-file <LOG_FILE>
  --game-directory <GAME_DIRECTORY>
  --weidu-binary <WEIDU_BINARY>
  --mod-directories <MOD_DIRECTORIES>
```
Let's break down what each part means:

* mod_installer(.exe): This is the name of the program you're running.
[OPTIONS]: These are additional settings you can use to customize how the program works (we'll explain these in detail below).

* --log-file <LOG_FILE>: This is where you tell the program where to find the "weidu.log" file.

* --game-directory <GAME_DIRECTORY>: This is where you tell the program where your game is installed.

* --weidu-binary <WEIDU_BINARY>: This is where you tell the program where to find the WeiDU program (WeiDU is a tool used to install mods).

* --mod-directories <MOD_DIRECTORIES>: This is where you tell the program where to find the mod files.

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

Remember, the Weidu log is a powerful tool, but it's important to understand what you're installing. Always back up your game files before installing mods, and be aware that some mods may conflict with others.

### Demo
We have a short video that shows how the tool works:

![](docs/mod_installer.gif)

### What options can I use?

**Don't panic** you can use the help command to find all the options listed below:

* -h, --help

  What it does: This shows a help message with information about how to use the program.
  How to use it: Just add this option to your command if you need help.
  Example: mod_installer --help

Here's a detailed explanation of all the options you can use:

* -f, --log-file <LOG_FILE>

  What it does: This tells the program where to find the "weidu.log" file.
  How to use it: Replace <LOG_FILE> with the path to your "weidu.log" file.
  Example: --log-file C:\Games\Baldur's Gate\weidu.log


* -g, --game-directory <GAME_DIRECTORY>

  What it does: This tells the program where your game is installed.
  How to use it: Replace <GAME_DIRECTORY> with the path to your game folder.
  Example: --game-directory C:\Games\Baldur's Gate


* -w, --weidu-binary <WEIDU_BINARY>

  What it does: This tells the program where to find the WeiDU program.
  How to use it: Replace <WEIDU_BINARY> with the path to your WeiDU executable.
  Example: --weidu-binary C:\WeiDU\weidu.exe


* -m, --mod-directories <MOD_DIRECTORIES>

  What it does: This tells the program where to find the mod files.
  How to use it: Replace <MOD_DIRECTORIES> with the path(s) to your mod folder(s).
  Example: --mod-directories C:\BG_Mods


* -l, --language <LANGUAGE>

  What it does: This sets the language for the game and mods.
  How to use it: Replace <LANGUAGE> with your preferred language code.
  Default: en_US (English)
  Example: --language fr_FR (for French)


* -d, --depth <DEPTH>

  What it does: This sets how deep the program should look in folders for mod files.
  How to use it: Replace <DEPTH> with a number.
  Default: 5
  Example: --depth 3


* -s, --skip-installed

  What it does: This makes the program check what's already installed and skip those mods.
  How to use it: Just add this option to your command if you want to use it.
  Default: This is on by default.
  Example: --skip-installed=false


* -a, --abort-on-warnings

  What it does: This makes the program stop if it encounters any warnings.
  How to use it: Just add this option to your command if you want to use it.
  Default: This is on by default.
  Example: --abort-on-warnings=false


* -t, --timeout <TIMEOUT>

  What it does: This sets how long the program will wait for each mod to install before giving up.
  How to use it: Replace <TIMEOUT> with a number of seconds.
  Default: 3600 (1 hour)
  Example: --timeout 7200 (2 hours)


* -u, --weidu-log-mode <WEIDU_LOG_MODE>

  What it does: This sets how WeiDU should log its actions.
  How to use it: Replace <WEIDU_LOG_MODE> with a WeiDU log mode.
  Default: --autolog
  Example: --weidu-log-mode --log


* -x, --strict-matching

  What it does: This makes the program more strict about matching mod versions and components.
  How to use it: Just add this option to your command if you want to use it.
  Default: This is off by default.
  Example: --strict-matching

* -V, --version

  What it does: This shows what version of the program you're using.
  How to use it: Just add this option to your command if you want to check the version.
  Example: mod_installer --version

### Configuring the Parser

See the `example_config.toml` for defaults parser uses. Here we provide a brief breakdown of what each configuration does:

Name|Category|Description|Example
----|----|:----|----
| in_progress_words | A list of words | Checks if weidu is currently running | ["installing", "creating",]
| useful_status_words | A list of words | Provides feedback on the weidu process | ["copied", "copying",]
| choice_words | A list of words | Words which check if weidu wants user input | ["choice", "choose",]
| choice_phrase | A list of phrases | Phrases which check if weidu wants user input | ["do you want", "would you like",]
| completed_with_warnings | A single phrase | Standard phrase wiedu uses if it finishes with warning | "installed with warnings"
| failed_with_error | A single phrase | Standard phrase wiedu uses if it finishes with an error | "not installed due to errors"
| finished | A single phrase | Standard phrase wiedu uses if it finishes successfully | "successfully installed"
| eet_finished | A single phrase | A special exemption for EET for EET Core install | "process ended"

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
