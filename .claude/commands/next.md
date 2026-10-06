---
description: After finishing something: sync, then propose and start the next work package
---
We just finished a unit of work. Do this:

1. `git checkout main && git pull`, delete merged local branches, run `gh pr list` and show me anything still open.
2. Update the "Status and next steps" section of AGENTS.md if it is out of date (on the branch of the next piece of work, never directly on main).
3. Name the next PLAN.md work package (WP id, goal, how you will test it visibly) and ask me to confirm before you branch and start.

Same rules as /start: feature/bugfix branches, my review before any merge, real terminal output, cheap window-only screenshots, no AI attribution, and remind me to /compact when something is complete.
