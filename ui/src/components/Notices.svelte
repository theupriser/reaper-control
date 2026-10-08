<script lang="ts">
  import type { Notice } from "../lib/generated/protocol";

  let { notices }: { notices: Notice[] } = $props();
</script>

{#if notices.length > 0}
  <ul aria-live="polite">
    {#each notices as notice (notice.key)}
      <li class={notice.level.toLowerCase()}>{notice.text}</li>
    {/each}
  </ul>
{/if}

<style>
  ul {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    padding: 10px 14px;
    border: 1px solid var(--line);
    border-left-width: 4px;
    border-radius: 6px;
    background: var(--panel);
    font-weight: 600;
  }
  .info {
    border-left-color: var(--green);
  }
  .warning {
    border-left-color: var(--amber);
  }
  .error {
    border-left-color: var(--red);
  }
</style>
