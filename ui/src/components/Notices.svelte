<script lang="ts">
  import { strings } from "../lib/strings";
  import type { Notice } from "../lib/generated/protocol";

  let { notices, onDismiss }: { notices: Notice[]; onDismiss: (key: string) => void } = $props();
</script>

{#if notices.length > 0}
  <div class="stack">
    {#each notices as notice (notice.key)}
      <div class="notice" role="status">
        <span class="dot {notice.level.toLowerCase()}"></span>
        <div class="words">
          <div class="title">{notice.title}</div>
          <div class="text">{notice.text}</div>
        </div>
        <button aria-label={strings.notices.dismiss} onclick={() => onDismiss(notice.key)}>
          <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M6 6l12 12M18 6L6 18" /></svg>
        </button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .stack {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: min(360px, calc(100vw - 32px));
  }
  .notice {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 14px;
    border: 1px solid var(--line);
    border-radius: 12px;
    background: var(--raised);
    box-shadow: var(--shadow-raised);
  }
  .dot {
    width: 10px;
    height: 10px;
    margin-top: 6px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .info {
    background: var(--info);
  }
  .warning {
    background: var(--amber);
  }
  .error {
    background: var(--red);
  }
  .words {
    flex: 1;
    min-width: 0;
  }
  .title {
    font-size: 15px;
    font-weight: 700;
  }
  .text {
    margin-top: 2px;
    font-size: 13px;
    line-height: 1.4;
    color: var(--muted);
  }
  button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border: none;
    border-radius: 8px;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    flex-shrink: 0;
  }
</style>
