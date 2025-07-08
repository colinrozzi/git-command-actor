use crate::bindings::theater::simple::process::{os_spawn, OutputMode, ProcessConfig};
use crate::bindings::theater::simple::runtime::log;
use crate::types::*;
use std::path::Path;
use std::time::SystemTime;

pub fn validate_repository(path: &str) -> Result<(), String> {
    log(&format!(
        "Skipping filesystem validation for path: {} (WASM sandbox limitation)",
        path
    ));

    // Skip filesystem validation in WASM environment due to sandboxing
    // Let the git process itself handle path validation
    log("Repository validation skipped - delegating to git process");
    Ok(())
}

pub fn start_git_command(state: &mut GitActorState) -> Result<(), String> {
    // Validate the repository first
    if let Err(e) = validate_repository(&state.repository_path) {
        state.validation_error = Some(e.clone());
        state.completed = true;
        return Err(e);
    }

    log(&format!(
        "Starting git command: {:?}",
        state.get_full_command()
    ));

    // Skip start time recording in WASM environment (SystemTime::now() not available)
    log("Skipping start time recording (WASM limitation)");
    state.start_time = None;

    // Build the process configuration
    let mut args = vec!["-C".to_string(), state.repository_path.clone()];
    args.extend(state.git_args.clone());

    let config = ProcessConfig {
        program: "git".to_string(),
        args,
        env: vec![],
        cwd: state.working_directory.clone(),
        buffer_size: 1024 * 1024, // 1MB buffer
        chunk_size: None,
        stdout_mode: OutputMode::Raw,
        stderr_mode: OutputMode::Raw,
        execution_timeout: None, // No timeout in WASM
    };

    // Spawn the process
    match os_spawn(&config) {
        Ok(pid) => {
            log(&format!("Git process started with PID: {}", pid));
            state.active_process = Some(pid);
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to spawn git process: {}", e);
            log(&error_msg);
            state.validation_error = Some(error_msg.clone());
            state.completed = true;
            Err(error_msg)
        }
    }
}

pub fn process_stdout(state: &mut GitActorState, data: &str) {
    log(&format!("Git stdout: {}", data));
    state.stdout_buffer.push_str(data);
}

pub fn process_stderr(state: &mut GitActorState, data: &str) {
    log(&format!("Git stderr: {}", data));
    state.stderr_buffer.push_str(data);
}

pub fn handle_process_exit(state: &mut GitActorState, pid: u64, exit_code: i32) {
    if let Some(active_pid) = state.active_process {
        if active_pid == pid {
            log(&format!(
                "Git process {} exited with code: {}",
                pid, exit_code
            ));

            state.exit_code = Some(exit_code);
            state.active_process = None;
            state.completed = true;

            // Log the results
            if exit_code == 0 {
                log("Git command completed successfully");
            } else {
                log(&format!("Git command failed with exit code: {}", exit_code));
            }

            if !state.stdout_buffer.is_empty() {
                log(&format!(
                    "Final stdout length: {} bytes",
                    state.stdout_buffer.len()
                ));
            }

            if !state.stderr_buffer.is_empty() {
                log(&format!(
                    "Final stderr length: {} bytes",
                    state.stderr_buffer.len()
                ));
            }
        }
    }
}

pub fn is_timeout_exceeded(state: &GitActorState) -> bool {
    // Timeout checking disabled in WASM environment due to SystemTime limitations
    false
}
