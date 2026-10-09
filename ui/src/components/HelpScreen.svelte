<script lang="ts">
  import HelpSection from "./HelpSection.svelte";
  import { strings } from "../lib/strings";

  const text = strings.help;
  const jump = (id: string) => document.getElementById(id)?.scrollIntoView({ block: "start" });
</script>

<section class="screen">
  <nav aria-label={text.topics}>
    <h1>{text.title}</h1>
    {#each text.sections as section (section.id)}
      <button onclick={() => jump(section.id)}>{section.title}</button>
    {/each}
  </nav>
  <div class="body">
    <p class="intro">{text.intro}</p>
    {#each text.sections as section (section.id)}
      <HelpSection {...section} />
    {/each}
  </div>
</section>

<style>
  .screen { display: flex; gap: 28px; padding: 28px 32px; }
  nav { display: flex; flex-direction: column; gap: 2px; flex: none; width: 190px; align-self: flex-start; position: sticky; top: 28px; }
  h1 { margin: 0 0 14px; font-size: 28px; font-weight: 800; }
  button { display: flex; align-items: center; min-height: var(--control-height); padding: 10px var(--space-3); border: none; border-radius: var(--radius); background: transparent; color: var(--text-soft); font: inherit; font-size: var(--text-body); font-weight: 600; text-align: left; cursor: pointer; }
  button:hover { background: var(--raised); }
  .body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: var(--space-4); padding-top: 4px; }
  .intro { margin: 0; color: var(--muted); }
</style>
