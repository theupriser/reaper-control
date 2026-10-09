<script lang="ts">
  let { text, label }: { text: string; label?: string } = $props();
</script>

<span class="clock" aria-label={label}>{text}</span>

<style>
  .clock { font-variant-numeric: tabular-nums; font-size: var(--text-body); color: var(--text); white-space: nowrap; }
</style>
