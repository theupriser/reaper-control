<script lang="ts">
  import { markerReading } from "../lib/ipc";
  import { readingLines } from "../lib/marker-reading";
  import { strings } from "../lib/strings";

  const text = strings.help.helper;
  let name = $state("");
  let lines = $state<string[]>([]);
  let failed = $state(false);

  const read = async (value: string) => {
    name = value;
    try {
      lines = readingLines(await markerReading(value));
      failed = false;
    } catch {
      failed = true;
    }
  };
</script>

<section id="try-it" class="card">
  <h2>{text.title}</h2>
  <p class="intro">{text.intro}</p>
  <input
    type="text"
    aria-label={text.label}
    placeholder={text.placeholder}
    value={name}
    oninput={(event) => read(event.currentTarget.value)}
  />
  <ul aria-live="polite">
    {#if failed}
      <li>{text.failed}</li>
    {:else if name.trim() === ""}
      <li class="quiet">{text.empty}</li>
    {:else if lines.length === 0}
      <li class="quiet">{text.plain}</li>
    {:else}
      {#each lines as line (line)}<li>{line}</li>{/each}
    {/if}
  </ul>
</section>

<style>
  .card { display: flex; flex-direction: column; gap: 10px; padding: 22px 24px; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; scroll-margin-top: var(--space-4); }
  h2 { margin: 0; font-size: var(--text-title); font-weight: 800; }
  .intro, ul { margin: 0; font-size: var(--text-body); line-height: 1.6; color: var(--text-soft); }
  ul { padding-left: 20px; display: flex; flex-direction: column; gap: var(--space-2); }
  .quiet { color: var(--muted); }
  input { min-height: var(--control-height); padding: 0 var(--space-3); border: 1px solid var(--line); border-radius: var(--radius); background: var(--bg); color: var(--text); font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: var(--text-body); }
</style>
