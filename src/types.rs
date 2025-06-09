use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Initial configuration for the git command actor
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitCommandConfig {
    /// Path to the git repository
    pub repository_path: String,
    /// Git command arguments (e.g., ["status", "--porcelain"])
    pub git_args: Vec<String>,
    /// Optional timeout in seconds (default: 30)
    pub timeout_seconds: Option<u32>,
    /// Optional working directory (defaults to repository_path)
    pub working_directory: Option<String>,
}

/// Internal actor state
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitActorState {
    pub repository_path: String,
    pub git_args: Vec<String>,
    pub timeout_seconds: u32,
    pub working_directory: Option<String>,
    pub active_process: Option<u64>,
    pub stdout_buffer: String,
    pub stderr_buffer: String,
    pub exit_code: Option<i32>,
    pub completed: bool,
    #[serde(skip)]
    pub start_time: Option<SystemTime>,
    pub validation_error: Option<String>,
}

/// Result structure returned on shutdown
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GitCommandResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub command: Vec<String>,
    pub execution_time_ms: Option<u64>,
    pub error: Option<String>,
    pub repository_path: String,
}

impl GitActorState {
    pub fn new(config: GitCommandConfig) -> Self {
        Self {
            repository_path: config.repository_path,
            git_args: config.git_args,
            timeout_seconds: config.timeout_seconds.unwrap_or(30),
            working_directory: config.working_directory,
            active_process: None,
            stdout_buffer: String::new(),
            stderr_buffer: String::new(),
            exit_code: None,
            completed: false,
            start_time: None,
            validation_error: None,
        }
    }

    pub fn get_full_command(&self) -> Vec<String> {
        let mut cmd = vec!["git".to_string(), "-C".to_string(), self.repository_path.clone()];
        cmd.extend(self.git_args.clone());
        cmd
    }

    pub fn to_result(&self) -> GitCommandResult {
        // Execution time disabled in WASM environment
        let execution_time_ms = None;

        GitCommandResult {
            success: self.exit_code == Some(0) && self.validation_error.is_none(),
            exit_code: self.exit_code,
            stdout: self.stdout_buffer.clone(),
            stderr: self.stderr_buffer.clone(),
            command: self.get_full_command(),
            execution_time_ms,
            error: self.validation_error.clone(),
            repository_path: self.repository_path.clone(),
        }
    }
}
