use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use extensions_proto::moosync::types::{
    GetProcessStatusRequest, GetProcessStatusResponse, KillProcessRequest, KillProcessResponse,
    ReadProcessStdoutRequest, ReadProcessStdoutResponse, SpawnProcessRequest, SpawnProcessResponse,
    WriteProcessStdinRequest, WriteProcessStdinResponse,
};
use sha2::{Digest, Sha256};

use crate::errors::ExtensionError;

#[tracing::instrument(level = "debug", skip_all)]
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

pub struct ProcessHandle {
    pub process_id: usize,
    pub pid: u32,
    pub stdin: Mutex<Option<ChildStdin>>,
    pub stdout_buffer: Arc<Mutex<Vec<u8>>>,
    pub is_running: Arc<AtomicBool>,
    pub exit_code: Arc<Mutex<Option<i32>>>,
    pub child: Mutex<Option<Child>>,
}

pub struct ExtensionProcessManager {
    processes: Arc<Mutex<HashMap<usize, ProcessHandle>>>,
    next_process_id: AtomicUsize,
    allowed_paths: Option<HashMap<String, PathBuf>>,
    allowed_binaries: Vec<String>,
}

impl ExtensionProcessManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        allowed_paths: Option<HashMap<String, PathBuf>>,
        allowed_binaries: Vec<String>,
    ) -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            next_process_id: AtomicUsize::new(1),
            allowed_paths,
            allowed_binaries,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn resolve_path(&self, requested_path: &str) -> Result<PathBuf, ExtensionError> {
        let path = Path::new(requested_path);
        let Some(allowed) = &self.allowed_paths else {
            return Err(ExtensionError::ProcessNoAllowedPaths);
        };

        let canonical_target = if path.is_absolute() {
            dunce::canonicalize(path)
                .map_err(|_| ExtensionError::ProcessBinaryNotFound(path.to_path_buf()))?
        } else {
            let mut resolved = None;
            for base_dir in allowed.values() {
                let candidate = base_dir.join(path);
                if let Ok(c) = dunce::canonicalize(&candidate) {
                    resolved = Some(c);
                    break;
                }
            }
            resolved.ok_or_else(|| {
                ExtensionError::ProcessBinaryNotFound(PathBuf::from(requested_path))
            })?
        };

        let in_allowed_dir = allowed.values().any(|base| {
            if let Ok(canonical_base) = dunce::canonicalize(base) {
                canonical_target.starts_with(&canonical_base)
            } else {
                false
            }
        });

        if !in_allowed_dir {
            return Err(ExtensionError::ProcessBinaryNotPermitted(canonical_target));
        }

        Ok(canonical_target)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn verify_binary_hash(&self, binary_path: &Path) -> Result<(), ExtensionError> {
        let file_bytes = fs::read(binary_path).map_err(|e| {
            ExtensionError::ProcessFailedToReadBinary(binary_path.to_path_buf(), e.to_string())
        })?;
        let computed_hash = sha256_hex(&file_bytes);

        let is_allowed = self
            .allowed_binaries
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(&computed_hash));

        if !is_allowed {
            return Err(ExtensionError::ProcessHashNotPermitted(computed_hash));
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn capture_stdout(stdout: Option<ChildStdout>, buffer: Arc<Mutex<Vec<u8>>>) {
        let Some(mut stdout_stream) = stdout else {
            return;
        };
        std::thread::spawn(move || {
            let mut chunk = [0u8; 4096];
            while let Ok(n) = stdout_stream.read(&mut chunk) {
                if n == 0 {
                    break;
                }
                let mut guard = buffer.lock().unwrap();
                let cur_len = guard.len();
                if cur_len + n > 1024 * 1024 {
                    let drain_len = (cur_len + n) - 1024 * 1024;
                    guard.drain(0..drain_len.min(cur_len));
                }
                guard.extend_from_slice(&chunk[..n]);
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn spawn_process(
        &self,
        req: SpawnProcessRequest,
    ) -> Result<SpawnProcessResponse, ExtensionError> {
        let binary_path = self.resolve_path(&req.path)?;
        self.verify_binary_hash(&binary_path)?;

        let mut command = Command::new(&binary_path);
        command.args(&req.args);
        for (k, v) in &req.env {
            command.env(k, v);
        }
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::null());

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }

        let mut child = command
            .spawn()
            .map_err(|e| ExtensionError::ProcessFailedToSpawn(e.to_string()))?;

        let pid = child.id();
        let stdin = Mutex::new(child.stdin.take());
        let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
        Self::capture_stdout(child.stdout.take(), stdout_buffer.clone());

        let process_id = self.next_process_id.fetch_add(1, Ordering::SeqCst);
        let handle = ProcessHandle {
            process_id,
            pid,
            stdin,
            stdout_buffer,
            is_running: Arc::new(AtomicBool::new(true)),
            exit_code: Arc::new(Mutex::new(None)),
            child: Mutex::new(Some(child)),
        };

        self.processes.lock().unwrap().insert(process_id, handle);

        Ok(SpawnProcessResponse {
            process_id: process_id as u64,
            pid,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn write_process_stdin(
        &self,
        req: WriteProcessStdinRequest,
    ) -> Result<WriteProcessStdinResponse, ExtensionError> {
        let proc_id = req.process_id as usize;
        let processes = self.processes.lock().unwrap();
        let handle = processes
            .get(&proc_id)
            .ok_or(ExtensionError::ProcessNotFound(proc_id))?;

        let mut stdin_guard = handle.stdin.lock().unwrap();
        let Some(stdin) = stdin_guard.as_mut() else {
            return Err(ExtensionError::ProcessStdinNotOpen(proc_id));
        };

        stdin
            .write_all(&req.data)
            .map_err(|e| ExtensionError::ProcessFailedToWrite(proc_id, e.to_string()))?;
        stdin
            .flush()
            .map_err(|e| ExtensionError::ProcessFailedToFlush(proc_id, e.to_string()))?;

        Ok(WriteProcessStdinResponse {
            bytes_written: req.data.len() as u64,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn read_process_stdout(
        &self,
        req: ReadProcessStdoutRequest,
    ) -> Result<ReadProcessStdoutResponse, ExtensionError> {
        let proc_id = req.process_id as usize;
        let processes = self.processes.lock().unwrap();
        let handle = processes
            .get(&proc_id)
            .ok_or(ExtensionError::ProcessNotFound(proc_id))?;

        let max_bytes = req.max_bytes.unwrap_or(65536) as usize;
        let mut buffer = handle.stdout_buffer.lock().unwrap();
        let take_len = buffer.len().min(max_bytes);
        let data: Vec<u8> = buffer.drain(0..take_len).collect();

        Ok(ReadProcessStdoutResponse { data })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn kill_process(
        &self,
        req: KillProcessRequest,
    ) -> Result<KillProcessResponse, ExtensionError> {
        let proc_id = req.process_id as usize;
        let mut processes = self.processes.lock().unwrap();
        let Some(handle) = processes.remove(&proc_id) else {
            return Ok(KillProcessResponse { success: false });
        };

        Self::kill_handle(&handle);

        Ok(KillProcessResponse { success: true })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_process_status(
        &self,
        req: GetProcessStatusRequest,
    ) -> Result<GetProcessStatusResponse, ExtensionError> {
        let proc_id = req.process_id as usize;
        let processes = self.processes.lock().unwrap();
        let handle = processes
            .get(&proc_id)
            .ok_or(ExtensionError::ProcessNotFound(proc_id))?;

        let mut child_guard = handle.child.lock().unwrap();
        if let Some(child) = child_guard.as_mut()
            && let Ok(Some(status)) = child.try_wait()
        {
            handle.is_running.store(false, Ordering::SeqCst);
            *handle.exit_code.lock().unwrap() = status.code();
        }

        let is_running = handle.is_running.load(Ordering::SeqCst);
        let exit_code = *handle.exit_code.lock().unwrap();

        Ok(GetProcessStatusResponse {
            is_running,
            exit_code,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn kill_handle(handle: &ProcessHandle) {
        handle.is_running.store(false, Ordering::SeqCst);
        let mut child_guard = handle.child.lock().unwrap();
        if let Some(mut child) = child_guard.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn kill_all(&self) {
        let mut processes = self.processes.lock().unwrap();
        for (_, handle) in processes.drain() {
            Self::kill_handle(&handle);
        }
    }
}

impl Drop for ExtensionProcessManager {
    #[tracing::instrument(level = "debug", skip_all)]
    fn drop(&mut self) { self.kill_all(); }
}
