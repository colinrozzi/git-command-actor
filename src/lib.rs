mod bindings;
mod git;
mod types;

use crate::bindings::exports::theater::simple::actor::Guest;
use crate::bindings::exports::theater::simple::process_handlers::Guest as ProcessHandlers;
use crate::bindings::theater::simple::runtime::{log, shutdown};
use crate::types::*;

struct Component;

impl Guest for Component {
    fn init(state: Option<Vec<u8>>, params: (String,)) -> Result<(Option<Vec<u8>>,), String> {
        let (actor_id,) = params;
        log(&format!(
            "Initializing git-command-actor with ID: {}",
            actor_id
        ));

        // Parse the initial configuration
        let config: GitCommandConfig = match state {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| format!("Failed to parse init state: {}", e))?,
            None => return Err("No init state provided".to_string()),
        };

        log(&format!(
            "Git command actor config - repo: {}, command: {:?}",
            config.repository_path, config.git_args
        ));

        // Create actor state
        let mut actor_state = GitActorState::new(config);

        // Start the git command
        match git::start_git_command(&mut actor_state) {
            Ok(_) => log("Git command started successfully"),
            Err(e) => {
                log(&format!("Failed to start git command: {}", e));
                // Complete immediately with error
                actor_state.completed = true;
            }
        }

        // If we completed immediately (due to validation error), shutdown
        if actor_state.completed {
            let result = actor_state.to_result();
            log(&format!("Immediate completion with result: {:?}", result));

            let shutdown_data = serde_json::to_vec(&result)
                .map_err(|e| format!("Failed to serialize result: {}", e))?;

            let _ = shutdown(Some(&shutdown_data));
        }

        // Serialize and return the state
        let state_bytes = serde_json::to_vec(&actor_state)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;

        log("Git command actor initialized successfully");
        Ok((Some(state_bytes),))
    }
}

impl ProcessHandlers for Component {
    fn handle_exit(
        state: Option<Vec<u8>>,
        params: (u64, i32),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (pid, exit_code) = params;

        // Parse the current state
        let mut actor_state: GitActorState = match state {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| format!("Failed to parse state: {}", e))?,
            None => return Err("No state provided".to_string()),
        };

        // Handle the process exit
        git::handle_process_exit(&mut actor_state, pid, exit_code);

        // If we're completed, shutdown with results
        if actor_state.completed {
            let result = actor_state.to_result();
            log(&format!(
                "Git command completed. Success: {}, Exit code: {:?}",
                result.success, result.exit_code
            ));

            let shutdown_data = serde_json::to_vec(&result)
                .map_err(|e| format!("Failed to serialize result: {}", e))?;

            let _ = shutdown(Some(&shutdown_data));
        }

        // Return updated state
        let updated_state = serde_json::to_vec(&actor_state)
            .map_err(|e| format!("Failed to serialize updated state: {}", e))?;
        Ok((Some(updated_state),))
    }

    fn handle_stdout(
        state: Option<Vec<u8>>,
        params: (u64, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (pid, data) = params;

        // Parse the current state
        let mut actor_state: GitActorState = match state {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| format!("Failed to parse state: {}", e))?,
            None => return Err("No state provided".to_string()),
        };

        // Check if this is from our active process
        if let Some(active_pid) = actor_state.active_process {
            if active_pid == pid {
                // Convert data to string and process
                let stdout_data = String::from_utf8_lossy(&data);
                git::process_stdout(&mut actor_state, &stdout_data);
            }
        }

        // Check for timeout
        if git::is_timeout_exceeded(&actor_state) && !actor_state.completed {
            log(&format!(
                "Git command timed out after {} seconds",
                actor_state.timeout_seconds
            ));
            actor_state.completed = true;
            actor_state.validation_error = Some(format!(
                "Command timed out after {} seconds",
                actor_state.timeout_seconds
            ));

            let result = actor_state.to_result();
            let shutdown_data = serde_json::to_vec(&result)
                .map_err(|e| format!("Failed to serialize result: {}", e))?;

            let _ = shutdown(Some(&shutdown_data));
        }

        // Return updated state
        let updated_state = serde_json::to_vec(&actor_state)
            .map_err(|e| format!("Failed to serialize updated state: {}", e))?;
        Ok((Some(updated_state),))
    }

    fn handle_stderr(
        state: Option<Vec<u8>>,
        params: (u64, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (pid, data) = params;

        // Parse the current state
        let mut actor_state: GitActorState = match state {
            Some(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| format!("Failed to parse state: {}", e))?,
            None => return Err("No state provided".to_string()),
        };

        // Check if this is from our active process
        if let Some(active_pid) = actor_state.active_process {
            if active_pid == pid {
                // Convert data to string and process
                let stderr_data = String::from_utf8_lossy(&data);
                git::process_stderr(&mut actor_state, &stderr_data);
            }
        }

        // Return updated state
        let updated_state = serde_json::to_vec(&actor_state)
            .map_err(|e| format!("Failed to serialize updated state: {}", e))?;
        Ok((Some(updated_state),))
    }
}

bindings::export!(Component with_types_in bindings);
