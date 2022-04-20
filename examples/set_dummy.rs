use set_env;

fn main() {
    // Check if DUMMY is set, if not set it to 1
    // export DUMMY=1
    set_env::check_or_set("DUMMY", 1).expect("Failed to find or set DUMMY");
    // Append $HOME/some/cool/bin to $PATH
    // export PATH= "$HOME/some/cool/bin:$PATH"
    set_env::append("PATH", "$HOME/some/cool/bin").expect("Couldn't find PATH");
    // Sets a variable without checking if it exists.
    // Note you need to use a raw string literal to include ""
    // export DUMMY="/something"
    set_env::set("DUMMY", r#""/something""#).expect("Failed to set DUMMY");
}
