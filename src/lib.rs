//! This crate allows you to permanently set environment variables
//!
//! # Examples
//! ```rust
//! // Check if DUMMY is set, if not set it to 1
//! // export DUMMY=1
//! env_perm::check_or_set("DUMMY", 1).expect("Failed to find or set DUMMY");
//! // Append $HOME/some/cool/bin to $PATH
//! // export PATH= "$HOME/some/cool/bin:$PATH"
//! env_perm::append("PATH", "$HOME/some/cool/bin").expect("Couldn't find PATH");
//! // Sets a variable without checking if it exists.
//! // Note you need to use a raw string literal to include ""
//! // export DUMMY="/something"
//! env_perm::set("DUMMY", r#""/something""#).expect("Failed to set DUMMY");
//! ```

#[cfg(target_family = "unix")]
use dirs;
#[cfg(target_family = "unix")]
use std::fs::{File, OpenOptions};
#[cfg(target_family = "unix")]
use std::io::Write;
#[cfg(target_family = "unix")]
use std::path::PathBuf;

#[cfg(target_family = "windows")]
use winreg::RegKey;

#[cfg(target_family = "windows")]
use winreg::enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS};

#[cfg(target_family = "unix")]
use std::env;
#[cfg(target_family = "unix")]
use std::env::VarError;
use std::fmt;
use std::io;


/// Checks if a environment variable is set.
/// If it is then nothing will happen.
/// If it's not then it will be added
/// to your profile.
#[cfg(target_family = "unix")]
pub fn check_or_set<T, U>(var: T, value: U) -> io::Result<()>
where
    T: fmt::Display + AsRef<std::ffi::OsStr>,
    U: fmt::Display,
{
    env::var(&var).map(|_| ()).or_else(|_| set(var, value))
}

#[cfg(target_os = "windows")]
pub fn check_or_set<T, U>(var: T, value: U) -> io::Result<()>
where
    T: fmt::Display + AsRef<std::ffi::OsStr>,
    U: fmt::Display,
{
    let from = get(&var).unwrap();
    if from.len() > 0 {
        Ok(())
    } else {
        set(var, value)?;
        Ok(())
    }
}

#[cfg(target_family = "unix")]
pub fn get<'a, T: fmt::Display>(var: T) -> io::Result<String> {
    env::var(var.to_string()).map_err(|err| match err {
        VarError::NotPresent => {
            io::Error::new(io::ErrorKind::NotFound, "Variable not present.")
        }
        VarError::NotUnicode(_) => {
            io::Error::new(io::ErrorKind::Unsupported, "Encoding not supported.")
        }
    })
}

#[cfg(target_os = "windows")]
pub fn get<'a, T: fmt::Display>(var: T) -> io::Result<String> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Environment")?;
    Ok(key.get_value::<String, String>(var.to_string())?.to_string())
}

/// Appends a value to an environment variable
/// Useful for appending a value to PATH
#[cfg(target_family = "unix")]
pub fn append<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}=\"{}:${}\"", var, value, var)?;
    profile.flush()
}
/// Appends a value to an environment variable
/// Useful for appending a value to PATH
#[cfg(target_os = "windows")]
pub fn append<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
    let env_original: String = key.get_value(var.to_string())?;
    let mut env_vec = env_original.split(";").collect::<Vec<&str>>();
    if !env_vec.contains(&value.to_string().as_str()) {
        let env = value.to_string();
        env_vec.push(env.as_str());
        key.set_value(var.to_string(), &env_vec.as_slice().join(";").as_str())?;
    }
    Ok(())
}

/// Prepends a value to an environment variable
/// Useful for prepending a value to PATH
#[cfg(target_family = "unix")]
pub fn prepend<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}=\"${}:{}\"", var, value, var)?;
    profile.flush()
}

/// Prepends a value to an environment variable
/// Useful for prepending a value to PATH
#[cfg(target_os = "windows")]
pub fn prepend<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
    let env_original: String = key.get_value(var.to_string())?;
    let mut env_vec = env_original.split(";").collect::<Vec<&str>>();
    if !env_vec.contains(&value.to_string().as_str()) {
        let env = value.to_string();
        env_vec.insert(0, env.as_str());
        key.set_value(var.to_string(), &env_vec.as_slice().join(";").as_str())?;
    }
    Ok(())
}

/// Sets an environment variable without checking
/// if it exists.
/// If it does you will end up with two
/// assignments in your profile.
/// It's recommended to use `check_or_set`
/// unless you are certain it doesn't exist.
#[cfg(target_family = "unix")]
pub fn set<T: fmt::Display, U: fmt::Display>(var: T, value: U) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}={}", var, value)?;
    profile.flush()
}
/// Sets an environment variable without checking
/// if it exists.
/// If it does you will override the value.
#[cfg(target_os = "windows")]
pub fn set<T: fmt::Display, U: fmt::Display>(var: T, value: U) -> io::Result<()> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags("Environment", KEY_ALL_ACCESS)?;
    key.set_value(var.to_string(), &value.to_string())
}

#[cfg(target_family = "unix")]
fn get_profile() -> io::Result<File> {
    dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No home directory"))
        .and_then(find_profile)
}

#[cfg(target_family = "unix")]
fn find_profile(mut profile: PathBuf) -> io::Result<File> {
    profile.push(".bash_profile");
    let mut oo = OpenOptions::new();
    oo.append(true).create(false);
    oo.open(profile.clone())
        .or_else(|_| {
            profile.pop();
            profile.push(".bash_login");
            oo.open(profile.clone())
        })
        .or_else(|_| {
            profile.pop();
            profile.push(".profile");
            oo.open(profile.clone())
        })
        .or_else(|_| {
            profile.pop();
            profile.push(".bash_profile");
            oo.create(true);
            oo.open(profile.clone())
        })
}
