//! Where the extension file that ships with the app is.

use std::path::PathBuf;

use crate::install_platform::InstallPlatform;

/// The bundled extension for `platform`: `RC2_EXTENSION_FILE` when set, else next to the app's
/// executable, in a macOS bundle's `Resources`, or in the repository's `dist` folder (development,
/// filled by `scripts/package_extension.sh`). The first that exists wins; when none does, the
/// executable's folder is returned so the error names a sensible place.
#[must_use]
pub fn bundled_extension(platform: InstallPlatform) -> PathBuf {
    let name = platform.library_file_name();
    if let Some(file) = std::env::var_os("RC2_EXTENSION_FILE") {
        return PathBuf::from(file);
    }
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf))
        .unwrap_or_default();
    let candidates = [
        beside.join(name),
        beside.join("../Resources").join(name),
        beside.join("../../../../dist").join(name),
        PathBuf::from("dist").join(name),
        PathBuf::from("../../dist").join(name),
    ];
    candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .unwrap_or_else(|| beside.join(name))
}
