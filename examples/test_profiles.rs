use std::fs::OpenOptions;
use std::path::Path;

fn main() {
    let mut profile = Path::new("/Users/tomgowan").to_path_buf();
    profile.push(".bash_profile");
    let mut oo = OpenOptions::new();
    oo.append(true)
        .create(false);
    let r = oo.open(profile.clone())
        .or_else(|_|{
            profile.pop();
            profile.push(".bash_login");
            oo.open(profile.clone())
        })
        .or_else(|_|{
            profile.pop();
            profile.push(".profile");
            oo.open(profile.clone())
        })
        .or_else(|_|{
            profile.pop();
            profile.push(".bash_profile");
            oo.create(true);
            oo.open(profile.clone())
        });
    dbg!(&r);
}
