/// Windows calls this when the DLL loads and unloads. It replaces the one that
/// `#[reaper_extension_plugin]` generates: that one runs `reaper-low`'s destroy hooks, which
/// panic at process exit (the thread-locals are gone), and the abort kills the process before
/// the marker is removed. This extension registers no destroy hooks, so only the marker is left
/// to clean up, with a plain operating system call.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn DllMain(
    hinstance: reaper_low::raw::HINSTANCE,
    reason: u32,
    _: *const u8,
) -> u32 {
    if reason == reaper_low::raw::DLL_PROCESS_ATTACH {
        let _ = reaper_low::register_hinstance(hinstance);
    } else if reason == reaper_low::raw::DLL_PROCESS_DETACH
        && let Some(file) = super::EXIT_FILE.get()
    {
        file.remove();
    }
    1
}
