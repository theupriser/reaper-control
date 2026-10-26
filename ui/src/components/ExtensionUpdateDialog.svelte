<script lang="ts">
  import Button from "../kit/Button.svelte";
  import Dialog from "../kit/Dialog.svelte";
  import { strings } from "../lib/strings";

  let { open, reaperRunning, onUpdate, onCancel }: { open: boolean; reaperRunning: boolean; onUpdate: (restart: boolean) => void; onCancel: () => void } = $props();
  const text = strings.extensionUpdate;
</script>

<Dialog {open} title={text.dialogTitle} onclose={onCancel}>
  <p>{reaperRunning ? text.dialogRunning : text.dialogStopped}</p>
  {#snippet actions()}
    <Button onclick={onCancel}>{text.laterButton}</Button>
    <Button onclick={() => onUpdate(false)}>{text.updateOnly}</Button>
    <Button kind="primary" onclick={() => onUpdate(true)}>{text.updateAndRestart}</Button>
  {/snippet}
</Dialog>

<style>
  p { margin: 0; color: var(--muted); }
</style>
