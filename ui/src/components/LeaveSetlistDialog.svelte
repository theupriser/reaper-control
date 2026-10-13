<script lang="ts">
  import Button from "../kit/Button.svelte";
  import Dialog from "../kit/Dialog.svelte";
  import { strings } from "../lib/strings";

  let { open, onLeave, onStay }: { open: boolean; onLeave: () => void; onStay: () => void } = $props();
</script>

<Dialog {open} title={strings.setlists.leaveTitle} onclose={onStay}>
  <p>{strings.setlists.leaveText}</p>
  {#snippet actions()}
    <Button onclick={onStay}>{strings.setlists.leaveStay}</Button>
    <Button kind="danger" onclick={onLeave}>{strings.setlists.leaveDiscard}</Button>
  {/snippet}
</Dialog>

<style>
  p { margin: 0; color: var(--muted); }
</style>
