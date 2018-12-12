use std::io::{self, Write}; 
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use std::env;
use std::fmt;
use dirs;

pub fn check_or_set<T, U>(var: T, value: U) -> io::Result<()>
where T: fmt::Display + AsRef<std::ffi::OsStr>,
      U: fmt::Display,
{
    env::var(&var)
        .map(|_|())
        .or_else(|_| set(var, value))
}

pub fn append<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}=\"{}:${}\"", var, value, var)
}

pub fn set<T: fmt::Display, U: fmt::Display>(var: T, value: U) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}={}", var, value)
}

fn get_profile() -> io::Result<File> {
    dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No home directory"))
        .and_then(find_profile)
}

#[cfg(target_family = "unix")]
fn find_profile(mut home: PathBuf) -> io::Result<File> {
    let mut oo = OpenOptions::new();
    oo.append(true);
    let mut profile = home.clone();
    profile.push(".profile");
    oo.clone()
        .open(profile)
        .or_else(|_| {
            home.push(".bash_profile");
            oo.open(home)
        })
}
