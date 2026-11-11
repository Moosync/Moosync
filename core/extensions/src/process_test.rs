// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::{
    collections::HashMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

use extensions_proto::moosync::types::{
    GetProcessStatusRequest, KillProcessRequest, ReadProcessStdoutRequest, SpawnProcessRequest,
    WriteProcessStdinRequest,
};

use crate::process::{ExtensionProcessManager, sha256_hex};

#[cfg(unix)]
fn create_test_binary(dir: &Path, name: &str, script_content: &str) -> (PathBuf, String) {
    let binary_path = dir.join(name);
    fs::write(&binary_path, script_content).unwrap();
    let mut perms = fs::metadata(&binary_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&binary_path, perms).unwrap();

    let bytes = fs::read(&binary_path).unwrap();
    let hash = sha256_hex(&bytes);
    (binary_path, hash)
}

#[test]
#[cfg(unix)]
fn test_process_manager_lifecycle_and_stdio() {
    let tmp = tempdir::TempDir::new("test_proc_mgr").unwrap();
    let script = "#!/bin/sh\nread line\necho \"REPLY:$line\"\n";
    let (bin_path, hash) = create_test_binary(tmp.path(), "echo_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("test".to_string(), tmp.path().to_path_buf());

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![hash]);

    let spawn_res = mgr.spawn_process(SpawnProcessRequest {
        path: bin_path.file_name().unwrap().to_str().unwrap().to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_res.is_ok());
    let proc_info = spawn_res.unwrap();
    assert_eq!(proc_info.process_id, 1);
    assert!(proc_info.pid > 0);

    let write_res = mgr.write_process_stdin(WriteProcessStdinRequest {
        process_id: proc_info.process_id,
        data: b"hello world\n".to_vec(),
    });
    assert!(write_res.is_ok());
    assert_eq!(write_res.unwrap().bytes_written, 12);

    std::thread::sleep(Duration::from_millis(150));

    let read_res = mgr.read_process_stdout(ReadProcessStdoutRequest {
        process_id: proc_info.process_id,
        max_bytes: Some(1024),
    });
    assert!(read_res.is_ok());
    let stdout_data = read_res.unwrap().data;
    assert_eq!(String::from_utf8_lossy(&stdout_data), "REPLY:hello world\n");

    let status_res = mgr.get_process_status(GetProcessStatusRequest {
        process_id: proc_info.process_id,
    });
    assert!(status_res.is_ok());

    let kill_res = mgr.kill_process(KillProcessRequest {
        process_id: proc_info.process_id,
    });
    assert!(kill_res.is_ok());
    assert!(kill_res.unwrap().success);
}

#[test]
#[cfg(unix)]
fn test_process_manager_hash_mismatch_rejected() {
    let tmp = tempdir::TempDir::new("test_hash_reject").unwrap();
    let script = "#!/bin/sh\necho hi\n";
    let (bin_path, _hash) = create_test_binary(tmp.path(), "test_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("test".to_string(), tmp.path().to_path_buf());

    let mgr = ExtensionProcessManager::new(
        Some(allowed_paths),
        vec!["0000000000000000000000000000000000000000000000000000000000000000".to_string()],
    );

    let spawn_res = mgr.spawn_process(SpawnProcessRequest {
        path: bin_path.file_name().unwrap().to_str().unwrap().to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_res.is_err());
    let err_str = spawn_res.err().unwrap().to_string();
    assert!(err_str.contains("Binary hash"));
}

#[test]
#[cfg(unix)]
fn test_process_manager_path_traversal_rejected() {
    let tmp = tempdir::TempDir::new("test_traversal").unwrap();
    let script = "#!/bin/sh\necho hi\n";
    let (_bin_path, hash) = create_test_binary(tmp.path(), "test_bin", script);

    let sub_dir = tmp.path().join("sub");
    fs::create_dir_all(&sub_dir).unwrap();

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("sub".to_string(), sub_dir);

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![hash]);

    let spawn_res = mgr.spawn_process(SpawnProcessRequest {
        path: "../test_bin".to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_res.is_err());
    let err_str = spawn_res.err().unwrap().to_string();
    assert!(err_str.contains("outside allowed extension paths"));
}

#[test]
#[cfg(unix)]
fn test_process_manager_internal_parent_dir_allowed() {
    let tmp = tempdir::TempDir::new("test_internal_parent").unwrap();
    let sub = tmp.path().join("sub");
    fs::create_dir_all(&sub).unwrap();
    let script = "#!/bin/sh\necho hi\n";
    let (_bin_path, hash) = create_test_binary(&sub, "test_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("root".to_string(), tmp.path().to_path_buf());

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![hash]);

    let spawn_res = mgr.spawn_process(SpawnProcessRequest {
        path: "sub/../sub/test_bin".to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_res.is_ok());
}

#[test]
#[cfg(unix)]
fn test_process_manager_outside_allowed_rejected() {
    let tmp1 = tempdir::TempDir::new("test_allowed").unwrap();
    let tmp2 = tempdir::TempDir::new("test_outside").unwrap();

    let script = "#!/bin/sh\necho outside\n";
    let (outside_bin, hash) = create_test_binary(tmp2.path(), "outside_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("test".to_string(), tmp1.path().to_path_buf());

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![hash]);

    let spawn_res = mgr.spawn_process(SpawnProcessRequest {
        path: outside_bin.to_string_lossy().to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_res.is_err());
    let err_str = spawn_res.err().unwrap().to_string();
    assert!(err_str.contains("outside allowed extension paths"));
}
