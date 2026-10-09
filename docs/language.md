# Language (glossary)

**Status: draft for the owner's review (WP 0.8), written 2026-10-09 from SPEC §14.2 and the code.** The words below are binding for code, UI copy and docs. The vocabulary test (`crates/architecture-tests/tests/vocabulary.rs`) rejects the banned words in code and copy.

| Term | Meaning | Not to be called | Where it lives in code |
|---|---|---|---|
| **Song** | A region in the REAPER project that can be performed | track, item | `catalogue::Song`, `protocol::SongInfo` |
| **Setlist** | Ordered list of entries that refer to songs; belongs to a project | playlist (that is a *mode*) | `setlists::Setlist` |
| **Entry** | One position in a setlist (a song reference) | item | `setlists::Entry`, `protocol::EntryInfo` |
| **Performance** | A running pass through a setlist | session | `performance::Performance` |
| **Phase** | Where a performance is: `Idle, Playing, Paused, CountingIn, HardStopped, HandingOver, Finished` | state, mode | `performance::Phase` |
| **Cue** | A marker the performer can jump to | marker (when meant for navigation) | `catalogue::Cue` |
| **Directive** | A special marker instruction: `!1008` hard stop, `!length:N`, `!bpm:N` | command | `catalogue::Directive` |
| **Hard stop** | A directive: the performance halts at the song end (or `!length`) until the performer resumes | pause | `performance::Phase::HardStopped` |
| **Hand-over** | The moment control passes from one song to the next | transition (in UI copy) | `performance::HandOverPolicy` |
| **Count-in** | Two bars of click before a song starts | | `performance::Phase::CountingIn` |
| **Intent** | What a person wants, from the UI, a key or a MIDI note, before it is checked | command | `app::Intent` |
| **Command** | A checked instruction on the bus | | `protocol::Command` |
| **Link** | The connection between the app and REAPER | | `app::LinkConnection` (app side), `link::LinkServer` (extension side), `protocol` (wire) |
| **Extension** | The companion extension loaded by REAPER (`reaper_rc2-*`) | script, plugin | `reaper-extension` |
| **Catalog** | The songs, cues and setlists of the project, as the extension reports them | | `protocol::Catalog` |
| **Installation** | Getting the extension into a REAPER and keeping it healthy | setup | `app::ExtensionInstaller` |
| **Region number** | The number REAPER shows for a region; v1 setlists stored it | region id | `reaper_port::Region::number` |

## Words with two meanings that must stay apart
- *Command* (checked, on the bus) versus *Intent* (wanted, not yet checked) versus *Directive* (an instruction written in a marker).
- *Marker* is REAPER's word for any marker; the app says *Cue* when the performer can jump to it, *Directive* when it carries an instruction.
- *Pause* is the transport pause; a *Hard stop* is a planned halt at the end of a song.

## Open for the owner
- Is **Entry** the right word for the UI, or does the Setlists screen say "song"? (SPEC keeps Entry in code; the UI copy follows v1 where it exists.)
- Should **Catalog** be shown to users at all, or stays an implementation word?
