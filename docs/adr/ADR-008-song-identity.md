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

## Proposed decision
1. A Song's identity is the region GUID. Region numbers and enumeration indexes are never stored in a setlist. Entries in D2 storage hold `{guid, name, start}` (name and start only as fallback).
2. A setlist entry is resolved by GUID first. If no GUID matches (deleted region, or a project without stored GUIDs), fall back to exact name, then to start time within 1 ms; an entry that still does not resolve is shown as broken and offered for repair (R13), never silently dropped or guessed.
3. Saving a setlist into a project that REAPER has not saved since loading GUID-less markers must also make the GUIDs permanent: the extension asks for a project save before it writes the setlist (or warns that the project must be saved). Otherwise the stored GUIDs are lost on the next load.
4. Cues and directives are matched by GUID the same way; a marker and a region can share a number, so never key on the number alone.

## Not tested
- Edits made by hand in the REAPER UI (Region/Marker Manager, copy and paste of regions, ripple edit, undo and redo): only API edits were run. Undo may resurrect or change GUIDs.
- Duplicate GUIDs (a project pasted into another project, template use).
- Deriving a stable `ProjectId` and the v1 setlist import mapping (rest of WP 1.6 and WP 3.x).
- Windows.
