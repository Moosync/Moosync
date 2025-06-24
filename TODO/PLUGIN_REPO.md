# Spec for plugin repositories

This document outlines the specifications for plugin repositories in Moosync.

## Repository Structure

Plugin repositories follow a standardized structure for discovery, distribution, and management:

```
repository-root/
├── index.json              # Repository manifest
├── packages/               # Individual plugin packages
│   ├── plugin-name-1/
│   │   ├── 1.0.0/
│   │   │   ├── package.zip
│   │   │   ├── manifest.json
│   │   │   └── signature.sig
│   │   ├── 1.1.0/
│   │   └── latest -> 1.1.0/
│   └── plugin-name-2/
├── categories/             # Category-based organization
│   ├── music-providers.json
│   ├── utilities.json
│   └── themes.json
├── security/
│   ├── public-keys/        # Repository signing keys
│   └── blocklist.json      # Blocked packages
└── metadata/
    ├── stats.json          # Download statistics
    └── featured.json       # Featured packages
```

### Repository Index (index.json)

```json
{
  "version": "1.0",
  "name": "Official Moosync Repository",
  "description": "Official repository for Moosync extensions",
  "url": "https://repo.moosync.app",
  "maintainer": {
    "name": "Moosync Team",
    "email": "support@moosync.app",
    "website": "https://moosync.app"
  },
  "security": {
    "signing_required": true,
    "public_key_url": "security/public-keys/repo-key.pem",
    "key_fingerprint": "SHA256:abc123...",
    "verification_method": "rsa-pss"
  },
  "packages": {
    "total": 42,
    "last_updated": "2024-06-24T12:00:00Z",
    "endpoint": "/packages"
  },
  "api": {
    "version": "v1",
    "endpoints": {
      "search": "/api/v1/search",
      "package": "/api/v1/packages/{package_name}",
      "download": "/api/v1/download/{package_name}/{version}"
    }
  }
}
```

## Signature verification

All packages in repositories must be cryptographically signed to ensure integrity and authenticity.

### Signing Process:

1. **Package Creation**: Developer creates extension package
2. **Hash Generation**: SHA-256 hash of the package contents
3. **Signature Creation**: RSA-PSS signature of the hash using private key
4. **Public Key Distribution**: Public key stored in repository security directory

### Verification Steps:

```rust
// Pseudo-code for verification process
fn verify_package(package_path: &Path, signature: &[u8], public_key: &[u8]) -> Result<bool> {
    // 1. Calculate package hash
    let package_hash = sha256_hash_file(package_path)?;
    
    // 2. Verify signature against hash
    let public_key = rsa::RsaPublicKey::from_der(public_key)?;
    let verify_result = public_key.verify_pss(
        &package_hash,
        signature,
        &rsa::pss::BlindedSigningKey::new(public_key)
    )?;
    
    // 3. Check against blocklist
    if is_package_blocked(&package_hash)? {
        return Err("Package is blocked");
    }
    
    Ok(verify_result)
}
```

### Security Requirements:

- **RSA-PSS**: Minimum 2048-bit keys, recommended 4096-bit
- **Hash Algorithm**: SHA-256 for package integrity
- **Key Rotation**: Support for key rotation with transition periods
- **Revocation**: Certificate revocation list for compromised keys
- **Chain of Trust**: Repository keys signed by root certificate authority

### Signature File Format (signature.sig):

```json
{
  "version": "1.0",
  "algorithm": "rsa-pss",
  "hash_algorithm": "sha256",
  "signature": "base64-encoded-signature",
  "public_key_fingerprint": "SHA256:abc123...",
  "timestamp": "2024-06-24T12:00:00Z",
  "metadata": {
    "package_hash": "sha256:def456...",
    "signer": "developer@example.com",
    "build_info": {
      "build_id": "12345",
      "commit_hash": "abcdef123456"
    }
  }
}
```

## OTA updates

Over-the-air updates enable automatic extension updates without manual intervention.

### Update Discovery:

1. **Version Check**: Client periodically checks for updates
2. **Delta Detection**: Compare local and remote versions
3. **Update Queue**: Queue compatible updates for installation
4. **User Notification**: Notify user of available updates

### Update Process:

