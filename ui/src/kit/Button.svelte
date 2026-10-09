<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    kind = "normal",
    disabled = false,
    label,
    onclick,
    children,
  }: {
    kind?: "normal" | "primary" | "danger";
    disabled?: boolean;
    label?: string;
    onclick?: () => void;
    children: Snippet;
  } = $props();
</script>

<button class={kind} {disabled} aria-label={label} {onclick}>{@render children()}</button>

<style>
  button {
    height: var(--control-height);
    padding: 0 var(--control-padding);
    border-radius: var(--radius);
    border: 1px solid var(--line);
    background: var(--raised);
    color: var(--text);
    font-size: var(--text-body);
    font-weight: 700;
    cursor: pointer;
    transition: transform var(--motion-fast);
  }
  button:active:not(:disabled) { transform: scale(0.98); }
  .primary { background: var(--green); color: var(--on-accent); border-color: var(--green); }
  .danger { background: var(--tint-danger); color: var(--danger); border-color: var(--danger); }
  button:disabled { opacity: 0.4; cursor: default; }
</style>
