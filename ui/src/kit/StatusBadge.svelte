<script lang="ts">
  let { tone, label }: { tone: "ok" | "warn" | "error" | "info" | "idle"; label: string } = $props();
</script>

<span class="badge {tone}"><span class="dot"></span>{label}</span>

<style>
  .badge { display: inline-flex; align-items: center; gap: var(--space-2); font-size: var(--text-small); font-weight: 700; color: var(--text-soft); }
  .dot { width: 10px; height: 10px; border-radius: 50%; background: var(--muted); }
  .ok .dot { background: var(--green); }
  .warn .dot { background: var(--amber); }
  .error .dot { background: var(--red); }
  .info .dot { background: var(--info); }
</style>
