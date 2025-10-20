use std::process::Command;

/// Create a command with sudo if not running as root
pub fn command(program: &str) -> Command {
    if is_root() {
        Command::new(program)
    } else {
        let mut cmd = Command::new("sudo");
        cmd.arg(program);
        cmd
    }
}

/// Check if running as root
fn is_root() -> bool {
    if let Ok(output) = std::process::Command::new("id").arg("-u").output()
        && let Ok(uid_str) = String::from_utf8(output.stdout)
        && uid_str.trim() == "0"
    {
        return true;
    }

    std::env::var("USER").is_ok_and(|user| user == "root")
        || std::env::var("EUID").is_ok_and(|euid| euid == "0")
}
