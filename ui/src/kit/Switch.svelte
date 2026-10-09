<script lang="ts">
  let { on, label, onchange, disabled = false }: { on: boolean; label: string; onchange: () => void; disabled?: boolean } = $props();
</script>

<button class="switch" role="switch" aria-checked={on} {disabled} onclick={onchange}>
  <span class="track" class:on><span class="knob"></span></span>
  {label}
</button>

<style>
  .switch { display: flex; align-items: center; gap: var(--space-3); min-height: var(--control-height); padding: 0; border: none; background: none; color: var(--text-soft); font: inherit; font-size: var(--text-body); font-weight: 600; cursor: pointer; }
  .switch:disabled { opacity: 0.5; cursor: default; }
  .track { position: relative; display: block; flex-shrink: 0; width: 44px; height: 26px; border-radius: 13px; background: var(--track); transition: background var(--motion-fast); }
  .knob { position: absolute; top: 3px; left: 3px; width: 20px; height: 20px; border-radius: 50%; background: var(--on-strong); transition: left var(--motion-fast); }
  .on { background: var(--green); }
  .on .knob { left: 21px; }
</style>
