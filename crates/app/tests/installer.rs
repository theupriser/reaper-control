//! The installer against fake REAPER resource folders in a temporary directory (WP 4.15).
//! The reports are compared with the files in `tests/golden/`; run with `UPDATE_GOLDEN=1` to
//! rewrite them after a deliberate change.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use app::{
    BinaryArchitecture, ExtensionInstaller, ExtensionPreparer, FakeProcessCheck, InstallAction,
    InstallError, InstallItemKind, InstallPlatform, InstallStatus, InstallVerification,
    PlainExtensionPreparer, ReaperInstall, reaper_folder_argument,
};
use protocol::LinkStatus;

type TestResult = Result<(), Box<dyn std::error::Error>>;

const PLATFORM: InstallPlatform = InstallPlatform::MacArm64;
const MACH_O_ARM64: [u8; 8] = [0xcf, 0xfa, 0xed, 0xfe, 0x0c, 0x00, 0x00, 0x01];
const MACH_O_X86_64: [u8; 8] = [0xcf, 0xfa, 0xed, 0xfe, 0x07, 0x00, 0x00, 0x01];

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A folder that is removed when the test ends.
struct TempFolder(PathBuf);

impl TempFolder {
    fn new() -> Result<Self, std::io::Error> {
        let path = std::env::temp_dir().join(format!(
            "rc2-installer-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir_all(&path)?;
        Ok(Self(path))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        // Read-only test folders must be writable again before they can go.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&self.0, fs::Permissions::from_mode(0o755));
        }
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Fixture {
    root: TempFolder,
    install: ReaperInstall,
    process: Arc<FakeProcessCheck>,
    installer: ExtensionInstaller,
}

/// A fake REAPER (`reaper_header` is the start of its program file) and a bundled extension.
fn fixture(reaper_header: &[u8]) -> Result<Fixture, Box<dyn std::error::Error>> {
    fixture_with(reaper_header, Box::new(PlainExtensionPreparer))
}

fn fixture_with(
    reaper_header: &[u8],
    preparer: Box<dyn ExtensionPreparer>,
) -> Result<Fixture, Box<dyn std::error::Error>> {
    let root = TempFolder::new()?;
    let resource = root.path().join("resource");
    fs::create_dir_all(&resource)?;
    let executable = root.path().join("REAPER");
    fs::write(&executable, reaper_header)?;
    let bundled = root.path().join("bundled.dylib");
    fs::write(&bundled, b"extension version two")?;
    let process = Arc::new(FakeProcessCheck::default());
    let installer = ExtensionInstaller::new(PLATFORM, bundled, process.clone(), preparer);
    let install = ReaperInstall {
        resource_folder: resource,
        executable: Some(executable),
    };
    Ok(Fixture {
        root,
        install,
        process,
        installer,
    })
}

fn install_old_extension(fixture: &Fixture) -> TestResult {
    let target = fixture.installer.installed_path(&fixture.install);
    fs::create_dir_all(target.parent().ok_or("no parent")?)?;
    fs::write(target, b"extension version one")?;
    Ok(())
}

fn check_golden(name: &str, fixture: &Fixture) -> TestResult {
    let actual = fixture
        .installer
        .inspect(&fixture.install)
        .to_string()
        .replace(&fixture.root.path().display().to_string(), "<root>");
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(format!("{name}.txt"));
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        fs::create_dir_all(golden.parent().ok_or("no parent")?)?;
        fs::write(&golden, &actual)?;
    }
    // Git may check the file out with Windows line endings.
    let expected = fs::read_to_string(&golden)?.replace("\r\n", "\n");
    assert_eq!(actual, expected, "golden file {name}");
    Ok(())
}

#[test]
fn a_clean_install_copies_the_extension_and_reports_it_healthy() -> TestResult {
    let fixture = fixture(&MACH_O_ARM64)?;
    check_golden("clean", &fixture)?;
    let plan = fixture.installer.install(&fixture.install)?;
    assert_eq!(plan.actions.len(), 2);
    let target = fixture.installer.installed_path(&fixture.install);
    assert_eq!(fs::read(&target)?, b"extension version two");
    assert!(!fixture.installer.backup_path(&fixture.install).exists());
    assert!(fixture.installer.inspect(&fixture.install).is_healthy());
    check_golden("installed", &fixture)
}

#[test]
fn installing_twice_changes_nothing_the_second_time() -> TestResult {
    let fixture = fixture(&MACH_O_ARM64)?;
    fixture.installer.install(&fixture.install)?;
    assert!(fixture.installer.install(&fixture.install)?.is_empty());
    Ok(())
}

#[test]
fn an_existing_older_extension_is_kept_as_a_backup_and_can_be_restored() -> TestResult {
    let fixture = fixture(&MACH_O_ARM64)?;
    install_old_extension(&fixture)?;
    check_golden("older_extension", &fixture)?;
    fixture.installer.install(&fixture.install)?;
    let backup = fixture.installer.backup_path(&fixture.install);
    assert_eq!(fs::read(&backup)?, b"extension version one");
    fixture.installer.uninstall(&fixture.install, true)?;
    let target = fixture.installer.installed_path(&fixture.install);
    assert_eq!(fs::read(target)?, b"extension version one");
    Ok(())
}

#[test]
fn a_dry_run_changes_nothing() -> TestResult {
    let fixture = fixture(&MACH_O_ARM64)?;
    install_old_extension(&fixture)?;
    let plan = fixture.installer.plan(&fixture.install)?;
    assert!(matches!(
        plan.actions.as_slice(),
        [
            InstallAction::KeepPrevious { .. },
            InstallAction::CopyExtension { .. },
            InstallAction::PrepareLibrary { .. }
        ]
    ));
    let target = fixture.installer.installed_path(&fixture.install);
    assert_eq!(fs::read(target)?, b"extension version one");
    assert!(!fixture.installer.backup_path(&fixture.install).exists());
    Ok(())
}

#[test]
fn an_intel_reaper_is_refused_with_guidance() -> TestResult {
    let fixture = fixture(&MACH_O_X86_64)?;
    check_golden("wrong_architecture", &fixture)?;
    let refused = fixture.installer.install(&fixture.install);
    assert!(matches!(
        refused,
        Err(InstallError::WrongArchitecture {
            found: BinaryArchitecture::X64,
            expected: BinaryArchitecture::Arm64
        })
    ));
    assert!(!fixture.installer.installed_path(&fixture.install).exists());
    Ok(())
}

#[test]
fn a_running_reaper_blocks_an_update_but_not_a_first_install() -> TestResult {
    let fixture = fixture(&MACH_O_ARM64)?;
    fixture.process.set_running(true);
    fixture.installer.install(&fixture.install)?;

    let fixture = self::fixture(&MACH_O_ARM64)?;
    install_old_extension(&fixture)?;
    fixture.process.set_running(true);
    check_golden("running_reaper", &fixture)?;
    assert!(matches!(
        fixture.installer.install(&fixture.install),
        Err(InstallError::ReaperIsRunning)
    ));
    assert!(matches!(
        fixture.installer.uninstall(&fixture.install, false),
        Err(InstallError::ReaperIsRunning)
    ));
    let target = fixture.installer.installed_path(&fixture.install);
    assert_eq!(fs::read(target)?, b"extension version one");
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_read_only_folder_is_reported_for_the_user_to_fix() -> TestResult {
    use std::os::unix::fs::PermissionsExt;
    let fixture = fixture(&MACH_O_ARM64)?;
    fs::set_permissions(
        &fixture.install.resource_folder,
        fs::Permissions::from_mode(0o555),
    )?;
    check_golden("read_only", &fixture)?;
    let report = fixture.installer.inspect(&fixture.install);
    assert_eq!(
        report.status_of(InstallItemKind::Writable),
        Some(InstallStatus::Manual)
    );
    assert!(!report.can_install());
    assert!(matches!(
        fixture.installer.install(&fixture.install),
        Err(InstallError::Io { .. })
    ));
    Ok(())
}

#[test]
fn a_missing_resource_folder_is_reported_as_manual() -> TestResult {
    let mut fixture = fixture(&MACH_O_ARM64)?;
    fixture.install.resource_folder = fixture.root.path().join("nowhere");
    let report = fixture.installer.inspect(&fixture.install);
    assert_eq!(
        report.status_of(InstallItemKind::ResourceFolder),
        Some(InstallStatus::Manual)
    );
    Ok(())
}

#[test]
fn a_reaper_without_a_readable_program_has_an_unknown_architecture() -> TestResult {
    let mut fixture = fixture(&MACH_O_ARM64)?;
    fixture.install.executable = None;
    let report = fixture.installer.inspect(&fixture.install);
    assert_eq!(
        report.status_of(InstallItemKind::Architecture),
        Some(InstallStatus::Unknown)
    );
    // Unknown does not stop an install: the handshake decides in the end.
    fixture.installer.install(&fixture.install)?;
    Ok(())
}

#[test]
fn a_portable_install_is_found_from_the_chosen_folder() -> TestResult {
    let root = TempFolder::new()?;
    let portable = root.path().join("portable");
    let executable = PLATFORM.executable_in(&portable);
    fs::create_dir_all(executable.parent().ok_or("no parent")?)?;
    fs::write(&executable, MACH_O_ARM64)?;
    let install = ReaperInstall::locate(PLATFORM, Some(portable.clone()), root.path());
    assert_eq!(install.resource_folder, portable);
    assert_eq!(install.executable, Some(executable));
    let normal = ReaperInstall::locate(PLATFORM, None, root.path());
    assert_eq!(
        normal.resource_folder,
        root.path().join("Library/Application Support/REAPER")
    );
    Ok(())
}

struct FailingPreparer;

impl ExtensionPreparer for FailingPreparer {
    fn prepare(&self, _library: &Path) -> Result<(), String> {
        Err("signing refused".into())
    }
}

#[test]
fn a_failed_preparation_puts_the_old_state_back() -> TestResult {
    let fresh = fixture_with(&MACH_O_ARM64, Box::new(FailingPreparer))?;
    assert!(matches!(
        fresh.installer.install(&fresh.install),
        Err(InstallError::Preparation(_))
    ));
    assert!(!fresh.installer.installed_path(&fresh.install).exists());

    let update = fixture_with(&MACH_O_ARM64, Box::new(FailingPreparer))?;
    install_old_extension(&update)?;
    assert!(update.installer.install(&update.install).is_err());
    let target = update.installer.installed_path(&update.install);
    assert_eq!(fs::read(target)?, b"extension version one");
    Ok(())
}

#[test]
fn repair_restores_a_removed_extension() -> TestResult {
    let fixture = fixture(&MACH_O_ARM64)?;
    fixture.installer.install(&fixture.install)?;
    fs::remove_file(fixture.installer.installed_path(&fixture.install))?;
    assert!(!fixture.installer.repair(&fixture.install)?.is_empty());
    assert!(fixture.installer.inspect(&fixture.install).is_healthy());
    Ok(())
}

#[test]
fn uninstalling_what_is_not_installed_is_an_error() -> TestResult {
    let fixture = fixture(&MACH_O_ARM64)?;
    assert!(matches!(
        fixture.installer.uninstall(&fixture.install, false),
        Err(InstallError::NotInstalled)
    ));
    Ok(())
}

#[test]
fn a_missing_bundle_is_reported() -> TestResult {
    let fixture = fixture(&MACH_O_ARM64)?;
    fs::remove_file(fixture.root.path().join("bundled.dylib"))?;
    assert!(matches!(
        fixture.installer.install(&fixture.install),
        Err(InstallError::BundleMissing { .. })
    ));
    let report = fixture.installer.inspect(&fixture.install);
    assert!(!report.can_install());
    Ok(())
}

#[test]
fn only_the_extension_answering_with_the_right_version_confirms_the_install() {
    let connected = |version: &str| LinkStatus::Connected {
        extension_version: version.into(),
    };
    assert_eq!(
        InstallVerification::from_link(&connected("1.2"), "1.2"),
        InstallVerification::Confirmed
    );
    assert_eq!(
        InstallVerification::from_link(&connected("1.1"), "1.2"),
        InstallVerification::WrongVersion {
            found: "1.1".into()
        }
    );
    assert_eq!(
        InstallVerification::from_link(&LinkStatus::NotRunning, "1.2"),
        InstallVerification::NotConnected
    );
}

#[test]
fn the_reaper_folder_argument_is_read_in_both_spellings() {
    let arguments = |list: &[&str]| list.iter().map(|text| text.to_string()).collect::<Vec<_>>();
    assert_eq!(
        reaper_folder_argument(arguments(&["app", "--reaper-folder", "/Music/REAPER 2"])),
        Some(PathBuf::from("/Music/REAPER 2"))
    );
    assert_eq!(
        reaper_folder_argument(arguments(&["--reaper-folder=/a"])),
        Some(PathBuf::from("/a"))
    );
    assert_eq!(
        reaper_folder_argument(arguments(&["--reaper-folder"])),
        None
    );
    assert_eq!(reaper_folder_argument(arguments(&["app"])), None);
}

#[test]
fn program_headers_are_recognised() {
    let mut pe = vec![0u8; 0x100];
    pe[0] = b'M';
    pe[1] = b'Z';
    pe[0x3c] = 0x80;
    pe[0x80..0x84].copy_from_slice(b"PE\0\0");
    pe[0x84..0x86].copy_from_slice(&0x8664u16.to_le_bytes());
    assert_eq!(
        BinaryArchitecture::from_header(&pe),
        Some(BinaryArchitecture::X64)
    );
    pe[0x84..0x86].copy_from_slice(&0x14cu16.to_le_bytes());
    assert_eq!(
        BinaryArchitecture::from_header(&pe),
        Some(BinaryArchitecture::Other)
    );

    // A universal Mach-O with an Intel and an Arm64 part counts as Arm64.
    let mut universal = vec![0xca, 0xfe, 0xba, 0xbe, 0, 0, 0, 2];
    universal.extend_from_slice(&0x0100_0007u32.to_be_bytes());
    universal.extend_from_slice(&[0; 16]);
    universal.extend_from_slice(&0x0100_000cu32.to_be_bytes());
    universal.extend_from_slice(&[0; 16]);
    assert_eq!(
        BinaryArchitecture::from_header(&universal),
        Some(BinaryArchitecture::Arm64)
    );
    assert_eq!(BinaryArchitecture::from_header(b"#!/bin/sh"), None);
    assert_eq!(BinaryArchitecture::from_header(&[]), None);
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn on_macos_the_copy_is_signed_and_not_quarantined() -> TestResult {
    use std::process::Command;
    // A real arm64 library: the extension this workspace built, or this test's own binary is
    // not loadable, so a tiny one is built with the system compiler.
    let root = TempFolder::new()?;
    let source = root.path().join("empty.c");
    fs::write(&source, "int rc2_probe(void) { return 1; }")?;
    let library = root.path().join("bundled.dylib");
    let built = Command::new("cc")
        .args(["-dynamiclib", "-arch", "arm64", "-o"])
        .arg(&library)
        .arg(&source)
        .status()?;
    assert!(built.success());
    let resource = root.path().join("resource");
    fs::create_dir_all(&resource)?;
    let installer = ExtensionInstaller::new(
        PLATFORM,
        library,
        Arc::new(FakeProcessCheck::default()),
        Box::new(app::MacExtensionPreparer),
    );
    let install = ReaperInstall {
        resource_folder: resource,
        executable: None,
    };
    installer.install(&install)?;
    let target = installer.installed_path(&install);
    assert!(
        Command::new("codesign")
            .arg("-v")
            .arg(&target)
            .status()?
            .success()
    );
    let quarantine = Command::new("xattr")
        .args(["-p", "com.apple.quarantine"])
        .arg(&target)
        .output()?;
    assert!(!quarantine.status.success());
    Ok(())
}

#[test]
fn the_endpoint_file_follows_the_isolated_directory_then_the_chosen_reaper_then_the_default() {
    use app::endpoint_file_from;
    let some = |text: &str| Some(PathBuf::from(text));
    assert_eq!(
        endpoint_file_from(some("/iso"), some("/portable"), some("/default")),
        Some(PathBuf::from("/iso").join("endpoint.json"))
    );
    assert_eq!(
        endpoint_file_from(None, some("/portable"), some("/default")),
        Some(PathBuf::from("/portable/RC2").join("endpoint.json"))
    );
    assert_eq!(
        endpoint_file_from(None, None, some("/default")),
        Some(PathBuf::from("/default/RC2").join("endpoint.json"))
    );
    assert_eq!(endpoint_file_from(None, None, None), None);
}

/// Changes the copy's bytes, as signing does on macOS.
struct AppendingPreparer;

impl ExtensionPreparer for AppendingPreparer {
    fn prepare(&self, library: &Path) -> Result<(), String> {
        let mut bytes = fs::read(library).map_err(|error| error.to_string())?;
        bytes.extend_from_slice(b" signed");
        fs::write(library, bytes).map_err(|error| error.to_string())
    }
}

#[test]
fn a_copy_whose_bytes_were_changed_by_preparing_still_counts_as_current_until_the_bundle_changes()
-> TestResult {
    let fixture = fixture_with(&MACH_O_ARM64, Box::new(AppendingPreparer))?;
    fixture.installer.install(&fixture.install)?;
    assert!(fixture.installer.inspect(&fixture.install).is_healthy());
    assert!(fixture.installer.plan(&fixture.install)?.is_empty());

    let newer = fixture.root.path().join("bundled.dylib");
    fs::write(&newer, b"extension version three")?;
    let report = fixture.installer.inspect(&fixture.install);
    assert_eq!(
        report.status_of(InstallItemKind::Extension),
        Some(InstallStatus::Fixable)
    );
    Ok(())
}
