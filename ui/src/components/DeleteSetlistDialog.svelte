<script lang="ts">
  import Button from "../kit/Button.svelte";
  import Dialog from "../kit/Dialog.svelte";
  import { strings } from "../lib/strings";

  let { open, name, playing, onConfirm, onCancel }: { open: boolean; name: string; playing: boolean; onConfirm: () => void; onCancel: () => void } = $props();
</script>

<Dialog {open} title={strings.setlists.deleteTitle(name)} onclose={onCancel}>
  <p>{strings.setlists.deleteWarning}</p>
  {#if playing}<p>{strings.setlists.deleteWasPlaying}</p>{/if}
  {#snippet actions()}
    <Button onclick={onCancel}>{strings.setlists.deleteCancel}</Button>
    <Button kind="danger" onclick={onConfirm}>{strings.setlists.deleteConfirm}</Button>
  {/snippet}
</Dialog>

<style>
  p { margin: 0 0 var(--space-2) 0; color: var(--muted); }
</style>
