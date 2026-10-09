<script lang="ts">
  import HelpSection from "./HelpSection.svelte";
  import { strings } from "../lib/strings";

  const text = strings.help;
</script>

<section class="screen">
  <h1>{text.title}</h1>
  <p class="intro">{text.intro}</p>
  {#each text.sections as section}
    <HelpSection {...section} />
  {/each}
</section>

<style>
  .screen { padding: 28px 32px; display: flex; flex-direction: column; gap: 16px; max-width: 760px; }
  h1 { margin: 0; font-size: 28px; font-weight: 800; }
  .intro { margin: 0; color: var(--muted); }
</style>
