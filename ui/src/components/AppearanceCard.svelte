<script lang="ts">
  import { strings } from "../lib/strings";
  import Panel from "../kit/Panel.svelte";
  import { densities, themes, touchSizes } from "../lib/appearance";

  let { theme = $bindable(), density = $bindable(), touch = $bindable() }: { theme: string; density: string; touch: string } = $props();
  const text = strings.settings.appearance;
</script>

<Panel title={text.title} hint={text.hint}>
  <div class="row">
    <label>{text.theme}
      <select bind:value={theme}>{#each themes as option (option)}<option value={option}>{text.themes[option]}</option>{/each}</select>
    </label>
    <label>{text.density}
      <select bind:value={density}>{#each densities as option (option)}<option value={option}>{text.densities[option]}</option>{/each}</select>
    </label>
    <label>{text.touch}
      <select bind:value={touch}>{#each touchSizes as option (option)}<option value={option}>{text.touchSizes[option]}</option>{/each}</select>
    </label>
  </div>
</Panel>

<style>
  .row { display: flex; gap: 14px; flex-wrap: wrap; }
  label { display: flex; flex-direction: column; gap: 6px; font-size: 13px; font-weight: 600; color: var(--muted); width: 180px; }
  select { height: 44px; border-radius: 10px; border: 1px solid var(--line); background: var(--bg); color: var(--text); padding: 0 14px; font-size: 15px; font-family: inherit; }
</style>
