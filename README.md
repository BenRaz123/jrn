# `jrn`

## About

`jrn` is a secure command line journal for storing embarrassing secrets, evil schemes, and diabolical plans.

## Security

`jrn` stores all data needed to run in the `jrn.json` file. `jrn` uses
    - `bcrpyt` for password hashing
    - `pbkdf2` for key-derivation
    - and `AES-256` for encrypting every single entry

## Usage

### Interactive Usage

To get a fully interactive command line experience, simply type

```
$ jrn
```
and you will be launched into an interactive prompt. If your storage file has not been written to by `jrn` before, it will prompt you for a new password. Otherwise, it will ask you for your password. Once the authentication is over, There will be a slight pause as `jrn` works on unencrypting the data. The pause is mainly due to the key generation step taking 100,000 rounds. 

Then, `jrn` will prompt you for an action. Any editing action (edit or edit today) will pull up your `$EDITOR`. After any action, by default, `jrn` will prompt you again, looping the UI forever (until you select `Quit`). If this is not your desired behaviour, you can set the `--dont-loop` or `-D` flag. This will force the UI never to loop.

### Non-Interactive (script able) usage

In order to get the full command line options, type `jrn  --help`. To start, the menu selection at the beginning of the program can be automated or skipped if subcommands are used. There is a subcommand equivalent for every single action except of course for Quit. Additionally, when the UI is launched with a subcommand specified, the default behaviour is to not loop the UI. If you would like for the UI to loop regardless, you can specify the `--do-loop` or `-L` flag. However, specifying a subcommand still entails interactivity. This can be avoided by fully fleshing out the subcommand (specifying all options). Even then, a password prompt will be shown. If this must be avoided, it is possible but not recomended to pass in the password as plaintext as either a file (`--password-file` or `-P`) or a string (`--password` or `-p`).

## Configuration

### Command Line Options

* `--config-file` | `-c` := use toml file as configuration. Defalt is `$JRN_CONFIG_FILE` or `$XDG_CONFIG_DIR/jrn/config.toml` or `~/.config/jrn/config.toml` 
* `--password` | `-p` := use given password instead of interactive authentication
* `--password-file` | `-P` := read from given password file instead of interactive authentication
* `--dont-loop` | `-D` := force ui not to loop, even when there are no subcommands specified
* `--do-loop` | `-L` := force ui to loop even when ther are subcommands specified
* `--file-type` | `-F` := use different file type for editing journal entry (example `".org"`)
    > [!NOTE]
    > This option is only for editor purposes. It does not change how the data is stored
* `--file-path` | `-f` := use different file for reading to and writing to (default is "./jrn.json" which may not be wanted)

### Toml configuration file

> [!NOTE]
> When there is a conflict between the configuration file and the command line arguments, the command line arguments take precedent

In the toml configuration file, one can specify all of the options above except for `--config-file`.

The format for string options is

```toml
hello_world="string"
```

And the format for boolean options is

```toml
boolean_option=true
```

> [!NOTE]
> Setting a boolean option to false is the same as not setting it at all

#### How to get default toml file

run 

```
$ jrn --dump-default-config > $JRN_CONFIG_FILE
```

