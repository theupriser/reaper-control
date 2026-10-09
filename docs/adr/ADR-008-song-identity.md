# ADR-008: A Song is identified by its region GUID, with a fallback for projects that have none

Status: **Proposed**. Spike S6 (WP 1.6), code in `spikes/s6-identity`. Date: 2026-10-07. Related: D2, ADR-002 finding 5, risk R13.

## Question (S6)
Setlists live in the project (D2) and refer to Songs. What can they refer to so the reference survives renaming, moving, inserting, deleting, saving and reloading in REAPER?

## What was run
REAPER 7.82, macOS arm64, isolated test instance, a copy of a small sample project with three regions and two markers (`.dev/s6/`, not committed). The spike reads each region and marker with `EnumProjectMarkers3` and its GUID with `GetSetProjectInfo_String("MARKER_GUID:<index>")` (the raw `reaper-low` call; `reaper-medium` has only the setter). A command file drives edits from the extension's main-thread tick through `SetProjectMarker4`, `AddProjectMarker2`, `DeleteProjectMarker`, action 40026 (save) and `Main_openProject`.

## Results (measured)
| Step | Result |
|---|---|
| Read GUIDs through the API | Works; identical to the `{GUID}` in the project file |
| Rename a region | GUID unchanged |
| Move a region (new start and end), also past other regions | GUID unchanged; the enumeration index changes |
| Insert a new region at the start | New region gets a new random GUID; all other GUIDs unchanged, their enumeration indexes shift by one |
| Delete a region and a marker | Remaining GUIDs unchanged |
| Save, then reload the project | All GUIDs unchanged (and written to the file) |
| Region number (`id`) of a new region | Collided with an existing marker's number (both `id=3`). Numbers are not unique across regions and markers |
| Project file without GUIDs (written by an old REAPER, `MARKER n pos "name" flags`) | REAPER invents a GUID on load, and **a different one on every load** until the project is saved |

## Second round (2026-10-07): undo, hand-edit action, ProjectId, copies
Same setup; the spike gained `ublock` (an edit inside one undo block), `action`, `undo`/`redo` (actions 40029/40030), `tsel`, and project ExtState get/set. Log: `.dev/reaper-test/RC2/spike-s6.log` (not committed).
| Step | Result |
|---|---|
| Delete a region in an undo block, undo, redo | Undo brings the region back with the **same GUID** (`{359BDBB5-…}`), redo removes it again |
| Move and rename a region, undo | Position and name back, GUID unchanged throughout |
| Insert a region, undo, redo | Undo removes it, redo brings it back with the **same GUID** (`{98E75CF2-…}`), not a new one |
| Region insert through a REAPER action (40306, the same code path as the UI), undo | New region got a new GUID, existing GUIDs unchanged, undo removed it. The action opens a modal "Edit Region" dialog and blocks the tick until it is closed (the extension must never run such actions) |
| Project ExtState (`RC2S6`) set, save, reload | Value survives (`PID proj-1234` in an `<EXTSTATE>` block of the file) |
| Copy the saved `.rpp`, open the copy | ExtState value **and all region GUIDs identical** to the original |

Not driven: the Region/Marker Manager and copy and paste of regions in the UI (no way to automate them; the action above is the same API path). Duplicate GUIDs inside one project were not produced.

v1 reference (read-only): v1 setlist items hold `regionId` (the region number) and `name`; a setlist holds a `projectId`, a random `project-<time>-<random>` string that v1 keeps in the project ExtState (`reaperConnector.ts`, `ProjectId`). v1 never stored GUIDs.

## Proposed decision
1. A Song's identity is the region GUID. Region numbers and enumeration indexes are never stored in a setlist. Entries in D2 storage hold `{guid, name, start}` (name and start only as fallback).
2. A setlist entry is resolved by GUID first. If no GUID matches (deleted region, or a project without stored GUIDs), fall back to exact name, then to start time within 1 ms; an entry that still does not resolve is shown as broken and offered for repair (R13), never silently dropped or guessed.
3. Saving a setlist into a project that REAPER has not saved since loading GUID-less markers must also make the GUIDs permanent: the extension asks for a project save before it writes the setlist (or warns that the project must be saved). Otherwise the stored GUIDs are lost on the next load.
4. Cues and directives are matched by GUID the same way; a marker and a region can share a number, so never key on the number alone.

5. Undo and redo are safe: they restore the same GUID, so a setlist never breaks because of undo.
6. `ProjectId` is a random id kept in the project ExtState, as v1 does. It is only the key for the app's restore-only mirror (D2); the setlist itself lives in the project. A copied project file carries the same id and GUIDs, so the extension also records the project path next to the id; when the path differs it treats the project as a copy and offers a new id.
7. v1 import: for every v1 item find the region whose number equals `regionId` among **regions only** (numbers collide with markers), check that its name equals the v1 name, store the GUID; on a mismatch fall back to name, then show the entry as broken for repair. v1's own `ProjectId` from the project ExtState is matched to the v1 setlist's `projectId`. Not run against a real v1 setlist file yet.

## Seen in WP 3.9 (REAPER 7.82, macOS)
Two project tabs each keep their own `project_id` when the user switches between them (the id is read from the current tab's ExtState), and the extension notices a switch by the tab pointer, because two projects can have the same change count. Copies (step 6): the extension stores the saved project path in ExtState `RC2`/`project_path` next to the id; when it differs from the current path the project gets a new id (measured live 2026-10-09: a copied file opened in REAPER 7.82 got its own id and path, the original kept its own). A file that was only moved or renamed looks the same and also gets a new id, which orphans its mirror (acceptable: the mirror is restore-only). An unsaved project has no path and keeps its id.

## Not tested
- Region/Marker Manager, copy and paste of regions and ripple edit in the UI (only the API and one action were run).
- Duplicate GUIDs inside one project (a project pasted into another, template use).
- The v1 import mapping against a real v1 setlist file, and the copy-detection path check (WP 3.x).
- Windows.
