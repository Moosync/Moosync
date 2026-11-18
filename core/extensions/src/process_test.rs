use std::{
    collections::HashMap,
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

use extensions_proto::moosync::types::{
    BinaryPermission, GetProcessStatusRequest, KillProcessRequest, ReadProcessStdoutRequest,
    SpawnProcessRequest, WriteProcessStdinRequest,
};

use crate::process::{ExtensionProcessManager, md5_hex, sha256_hex, sha512_hex};

#[cfg(unix)]
fn create_test_binary(dir: &Path, name: &str, script_content: &str) -> (PathBuf, BinaryPermission) {
    let binary_path = dir.join(name);
    fs::write(&binary_path, script_content).unwrap();
    let mut perms = fs::metadata(&binary_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&binary_path, perms).unwrap();

    let bytes = fs::read(&binary_path).unwrap();
    let mut hashes = HashMap::new();
    hashes.insert("sha256".to_string(), sha256_hex(&bytes));
    hashes.insert("sha512".to_string(), sha512_hex(&bytes));
    hashes.insert("md5".to_string(), md5_hex(&bytes));

    let perm = BinaryPermission {
        name: name.to_string(),
        hashes,
        justification: "Test binary".to_string(),
    };
    (binary_path, perm)
}

#[test]
#[cfg(unix)]
fn test_process_manager_lifecycle_and_stdio() {
    let tmp = tempdir::TempDir::new("test_proc_mgr").unwrap();
    let script = "#!/bin/sh\nread line\necho \"REPLY:$line\"\n";
    let (bin_path, perm) = create_test_binary(tmp.path(), "echo_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("test".to_string(), tmp.path().to_path_buf());

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![perm]);

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
    let (bin_path, _perm) = create_test_binary(tmp.path(), "test_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("test".to_string(), tmp.path().to_path_buf());

    let bad_perm = BinaryPermission {
        name: "test_bin".to_string(),
        hashes: [(
            "sha256".to_string(),
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        )]
        .into_iter()
        .collect(),
        justification: "test".to_string(),
    };

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![bad_perm]);

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
fn test_process_manager_sha512_and_md5_verification() {
    let tmp = tempdir::TempDir::new("test_multi_hash").unwrap();
    let script = "#!/bin/sh\necho ok\n";
    let (bin_path, mut perm) = create_test_binary(tmp.path(), "test_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("test".to_string(), tmp.path().to_path_buf());

    let mgr = ExtensionProcessManager::new(Some(allowed_paths.clone()), vec![perm.clone()]);
    let spawn_res = mgr.spawn_process(SpawnProcessRequest {
        path: bin_path.file_name().unwrap().to_str().unwrap().to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_res.is_ok());

    perm.hashes
        .insert("md5".to_string(), "badmd5hash".to_string());
    let mgr_bad_md5 = ExtensionProcessManager::new(Some(allowed_paths.clone()), vec![perm.clone()]);
    let spawn_bad_md5 = mgr_bad_md5.spawn_process(SpawnProcessRequest {
        path: bin_path.file_name().unwrap().to_str().unwrap().to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_bad_md5.is_err());

    perm.hashes.remove("md5");
    perm.hashes
        .insert("sha512".to_string(), "badsha512hash".to_string());
    let mgr_bad_sha512 = ExtensionProcessManager::new(Some(allowed_paths), vec![perm]);
    let spawn_bad_sha512 = mgr_bad_sha512.spawn_process(SpawnProcessRequest {
        path: bin_path.file_name().unwrap().to_str().unwrap().to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_bad_sha512.is_err());
}

#[test]
#[cfg(unix)]
fn test_process_manager_unpermitted_binary_name_rejected() {
    let tmp = tempdir::TempDir::new("test_unpermitted_bin").unwrap();
    let script = "#!/bin/sh\necho unpermitted\n";
    let (bin_path, perm) = create_test_binary(tmp.path(), "test_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("test".to_string(), tmp.path().to_path_buf());

    let mut unpermitted_perm = perm;
    unpermitted_perm.name = "other_bin".to_string();

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![unpermitted_perm]);
    let spawn_res = mgr.spawn_process(SpawnProcessRequest {
        path: bin_path.file_name().unwrap().to_str().unwrap().to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_res.is_err());
    let err_str = spawn_res.err().unwrap().to_string();
    assert!(err_str.contains("Binary") || err_str.contains("permitted"));
}

#[test]
#[cfg(unix)]
fn test_process_manager_path_traversal_rejected() {
    let tmp = tempdir::TempDir::new("test_traversal").unwrap();
    let script = "#!/bin/sh\necho hi\n";
    let (_bin_path, perm) = create_test_binary(tmp.path(), "test_bin", script);

    let sub_dir = tmp.path().join("sub");
    fs::create_dir_all(&sub_dir).unwrap();

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("sub".to_string(), sub_dir);

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![perm]);

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
    let (_bin_path, perm) = create_test_binary(&sub, "test_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("root".to_string(), tmp.path().to_path_buf());

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![perm]);

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
    let (outside_bin, perm) = create_test_binary(tmp2.path(), "outside_bin", script);

    let mut allowed_paths = HashMap::new();
    allowed_paths.insert("test".to_string(), tmp1.path().to_path_buf());

    let mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![perm]);

    let spawn_res = mgr.spawn_process(SpawnProcessRequest {
        path: outside_bin.to_string_lossy().to_string(),
        args: vec![],
        env: Default::default(),
    });
    assert!(spawn_res.is_err());
    let err_str = spawn_res.err().unwrap().to_string();
    assert!(err_str.contains("outside allowed extension paths"));
}
