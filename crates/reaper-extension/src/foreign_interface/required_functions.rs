/// Every REAPER function the extension calls. `reaper-rs` panics when it calls a function the
/// running REAPER does not have, so all of them are looked up once at load (ADR-009). Add a name
/// here whenever the adapter starts to use another function.
pub(super) const REQUIRED_FUNCTIONS: &[&str] = &[
    "GetAppVersion",
    "GetResourcePath",
    "ShowConsoleMsg",
    "plugin_register",
    "GetPlayStateEx",
    "GetPlayPosition2Ex",
    "GetCursorPositionEx",
    "SetEditCurPos2",
    "OnPlayButtonEx",
    "OnPauseButtonEx",
    "get_config_var",
    "EnumProjectMarkers3",
    "GetSetProjectInfo_String",
    "CountTempoTimeSigMarkers",
    "GetTempoTimeSigMarker",
    "TimeMap_GetMeasureInfo",
    "GetProjExtState",
    "SetProjExtState",
    "MarkProjectDirty",
    "GetProjectStateChangeCount",
];
