# Design source (fallback copy)

Source files of the Claude Design canvas for Reaper Control v2, kept here so the designs survive independently of the hosted canvas.

- Hosted canvas (private to the owner): https://claude.ai/artifact/X3zo5xvtRT58fPWjRPtXsH
- `canvas/canvas.json` is the canvas index (board positions, sizes, titles, notes).
- `canvas/*.dc.html` is one file per artboard (Design Component format: HTML with `{{holes}}`, `<sc-for>`, `<sc-if>` and a small logic class). They need the Design canvas runtime to render; as plain files they are still readable as markup and carry every colour, size and text.
- `canvas/Main.dc.html` is the Performer screen (keeps the v1 look). `Sidebar.dc.html` is the shared sidebar component.

Status and open design work: `docs/SPEC.md` §10b–10e (the wizard, settings connection card, connection states and pre-show check still show the old script/web-interface wording and must be updated for the Rust extension, decision D7).

Design tokens used by all non-Performer screens: ground `#0E0F11`, sidebar `#121417`, surface `#17191C`, raised `#1E2125`, border `#2A2E34`, text `#F2F3F4`, muted `#9AA1A9` / `#8B929A`, accent green `#4CAF50`, amber `#FFC107`, red `#FF5252`, blue `#5AA9F0`; system font stack, monospace for times; 44 px minimum touch targets. Performer uses the v1 values (`#121212`, `#4CAF50`, `#FFC107`, `#ff5252`, `#333`).
