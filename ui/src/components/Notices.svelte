<script lang="ts">
  import { strings } from "../lib/strings";
  import { noticeTone } from "../lib/notices";
  import Toast from "../kit/Toast.svelte";
  import type { Notice } from "../lib/generated/protocol";

  let { notices, onDismiss }: { notices: Notice[]; onDismiss: (key: string) => void } = $props();
</script>

{#if notices.length > 0}
  <div class="stack">
    {#each notices as notice (notice.key)}
      <Toast tone={noticeTone(notice.level)} title={notice.title} text={notice.text} dismissLabel={strings.notices.dismiss} onDismiss={() => onDismiss(notice.key)} />
    {/each}
  </div>
{/if}

<style>
  .stack { position: fixed; right: var(--space-4); bottom: var(--space-4); z-index: 10; display: flex; flex-direction: column; gap: var(--space-2); width: min(360px, calc(100vw - 2 * var(--space-4))); }
</style>
