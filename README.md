# Nomgen - A Configurable File Generation and Check Tool

Nomgen stands for **No**t **M**odified **Gen**eration.

It is a tool that help you generate files based on commands and protect generated files from manual modifications in pre-commit git hook.

## Features

- Define file generators using a configuration file in TOML format.
- Automatically generate and modify files based on configured commands.
- Run check to ensure that files that match configured patterns are not modified manually.
- Integrate with Git pre-commit hooks for automated check before committing unless commit is made by nomgen itself.

## Installation

To install Nomgen, make sure you have Rust and Cargo installed, and then run:

```sh
cargo install nomgen
```

## Configuration
Nomgen uses a configuration file (nomgen.toml) to define file generators and check patterns. 
Here's an example configuration:

```toml
[[generators]]
command = "protoc --rust_out=experimental-codegen=enabled,kernel=upb:. foo.proto"
pattern = "*.pb.rs"

[[generators]]
command = "cargo run -- generate-hook"
pattern = "pre-commit-hook.*"
```

## Usage
Nomgen provides several commands to manage file generation and checks:

`nomgen generate [OPTIONS]`
Generate or modify files based on the specified configuration.

Command would firstly check for whether manual modifications are already present and only if repository state is valid, would run generators.

Options:

`-c, --config <FILE>`: Path to the configuration file in TOML format (by default: auto locates closest nomgen.toml).
Example:

```sh
nomgen generate -c nomgen.toml
```

`nomgen hook [OPTIONS]`

Generate a Git pre-commit hook to enforce checks before committing.

Options:

`-t, --target <DIR>`: Target directory for the hook script (default: auto locates closest .git/hooks).
Example:

```sh
nomgen hook -t .git/hooks
```

`nomgen check [OPTIONS]`

Checks whether files in working directory contain modifications in protected patterns. 

Command verifies all files that match patterns configured in nomgen.toml - staged or not. 
In case if any modifications found, it would exit with error code 1. 
Normally is supposed to be called from the hook, but can be called manually for troubleshooting as well.

Options:

`-c, --config <FILE>`: Path to the configuration file in TOML format (by default: auto locates closest nomgen.toml).
Example:

```sh
nomgen check -c nomgen.toml
```

## Getting Started

You should start from installing the hook by running `nomgen hook` in your repository.

Then setup `nomgen.toml` file by specifying generators.

Each generator can have command that should be run to generate the content.
In addition to command, you probably would want to setup pattern that would match generated files to protect from manual writes.

It is possible to have a generator with only command - if you do not care to protect from manual writes.
You could also have a generator with only pattern, if you do not want to delegate generation call to nomgen, but still want to use it to locate unwanted changes.

So all following nomgen.toml examples are valid uses:

```toml
[[generators]]
# Configures `nomgen generate` to run protoc 
command = "protoc"
args = ["--rust_out=experimental-codegen=enabled,kernel=upb:.", "foo.proto"]
# Configures `nomgen check` and `nomgen generate` to prevent accidental changes to files ending on `.pb.rs`
pattern = "*.pb.rs"
```

```toml
[[generators]]
# Configures `nomgen generate` to run protoc 
command = "protoc"
args = ["--rust_out=experimental-codegen=enabled,kernel=upb:.", "foo.proto"]
# No pattern config, no protection from manual changes
```

```toml
[[generators]]
# Configures `nomgen check` and `nomgen generate` to prevent accidental changes to files ending on `.pb.rs`
pattern = "*.pb.rs"
# No command to use during `nomgen generate` 
```

```toml
# Generator for protobuf things
[[generators]]
# Configures `nomgen generate` to run protoc 
command = "protoc"
args = ["--rust_out=experimental-codegen=enabled,kernel=upb:.", "foo.proto"]
# Configures `nomgen check` and `nomgen generate` to prevent accidental changes to files ending on `.pb.rs`
pattern = "*.pb.rs"

# Generator for nomgen hook
[[generators]]
command = "nomgen"
args = ["hook", "-d", ".husky"]
pattern = ".husky/pre-commit"

# Some other generators could follow
# [[generators]]
# ...
```

### With husky?

Yeah, right, example above shows that if your setup includes hooks commited to repository - then generating nomgen hook falls under the use case of nomgen being one of its own generators. It is a valid use case, but look out for different users having different versions of nomgen - this can result in hook being regenerated all the time.

Naturally, if you want more than one thing to happen in pre-commit, you probably would want to do something like this:

```toml
# Generator for nomgen hook
[[generators]]
command = "nomgen"
args = ["hook", "-d", ".husky", "-h", "pre-commit-nomgen"]
pattern = ".husky/pre-commit-nomgen"

# Add some more generators
# ...
```

This produces nomgen hook in `pre-commit-nomgen` file, which is not called by git, which means 
that now in pre-commit you would have something along those lines:

```bash
#!/bin/bash

set -e
DIR="$( dirname "${BASH_SOURCE[0]}" )"
sh "${DIR}/pre-commit-nomgen"

# Add other checks that should be run before commiting here
# ... yarn test
# ... yarn prettier

```

So now you have a setup that autogenerates nomgen hook when you call for `nomgen generate`, same as runs any of other generators. The hook protects you from manual changes to autogenerated files, including nomgen hook itself and you could stil execute other checks before commiting, while husky handles configuring git to use your hooks. 

One of examples sets this up with one additional feature - instead of calling `check` (default), it configures hook to call `generate` instead. This forces nomgen to regenerate files before commiting, which is useful if you want to make sure that you do not commit outdated files.

### Generate during hook?

Running `nomgen hook -c generate` can get you the kind of hook that would regenerate files before commiting. This is useful if you want to make sure that you do not commit outdated files. `nomgen` is clever enough to know that generate would not create a commit in this case and would only stage the generated changes so that commit that triggered the hook would include the changes. 

As a variation of that here is config that would make `nomgen generate` to create such generating hook in .husky/pre-commit-nomgen file: 

```toml
[[generators]]
command = "nomgen"
args = ["hook", "-d", ".husky", "-h", "pre-commit-nomgen", "-c", "generate"]
pattern = ".husky/pre-commit-nomgen"
```

So later, you can call that script from your husky-controlled pre-commit as explained above.

