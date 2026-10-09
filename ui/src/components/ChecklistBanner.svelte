<script lang="ts">
  import Button from "../kit/Button.svelte";
  import { strings } from "../lib/strings";

  let { ready, passed, failed, onPerform }: { ready: boolean; passed: number; failed: number; onPerform: () => void } = $props();
  const text = strings.checklist;
</script>

<div class="banner" class:ready role="status">
  <div class="message">
    <span class="mark" aria-hidden="true">{ready ? "✓" : "!"}</span>
    <div>
      <div class="title">{ready ? text.readyTitle : text.attentionTitle}</div>
      <div class="counts">{text.counts(passed, failed)}</div>
    </div>
  </div>
  <Button kind={ready ? "primary" : "normal"} onclick={onPerform}>{ready ? text.perform : text.performAnyway}</Button>
</div>

<style>
  .banner { --tone: var(--amber); display: flex; align-items: center; justify-content: space-between; gap: 20px; padding: 18px 22px; border: 1px solid color-mix(in srgb, var(--tone) 35%, transparent); border-radius: 16px; background: color-mix(in srgb, var(--tone) 10%, transparent); }
  .ready { --tone: var(--green); }
  .message { display: flex; align-items: center; gap: 14px; }
  .mark { display: grid; place-items: center; width: 30px; height: 30px; border-radius: 50%; background: var(--tone); color: var(--on-accent); font-weight: 900; }
  .title { font-size: var(--text-title); font-weight: 800; color: var(--tone); }
  .counts { margin-top: 2px; font-size: 14px; color: var(--text-soft); }
</style>