```rust
// Update flow implementation
async fn check_for_updates() -> Result<Vec<UpdateInfo>> {
    let installed_extensions = get_installed_extensions().await?;
    let mut updates = Vec::new();
    
    for extension in installed_extensions {
        let remote_info = fetch_remote_package_info(&extension.name).await?;
        
        if is_update_available(&extension.version, &remote_info.version) {
            // Check compatibility
            if is_compatible(&remote_info.compatibility) {
                updates.push(UpdateInfo {
                    name: extension.name,
                    current_version: extension.version,
                    new_version: remote_info.version,
                    changelog: remote_info.changelog,
                    size: remote_info.size,
                    priority: remote_info.priority,
                });
            }
        }
    }
    
    Ok(updates)
}
```

### Update Configuration:

```json
{
  "update_policy": {
    "auto_check": true,
    "check_interval": "24h",
    "auto_install": {
      "enabled": false,
      "security_only": true,
      "exclude_beta": true
    },
    "notification": {
      "show_available": true,
      "show_progress": true,
      "show_completed": true
    }
  },
  "channels": {
    "stable": {
      "enabled": true,
      "priority": 1
    },
    "beta": {
      "enabled": false,
      "priority": 2
    },
    "nightly": {
      "enabled": false,
      "priority": 3
    }
  }
}
```

### Update Metadata:

```json
{
  "package_name": "spotify-provider",
  "versions": {
    "1.2.0": {
      "release_date": "2024-06-24",
      "changelog": "Fixed authentication issues",
      "priority": "medium",
      "compatibility": {
        "moosync_version": ">=2.0.0",
        "platforms": ["windows", "linux", "macos"]
      },
      "download": {
        "url": "/packages/spotify-provider/1.2.0/package.zip",
        "size": 2048576,
        "checksum": "sha256:abc123..."
      }
    }
  }
}
```

## Permission reporting

Extensions must declare and report their required permissions for transparency and security.

### Permission Categories:

1. **Core Permissions**:
   - `songs.read`: Read song metadata
   - `songs.write`: Modify song data
   - `playlists.read`: Access playlist data
   - `playlists.write`: Modify playlists
   - `player.control`: Control playback
   - `preferences.read`: Read user preferences
   - `preferences.write`: Modify preferences

2. **Network Permissions**:
   - `network.http`: Make HTTP requests
   - `network.websocket`: WebSocket connections
   - `network.hosts`: Specific host access

3. **System Permissions**:
   - `filesystem.read`: Read files
   - `filesystem.write`: Write files
   - `system.notifications`: Show notifications
   - `system.tray`: System tray access

### Permission Declaration:

```json
{
  "permissions": {
    "required": [
      {
        "name": "songs.read",
        "reason": "Access song metadata for search functionality"
      },
      {
        "name": "network.http",
        "reason": "Connect to Spotify API for music streaming",
        "hosts": ["api.spotify.com", "accounts.spotify.com"]
      }
    ],
    "optional": [
      {
        "name": "system.notifications",
        "reason": "Show track change notifications",
        "default": false
      }
    ]
  }
}
```

### Runtime Permission Reporting:

```json
{
  "extension_id": "spotify-provider",
  "permissions_used": [
    {
      "permission": "network.http",
      "usage_count": 42,
      "last_used": "2024-06-24T12:00:00Z",
      "hosts_accessed": ["api.spotify.com"]
    },
    {
      "permission": "songs.write",
      "usage_count": 15,
      "last_used": "2024-06-24T11:30:00Z"
    }
  ],
  "permission_violations": [],
  "report_timestamp": "2024-06-24T12:05:00Z"
}
```

### User Consent Flow:

1. **Installation**: Show permission dialog during installation
2. **Runtime Requests**: Request additional permissions as needed
3. **Audit Trail**: Log all permission usage for review
4. **Revocation**: Allow users to revoke permissions
5. **Transparency**: Show permission usage in extension details

### Security Monitoring:

- **Anomaly Detection**: Monitor for unusual permission usage
- **Rate Limiting**: Limit API calls and resource usage
- **Sandboxing**: Isolate extensions in secure execution environment
- **Violation Reporting**: Report permission violations to repository
- **Automatic Suspension**: Suspend extensions with serious violations

## Repository API

### Endpoints:

- `GET /api/v1/packages` - List all packages
- `GET /api/v1/packages/{name}` - Get package details
- `GET /api/v1/packages/{name}/{version}` - Get specific version
- `GET /api/v1/search?q={query}` - Search packages
- `GET /api/v1/categories/{category}` - Get packages by category
- `POST /api/v1/report` - Report security issues

### Authentication:

- **API Keys**: For authenticated operations
- **Rate Limiting**: Prevent abuse
- **Analytics**: Track download statistics
