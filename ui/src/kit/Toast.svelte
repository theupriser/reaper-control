<script lang="ts">
  let {
    tone,
    title,
    text,
    dismissLabel,
    actionLabel,
    onAction,
    onDismiss,
  }: {
    tone: "info" | "warn" | "error";
    title: string;
    text?: string;
    dismissLabel: string;
    actionLabel?: string;
    onAction?: () => void;
    onDismiss: () => void;
  } = $props();
</script>

<div class="toast" role="status">
  <span class="dot {tone}"></span>
  <div class="words">
    <div class="title">{title}</div>
    {#if text}<div class="text">{text}</div>{/if}
    {#if actionLabel && onAction}<button class="action" onclick={onAction}>{actionLabel}</button>{/if}
  </div>
  <button class="close" aria-label={dismissLabel} onclick={onDismiss}>
    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M6 6l12 12M18 6L6 18" /></svg>
  </button>
</div>

<style>
  .toast { display: flex; align-items: flex-start; gap: var(--space-3); padding: var(--space-3) var(--space-4); border: 1px solid var(--line); border-radius: calc(var(--radius) * 1.2); background: var(--raised); box-shadow: var(--shadow-raised); }
  .dot { width: 10px; height: 10px; margin-top: var(--space-2); border-radius: 50%; flex-shrink: 0; }
  .info { background: var(--info); }
  .warn { background: var(--amber); }
  .error { background: var(--red); }
  .words { flex: 1; min-width: 0; }
  .title { font-size: var(--text-body); font-weight: 700; }
  .text { margin-top: var(--space-1); font-size: var(--text-small); line-height: 1.4; color: var(--muted); }
  .action { margin-top: var(--space-2); padding: 0; border: none; background: none; color: var(--info); font: inherit; font-size: var(--text-small); font-weight: 700; cursor: pointer; }
  .close { display: flex; align-items: center; justify-content: center; width: var(--control-height); height: var(--control-height); margin: calc(var(--space-2) * -1) calc(var(--space-2) * -1) 0 0; border: none; border-radius: var(--radius); background: transparent; color: var(--muted); cursor: pointer; flex-shrink: 0; }
</style>
