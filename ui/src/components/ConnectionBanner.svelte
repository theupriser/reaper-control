<script lang="ts">
  import type { HealthBanner } from "../lib/health";

  let { banner }: { banner: HealthBanner } = $props();
</script>

<div class="banner" role="alert">
  <strong>{banner.title}</strong>
  <span>{banner.text}</span>
</div>

<style>
  .banner { display: flex; flex-wrap: wrap; gap: var(--space-2) var(--space-3); align-items: baseline; padding: var(--space-3) var(--space-5); background: var(--red); color: var(--on-accent); font-size: var(--text-small); }
  strong { font-weight: 800; }
</style>
