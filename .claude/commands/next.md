---
description: After finishing something: sync, update STATUS.md, then propose the next work package
---
We just finished a unit of work. Do this:

1. `git checkout main && git pull`, delete merged local branches, run `gh pr list` and show me anything still open.
2. Update docs/STATUS.md (Done / In progress / Next / Gate 1 checklist / Last updated). Do it on the branch of the next piece of work, never directly on main, and include it in that PR.
3. Name the next work package from STATUS.md "Next" (WP id, goal, how you will test it visibly) and ask me to confirm before you branch and start.

Same rules as /start: feature/bugfix branches, my review before any merge, real terminal output, cheap window-only screenshots, no AI attribution, and remind me to /compact when something is complete.
