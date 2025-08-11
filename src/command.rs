// src/command.rs

use crate::error::AppResult;
// no serde types used here anymore
use anyhow;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum CommandUpdate {
    NewLine(String),
    Finished(Option<i32>),
}

#[derive(Clone, Debug)]
pub struct CommandLog {
    pub command: String,
    pub output: String,
    pub is_running: bool,
    pub cwd: PathBuf,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<u128>,
}

impl CommandLog {
    pub fn new(command: String, output: String, is_running: bool, cwd: PathBuf) -> Self {
        Self {
            command,
            output,
            is_running,
            cwd,
            exit_code: None,
            duration_ms: None,
        }
    }
}

#[derive(Default)]
pub struct CommandManager {
    kill_sender: Option<oneshot::Sender<()>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn_command(
        &mut self,
        cmd: &str,
        args: &[String],
        cwd: &Path,
        tx: UnboundedSender<CommandUpdate>,
    ) -> AppResult<()> {
        let mut child = TokioCommand::new(cmd)
            .args(args)
            .current_dir(cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout for command: {cmd}"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("Failed to capture stderr for command: {cmd}"))?;

        let (kill_tx, mut kill_rx) = oneshot::channel();
        self.kill_sender = Some(kill_tx);

        let tx_out = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_out.send(CommandUpdate::NewLine(line)).is_err() {
                    break;
                }
            }
        });

        let tx_err = tx.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if tx_err
                    .send(CommandUpdate::NewLine(format!("[stderr] {line}")))
                    .is_err()
                {
                    break;
                }
            }
        });

        let tx_finish = tx;
        #[allow(unused_mut)]
        tokio::spawn(async move {
            tokio::select! {
                status = child.wait() => {
                    // Command finished on its own
                    let code = status.ok().and_then(|s| s.code());
                    let _ = tx_finish.send(CommandUpdate::Finished(code));
                    return;
                }
                _ = &mut kill_rx => {
                    // Kill signal received
                    let _ = child.kill().await;
                    let _ = tx_finish.send(CommandUpdate::Finished(None));
                    return;
                }
            }
        });

        Ok(())
    }

    pub fn kill_running_command(&mut self) -> AppResult<()> {
        if let Some(sender) = self.kill_sender.take() {
            // Send the kill signal. We don't care if it fails,
            // as that means the process already finished.
            let _ = sender.send(());
        }
        Ok(())
    }
}

// Removed duplicate CompletionState. The canonical implementation lives in crate::completion.
