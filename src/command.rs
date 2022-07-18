use std::process::ExitStatus;

pub fn parse_command_result(status: ExitStatus, msg: &str) {
    if !status.success() {
        panic!("{}", msg);
    }
}
