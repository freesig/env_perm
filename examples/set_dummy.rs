use env_perm;

fn main() {
    env_perm::check_or_set("DUMMY", 1).expect("Failed to find or set DUMMY");
    env_perm::append("PATH", "$HOME/some/cool/bin").expect("Couldn't find PATH");
    env_perm::set("DUMMY", r#""/something""#).expect("Failed to set DUMMY");
}
