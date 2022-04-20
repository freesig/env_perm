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
use windows::Win32::System::Registry;

#[cfg(target_family = "windows")]
use windows::core::PCWSTR;

use std::env;
use std::fmt;
use std::io;

#[cfg(target_family = "windows")]
struct ToWide {
    inner: Vec<u16>,
    pub wide: PCWSTR,
}

#[cfg(target_family = "windows")]
impl ToWide {
    fn from(s: &str) -> Self {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let v = OsStr::new(s)
            .encode_wide()
            .chain(Some(0).into_iter())
            .collect::<Vec<u16>>();
        ToWide {
            inner: v.to_vec(),
            wide: PCWSTR(v.as_ptr()),
        }
    }
}

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
    env::var(&var).map(|_| ()).or_else(|_| set(var, value))
}

#[cfg(target_os = "windows")]
pub fn get_from_environment<'a, T: fmt::Display>(var: T) -> io::Result<String> {
    use std::ffi::c_void;

    use windows::Win32::Foundation::ERROR_SUCCESS;
    return unsafe {
        // A pointer to a null-terminated string of 16-bit Unicode characters.
        let subkey = ToWide::from("Environment");

        let var = ToWide::from(&var.to_string());
        let mut buffer: Vec<u16> = Vec::with_capacity(1024 * 1024);
        let mut buffer_len: u32 = (buffer.capacity() * 2) as u32;
        let result = Registry::RegGetValueW(
            Registry::HKEY_CURRENT_USER,
            subkey.wide,
            var.wide,
            Registry::RRF_RT_ANY,
            std::ptr::null_mut(),
            buffer.as_mut_ptr() as *mut c_void,
            &mut buffer_len as *mut u32,
        );
        if result != ERROR_SUCCESS {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to get environment variable, Error code: {}",
                    result.0
                ),
            ));
        }
        buffer.set_len(buffer_len as usize);
        let str = String::from_utf16(&buffer).unwrap();
        let str = &str.trim_end_matches('\0');
        Ok(str.to_string())
    };
}

/// Prepends a value to an environment variable
/// Useful for appending a value to PATH
#[cfg(target_family = "unix")]
pub fn append<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}=\"{}:${}\"", var, value, var)?;
    profile.flush()
}
/// Prepends a value to an environment variable
/// Useful for appending a value to PATH
#[cfg(target_os = "windows")]
pub fn append<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    use windows::Win32::Foundation::ERROR_SUCCESS;

    return unsafe {
        // A pointer to a null-terminated string of 16-bit Unicode characters.
        let mut key: Registry::HKEY = std::mem::zeroed();
        let subkey = ToWide::from("Environment");

        let mut result = Registry::RegOpenKeyExW(
            Registry::HKEY_CURRENT_USER,
            subkey.wide,
            0,
            Registry::KEY_ALL_ACCESS,
            &mut key as *mut Registry::HKEY,
        );

        let env_string = get_from_environment(&var)?;

        let var = ToWide::from(&var.to_string());
        if result == ERROR_SUCCESS {
            let mut env_vec = env_string.split(";").collect::<Vec<&str>>();
            let env = value.to_string();
            env_vec.push(env.as_str());
            let env_result = env_vec.as_slice().join(";");
            let wresult = ToWide::from(&env_result);
            result = Registry::RegSetValueExW(
                key,
                var.wide,
                0,
                Registry::REG_SZ,
                wresult.inner.as_ptr() as *const u8,
                (wresult.inner.len() * 2) as u32,
            )
        }

        if result == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to set environment variable, Error code: {}",
                    result.0
                ),
            ))
        }
    };
}

/// Appends a value to an environment variable
/// Useful for appending a value to PATH
#[cfg(target_family = "unix")]
pub fn prepend<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    let mut profile = get_profile()?;
    writeln!(profile, "\nexport {}=\"${}:{}\"", var, value, var)?;
    profile.flush()
}
#[cfg(target_os = "windows")]
pub fn prepend<T: fmt::Display>(var: T, value: T) -> io::Result<()> {
    use windows::Win32::Foundation::ERROR_SUCCESS;

    return unsafe {
        // A pointer to a null-terminated string of 16-bit Unicode characters.
        let mut key: Registry::HKEY = std::mem::zeroed();
        let subkey = ToWide::from("Environment");

        let mut result = Registry::RegOpenKeyExW(
            Registry::HKEY_CURRENT_USER,
            subkey.wide,
            0,
            Registry::KEY_ALL_ACCESS,
            &mut key as *mut Registry::HKEY,
        );

        let env_string = get_from_environment(&var)?;

        let var = ToWide::from(&var.to_string());
        if result == ERROR_SUCCESS {
            let mut env_vec = env_string.split(";").collect::<Vec<&str>>();
            let env = value.to_string();
            env_vec.insert(0, env.as_str());
            let env_result = env_vec.as_slice().join(";");
            let wresult = ToWide::from(&env_result);
            result = Registry::RegSetValueExW(
                key,
                var.wide,
                0,
                Registry::REG_SZ,
                wresult.inner.as_ptr() as *const u8,
                (wresult.inner.len() * 2) as u32,
            )
        }

        if result == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to set environment variable, Error code: {}",
                    result.0
                ),
            ))
        }
    };
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
    use windows::Win32::Foundation::ERROR_SUCCESS;

    return unsafe {
        // A pointer to a null-terminated string of 16-bit Unicode characters.
        let mut key: Registry::HKEY = std::mem::zeroed();
        let subkey = ToWide::from("Environment");

        let mut result = Registry::RegOpenKeyExW(
            Registry::HKEY_CURRENT_USER,
            subkey.wide,
            0,
            Registry::KEY_ALL_ACCESS,
            &mut key as *mut Registry::HKEY,
        );

        let var = ToWide::from(&var.to_string());
        let wdata = ToWide::from(&value.to_string());
        if result == ERROR_SUCCESS {
            result = Registry::RegSetValueExW(
                key,
                var.wide,
                0,
                Registry::REG_SZ,
                wdata.inner.as_ptr() as *const u8,
                (wdata.inner.len() * 2) as u32,
            )
        }

        if result == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Failed to set environment variable, Error code: {}",
                    result.0
                ),
            ))
        }
    };
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
