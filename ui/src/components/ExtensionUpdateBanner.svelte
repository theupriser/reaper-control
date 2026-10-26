<script lang="ts">
  import Button from "../kit/Button.svelte";
  import { strings } from "../lib/strings";

  let { busy, error, onReview, onLater }: { busy: boolean; error: string | null; onReview: () => void; onLater: () => void } = $props();
  const text = strings.extensionUpdate;
</script>

<div class="banner" role="status">
  <div>
    <div class="title">{text.title}</div>
    <div class="detail">{busy ? text.working : text.text}</div>
    {#if error}<div class="error" role="alert">{error}</div>{/if}
  </div>
  <div class="buttons">
    <Button disabled={busy} onclick={onLater}>{text.laterButton}</Button>
    <Button kind="primary" disabled={busy} onclick={onReview}>{text.review}</Button>
  </div>
</div>

<style>
  .banner { display: flex; align-items: center; justify-content: space-between; gap: 20px; margin: var(--space-3) var(--space-4) 0; padding: 14px 20px; border: 1px solid color-mix(in srgb, var(--amber) 35%, transparent); border-radius: 16px; background: color-mix(in srgb, var(--amber) 10%, transparent); }
  .title { font-size: var(--text-title); font-weight: 800; color: var(--amber); }
  .detail { margin-top: 2px; font-size: 14px; color: var(--text-soft); }
  .error { margin-top: 4px; font-size: 14px; color: var(--red); }
  .buttons { display: flex; gap: var(--space-3); }
</style>
