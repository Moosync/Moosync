#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    mem::take,
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use extensions_proto::moosync::types::{
    BinaryPermission, ExtensionCommand, GetProcessStatusRequest, GetProcessStatusResponse,
    KillProcessRequest, KillProcessResponse, ProcessEventRequest, ProcessOutput,
    ReadProcessStdoutRequest, ReadProcessStdoutResponse, SpawnProcessRequest, SpawnProcessResponse,
    WriteProcessStdinRequest, WriteProcessStdinResponse, extension_command, process_event_request,
};
use md5::Md5;
use sha2::{Digest, Sha256, Sha512};
use tokio::{
    runtime::Handle,
    sync::mpsc,
    time::{Instant, sleep_until},
};
use tracing::Instrument;

use crate::{
    errors::ExtensionError,
    event_sink::{ExtensionEventSink, context_event_sink},
    extension::AsyncContext,
};

const OUTPUT_DEBOUNCE_MAX_BYTES: usize = 8192;
const OUTPUT_DEBOUNCE_WINDOW: Duration = Duration::from_millis(50);

enum StreamChunk {
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn sha512_hex(data: &[u8]) -> String {
    let mut hasher = Sha512::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn md5_hex(data: &[u8]) -> String {
    let mut hasher = Md5::new();
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
    allowed_binaries: Vec<BinaryPermission>,
    sink: Option<Arc<dyn ExtensionEventSink>>,
}

impl ExtensionProcessManager {
    #[tracing::instrument(level = "debug", skip_all)]
    fn is_path_like(arg: &str) -> bool {
        if arg.starts_with('/')
            || arg.starts_with("./")
            || arg.starts_with("../")
            || arg.starts_with("~/")
        {
            return true;
        }

        let bytes = arg.as_bytes();
        if bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'\\' || bytes[2] == b'/')
        {
            return true;
        }

        if arg.starts_with(r"\") || arg.starts_with(r".\") || arg.starts_with(r"..\") {
            return true;
        }

        false
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        allowed_paths: Option<HashMap<String, PathBuf>>,
        allowed_binaries: Vec<BinaryPermission>,
    ) -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            next_process_id: AtomicUsize::new(1),
            allowed_paths,
            allowed_binaries,
            sink: None,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn new_with_context(
        allowed_paths: Option<HashMap<String, PathBuf>>,
        allowed_binaries: Vec<BinaryPermission>,
        context: Weak<AsyncContext>,
    ) -> Self {
        Self::with_sink(allowed_paths, allowed_binaries, context_event_sink(context))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn with_sink(
        allowed_paths: Option<HashMap<String, PathBuf>>,
        allowed_binaries: Vec<BinaryPermission>,
        sink: Arc<dyn ExtensionEventSink>,
    ) -> Self {
        Self {
            processes: Arc::new(Mutex::new(HashMap::new())),
            next_process_id: AtomicUsize::new(1),
            allowed_paths,
            allowed_binaries,
            sink: Some(sink),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn is_in_allowed(&self, target: &Path, allowed: &HashMap<String, PathBuf>) -> bool {
        allowed.values().any(|base| {
            if let Ok(canonical_base) = dunce::canonicalize(base) {
                target.starts_with(&canonical_base)
            } else {
                false
            }
        }) || allowed.keys().any(|base| {
            if let Ok(canonical_base) = dunce::canonicalize(base) {
                target.starts_with(&canonical_base)
            } else {
                false
            }
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn resolve_path(&self, requested_path: &str) -> Result<PathBuf, ExtensionError> {
        let path = Path::new(requested_path);
        let Some(allowed) = &self.allowed_paths else {
            return Err(ExtensionError::ProcessNoAllowedPaths);
        };

        let mut resolved_candidate = None;

        for (host_dir, guest_dir) in allowed {
            if let Ok(rel) = path.strip_prefix(guest_dir) {
                let candidate = Path::new(host_dir).join(rel);
                if let Ok(c) = dunce::canonicalize(&candidate) {
                    resolved_candidate = Some(c);
                    break;
                }
            }
        }

        if resolved_candidate.is_none() && path.is_absolute() {
            resolved_candidate = dunce::canonicalize(path).ok();
        }

        if resolved_candidate.is_none() {
            for host_dir in allowed.keys() {
                let candidate = Path::new(host_dir).join(path);
                if let Ok(c) = dunce::canonicalize(&candidate) {
                    resolved_candidate = Some(c);
                    break;
                }
            }
            if resolved_candidate.is_none() {
                for base_dir in allowed.values() {
                    let candidate = base_dir.join(path);
                    if let Ok(c) = dunce::canonicalize(&candidate) {
                        resolved_candidate = Some(c);
                        break;
                    }
                }
            }
        }

        let canonical_target = resolved_candidate
            .ok_or_else(|| ExtensionError::ProcessBinaryNotFound(PathBuf::from(requested_path)))?;

        if !self.is_in_allowed(&canonical_target, allowed) {
            return Err(ExtensionError::ProcessBinaryNotPermitted(canonical_target));
        }

        Ok(canonical_target)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn verify_binary_hash(
        &self,
        binary_path: &Path,
        requested_path: &str,
    ) -> Result<(), ExtensionError> {
        let matching_perm = self.allowed_binaries.iter().find(|b| {
            if b.name == requested_path {
                return true;
            }
            if let Ok(resolved) = self.resolve_path(&b.name)
                && resolved == binary_path
            {
                return true;
            }
            if binary_path.ends_with(&b.name) {
                return true;
            }
            false
        });

        let Some(perm) = matching_perm else {
            return Err(ExtensionError::ProcessBinaryNotPermitted(
                binary_path.to_path_buf(),
            ));
        };

        if perm.hashes.is_empty() {
            return Err(ExtensionError::ProcessHashNotPermitted(String::new()));
        }

        let file_bytes = fs::read(binary_path).map_err(|e| {
            ExtensionError::ProcessFailedToReadBinary(binary_path.to_path_buf(), e.to_string())
        })?;

        for (algo, expected_hash) in &perm.hashes {
            match algo.to_ascii_lowercase().as_str() {
                "sha256" => {
                    let computed = sha256_hex(&file_bytes);
                    if !computed.eq_ignore_ascii_case(expected_hash) {
                        return Err(ExtensionError::ProcessHashNotPermitted(computed));
                    }
                }
                "sha512" => {
                    let computed = sha512_hex(&file_bytes);
                    if !computed.eq_ignore_ascii_case(expected_hash) {
                        return Err(ExtensionError::ProcessHashNotPermitted(computed));
                    }
                }
                "md5" => {
                    let computed = md5_hex(&file_bytes);
                    if !computed.eq_ignore_ascii_case(expected_hash) {
                        return Err(ExtensionError::ProcessHashNotPermitted(computed));
                    }
                }
                _ => {
                    return Err(ExtensionError::ProcessHashNotPermitted(format!(
                        "unsupported algo: {algo}"
                    )));
                }
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn flush_output(
        sink: &Arc<dyn ExtensionEventSink>,
        process_id: usize,
        stdout_buf: &mut Vec<u8>,
        stderr_buf: &mut Vec<u8>,
    ) {
        if stdout_buf.is_empty() && stderr_buf.is_empty() {
            return;
        }
        let cmd = ExtensionCommand {
            package_name: String::new(),
            event: Some(extension_command::Event::ProcessEvent(
                ProcessEventRequest {
                    process_id: process_id as u64,
                    event: Some(process_event_request::Event::Output(ProcessOutput {
                        stdout: take(stdout_buf),
                        stderr: take(stderr_buf),
                    })),
                },
            )),
        };
        if let Err(err) = sink.dispatch(cmd).await {
            tracing::error!(
                "Failed to dispatch process output event for {}: {:?}",
                process_id,
                err
            );
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn append_chunk(
        chunk: StreamChunk,
        stdout_buf: &mut Vec<u8>,
        stderr_buf: &mut Vec<u8>,
    ) -> bool {
        match chunk {
            StreamChunk::Stdout(data) => stdout_buf.extend_from_slice(&data),
            StreamChunk::Stderr(data) => stderr_buf.extend_from_slice(&data),
        }
        stdout_buf.len() + stderr_buf.len() >= OUTPUT_DEBOUNCE_MAX_BYTES
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn run_output_debouncer(
        process_id: usize,
        sink: Arc<dyn ExtensionEventSink>,
        processes: Arc<Mutex<HashMap<usize, ProcessHandle>>>,
        mut rx: mpsc::UnboundedReceiver<StreamChunk>,
    ) {
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();
        let mut deadline: Option<Instant> = None;

        loop {
            let chunk = match deadline {
                Some(flush_at) => {
                    tokio::select! {
                        chunk = rx.recv() => chunk,
                        _ = sleep_until(flush_at) => {
                            Self::flush_output(&sink, process_id, &mut stdout_buf, &mut stderr_buf).await;
                            deadline = None;
                            continue;
                        }
                    }
                }
                None => rx.recv().await,
            };

            let Some(chunk) = chunk else {
                Self::flush_output(&sink, process_id, &mut stdout_buf, &mut stderr_buf).await;
                break;
            };

            if Self::append_chunk(chunk, &mut stdout_buf, &mut stderr_buf) {
                Self::flush_output(&sink, process_id, &mut stdout_buf, &mut stderr_buf).await;
                deadline = None;
            } else if deadline.is_none() {
                deadline = Some(Instant::now() + OUTPUT_DEBOUNCE_WINDOW);
            }
        }

        let exit_code = {
            let guard = processes.lock().unwrap();
            let Some(handle) = guard.get(&process_id) else {
                return;
            };
            let mut child_guard = handle.child.lock().unwrap();
            if let Some(child) = child_guard.as_mut() {
                match child.wait() {
                    Ok(status) => {
                        handle.is_running.store(false, Ordering::SeqCst);
                        *handle.exit_code.lock().unwrap() = status.code();
                        status.code()
                    }
                    Err(_) => *handle.exit_code.lock().unwrap(),
                }
            } else {
                *handle.exit_code.lock().unwrap()
            }
        };

        let exit_cmd = ExtensionCommand {
            package_name: String::new(),
            event: Some(extension_command::Event::ProcessEvent(
                ProcessEventRequest {
                    process_id: process_id as u64,
                    event: Some(process_event_request::Event::Exited(
                        exit_code.unwrap_or(-1),
                    )),
                },
            )),
        };
        if let Err(err) = sink.dispatch(exit_cmd).await {
            tracing::debug!(
                "Failed to dispatch process exited event for process {}: {}",
                process_id,
                err
            );
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn capture_stream<R: Read + Send + 'static>(
        stream: Option<R>,
        buffer: Option<Arc<Mutex<Vec<u8>>>>,
        is_stderr: bool,
        output_tx: Option<mpsc::UnboundedSender<StreamChunk>>,
    ) {
        let Some(mut reader) = stream else {
            return;
        };
        thread::spawn(move || {
            let mut chunk = [0u8; 4096];
            while let Ok(n) = reader.read(&mut chunk) {
                if n == 0 {
                    break;
                }
                let data = chunk[..n].to_vec();
                if let Some(buf) = &buffer {
                    let mut guard = buf.lock().unwrap();
                    let cur_len = guard.len();
                    if cur_len + n > 1024 * 1024 {
                        let drain_len = (cur_len + n) - 1024 * 1024;
                        guard.drain(0..drain_len.min(cur_len));
                    }
                    guard.extend_from_slice(&data);
                }
                if let Some(tx) = &output_tx {
                    let msg = if is_stderr {
                        StreamChunk::Stderr(data)
                    } else {
                        StreamChunk::Stdout(data)
                    };
                    if let Err(err) = tx.send(msg) {
                        tracing::debug!("Process output receiver closed: {}", err);
                    }
                }
            }
        });
    }

    #[cfg(target_os = "android")]
    #[tracing::instrument(level = "debug", skip_all)]
    fn android_native_lib_dir() -> Option<PathBuf> {
        let maps = fs::read_to_string("/proc/self/maps").ok()?;
        for line in maps.lines() {
            if let Some(path_str) = line.split_whitespace().last() {
                if path_str.ends_with("/libslint_app.so") {
                    return Path::new(path_str).parent().map(Path::to_path_buf);
                }
            }
        }
        None
    }

    #[cfg(target_os = "android")]
    #[tracing::instrument(level = "debug", skip_all)]
    fn build_command(
        &self,
        binary_path: &Path,
        resolved_args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Command, ExtensionError> {
        let mut command = if let Some(ld_lib_path) = env.get("LD_LIBRARY_PATH") {
            let native_dir = Self::android_native_lib_dir().ok_or_else(|| {
                ExtensionError::ProcessFailedToSpawn(
                    "Unable to locate Android native library directory".to_string(),
                )
            })?;
            let ld_linux = native_dir.join("libld_linux.so");
            let resolved_ld_path = self
                .resolve_path(ld_lib_path)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ld_lib_path.clone());
            let mut cmd = Command::new(ld_linux);
            cmd.env_clear();
            cmd.arg("--library-path")
                .arg(resolved_ld_path)
                .arg(binary_path)
                .args(resolved_args);
            cmd
        } else {
            let mut cmd = Command::new("/system/bin/linker64");
            cmd.arg(binary_path).args(resolved_args);
            cmd
        };

        for (k, v) in env {
            if k == "LD_LIBRARY_PATH" {
                continue;
            }
            if k == "LD_PRELOAD" {
                let resolved_preload = self
                    .resolve_path(v)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| v.clone());
                command.env(k, resolved_preload);
                continue;
            }
            command.env(k, v);
        }
        Ok(command)
    }

    #[cfg(not(target_os = "android"))]
    #[tracing::instrument(level = "debug", skip_all)]
    fn build_command(
        &self,
        binary_path: &Path,
        resolved_args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<Command, ExtensionError> {
        let mut command = Command::new(binary_path);
        command.args(resolved_args);
        for (k, v) in env {
            command.env(k, v);
        }
        Ok(command)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn spawn_process(
        &self,
        req: SpawnProcessRequest,
    ) -> Result<SpawnProcessResponse, ExtensionError> {
        let binary_path = self.resolve_path(&req.path)?;
        self.verify_binary_hash(&binary_path, &req.path)?;

        #[cfg(unix)]
        {
            if let Ok(metadata) = fs::metadata(&binary_path) {
                let mut perms = metadata.permissions();
                if perms.mode() & 0o111 == 0 {
                    perms.set_mode(perms.mode() | 0o755);
                    if let Err(err) = fs::set_permissions(&binary_path, perms) {
                        tracing::warn!(
                            "Failed to set executable permissions on {}: {}",
                            binary_path.display(),
                            err
                        );
                    }
                }
            }
        }

        let resolved_args: Vec<String> = req
            .args
            .iter()
            .map(|arg| {
                if Self::is_path_like(arg) {
                    self.resolve_path(arg)
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_else(|_| arg.clone())
                } else {
                    arg.clone()
                }
            })
            .collect();

        let mut command = self.build_command(&binary_path, &resolved_args, &req.env)?;
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(if self.sink.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });

        let mut child = command
            .spawn()
            .map_err(|e| ExtensionError::ProcessFailedToSpawn(e.to_string()))?;

        let pid = child.id();
        let stdin = Mutex::new(child.stdin.take());
        let stdout_stream = child.stdout.take();
        let stderr_stream = child.stderr.take();
        let stdout_buffer = Arc::new(Mutex::new(Vec::new()));
        let process_id = self.next_process_id.fetch_add(1, Ordering::SeqCst);
        let handle = ProcessHandle {
            process_id,
            pid,
            stdin,
            stdout_buffer: stdout_buffer.clone(),
            is_running: Arc::new(AtomicBool::new(true)),
            exit_code: Arc::new(Mutex::new(None)),
            child: Mutex::new(Some(child)),
        };

        self.processes.lock().unwrap().insert(process_id, handle);

        let output_tx =
            if let (Some(sink), Some(rt)) = (self.sink.clone(), Handle::try_current().ok()) {
                let (tx, rx) = mpsc::unbounded_channel();
                rt.spawn(
                    Self::run_output_debouncer(process_id, sink, self.processes.clone(), rx)
                        .in_current_span(),
                );
                Some(tx)
            } else {
                None
            };
        Self::capture_stream(stdout_stream, Some(stdout_buffer), false, output_tx.clone());
        Self::capture_stream(stderr_stream, None, true, output_tx);

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
            if let Err(err) = child.kill() {
                tracing::debug!("Failed to kill child process {}: {}", handle.pid, err);
            }
            if let Err(err) = child.wait() {
                tracing::debug!("Failed to wait for child process {}: {}", handle.pid, err);
            }
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
