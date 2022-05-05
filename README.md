# set_env

This crate allows you to permanently set environment variables

## Based on [env_perm](https://github.com/freesig/env_perm)

## Examples
```rust
// Check if DUMMY is set, if not set it to 1
// export DUMMY=1
set_env::check_or_set("DUMMY", 1).expect("Failed to find or set DUMMY");
// Append $HOME/some/cool/bin to $PATH
// export PATH= "$HOME/some/cool/bin:$PATH"Cancel changes
set_env::append("PATH", "$HOME/some/cool/bin").expect("Couldn't find PATH");
// Sets a variable without checking if it exists.
// Note you need to use a raw string literal to include ""
// export DUMMY="/something"
set_env::set("DUMMY", r#""/something""#).expect("Failed to set DUMMY");
```

## Usage
This crate simply appends to your `.bash_profile` or `.bash_login` or `.profile`
in that order.
It will create a `.bash_profile` file if none of the above are
found in your home directory.
`ie. /Users/me/.bash_profile`.

On windows, this crate will modify the `HKEY_CURRENT_USER\Environment` registry items
