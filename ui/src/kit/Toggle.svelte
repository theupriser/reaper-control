<script lang="ts">
  let { on, label, onchange }: { on: boolean; label: string; onchange: () => void } = $props();
</script>

<button class="toggle" class:on aria-pressed={on} onclick={onchange}>{label}</button>

<style>
  .toggle {
    width: 100%;
    min-height: var(--control-height);
    padding: var(--space-2) var(--space-4);
    border: none;
    border-radius: var(--radius);
    font: inherit;
    font-size: var(--text-title);
    font-weight: bold;
    text-align: center;
    cursor: pointer;
    background: var(--tint-danger);
    color: var(--danger);
    transition: transform var(--motion-fast);
  }
  .toggle:active { transform: scale(0.98); }
  .on { background: var(--tint-ok); color: var(--green); }
</style>
