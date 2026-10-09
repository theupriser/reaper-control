<script lang="ts">
  import { strings } from "../lib/strings";
  import Button from "../kit/Button.svelte";
  import type { StageLock } from "../lib/stage-lock";

  let { state, onLock, onUnlock }: { state: StageLock; onLock: () => void; onUnlock: () => void } = $props();
</script>

{#if state.kind === "open"}
  <Button onclick={onLock}>{strings.performer.lock}</Button>
{:else}
  <Button kind={state.kind === "unlocking" ? "danger" : "normal"} onclick={onUnlock}>
    {state.kind === "unlocking" ? strings.performer.unlockAgain : strings.performer.unlock}
  </Button>
{/if}
