use picolayer::utils::sudo;

#[test]
fn test_sudo_command_integration() {
    let cmd = sudo::command("test-command");
    let program = cmd.get_program();

    assert!(program == "test-command" || program == "sudo");
}
