# Domain: Installation

**Status: draft for the owner's review (WP 0.8), 2026-10-09.** Source: SPEC §13, ADR-006, `app::ExtensionInstaller`.

## Timeline of events
`InstallInspected` (an `InstallReport`) → plan (dry run) → apply: keep previous, copy, prepare → `InstallVerified` (the handshake answers with the expected version) → later: repair, update on request with REAPER closed, uninstall (optionally restoring the previous file).

## `ExtensionInstaller` rules
| # | Invariant | Test (`crates/app/tests/installer.rs`) |
|---|---|---|
| I1 | A clean install copies the extension and the report turns healthy | `a_clean_install_copies_the_extension_and_reports_it_healthy` |
| I2 | Installing twice changes nothing the second time | `installing_twice_changes_nothing_the_second_time` |
| I3 | An older extension is kept as a backup and can be restored | `an_existing_older_extension_is_kept_as_a_backup_and_can_be_restored` |
| I4 | A dry run changes nothing | `a_dry_run_changes_nothing` |
| I5 | A REAPER of the wrong architecture is refused with guidance | `an_intel_reaper_is_refused_with_guidance`, `program_headers_are_recognised` |
| I6 | A running REAPER blocks an update and an uninstall, not a first install | `a_running_reaper_blocks_an_update_but_not_a_first_install` |
| I7 | A read-only or missing folder is reported for the user to fix | `a_read_only_folder_is_reported_for_the_user_to_fix`, `a_missing_resource_folder_is_reported_as_manual` |
| I8 | A failed preparation puts the old state back | `a_failed_preparation_puts_the_old_state_back` |
| I9 | Repair restores a removed extension | `repair_restores_a_removed_extension` |
| I10 | Uninstalling what is not installed is an error | `uninstalling_what_is_not_installed_is_an_error` |
| I11 | A missing bundle is reported | `a_missing_bundle_is_reported` |
| I12 | Only the extension answering with the right version confirms the install | `only_the_extension_answering_with_the_right_version_confirms_the_install` |
| I13 | A portable REAPER is found from the chosen folder (`--reaper-folder`) | `a_portable_install_is_found_from_the_chosen_folder`, `the_reaper_folder_argument_is_read_in_both_spellings` |
| I14 | On macOS the copy is signed and not quarantined | `on_macos_the_copy_is_signed_and_not_quarantined` |

## Open for the owner
- The wizard screens (WP 6.x) still have to be built on this module; the words on the guidance cards are the `advice` texts in `extension_installer.rs`, in English only for now.
