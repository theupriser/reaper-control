//! The endpoint file holds the token, so only its owner may read it.

#[cfg(unix)]
#[test]
fn the_endpoint_file_is_readable_only_by_its_owner() -> Result<(), Box<dyn std::error::Error>> {
    use std::os::unix::fs::PermissionsExt;

    use link::Endpoint;

    let directory = std::env::temp_dir().join(format!("rc2-endpoint-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("endpoint.json");
    let endpoint = Endpoint {
        port: 4242,
        token: Endpoint::new_token()?,
        protocol: 3,
    };
    endpoint.write(&path)?;
    let mode = std::fs::metadata(&path)?.permissions().mode() & 0o777;
    assert_eq!(Endpoint::read(&path)?, endpoint);
    std::fs::remove_dir_all(&directory)?;
    assert_eq!(mode, 0o600);
    Ok(())
}
