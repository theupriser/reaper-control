<script lang="ts">
  import Button from "../kit/Button.svelte";
  import ListRow from "../kit/ListRow.svelte";
  import StatusBadge from "../kit/StatusBadge.svelte";
  import { strings } from "../lib/strings";

  let {
    position,
    name,
    missing,
    first,
    last,
    onUp,
    onDown,
    onRemove,
  }: { position: number; name: string; missing: boolean; first: boolean; last: boolean; onUp: () => void; onDown: () => void; onRemove: () => void } = $props();
</script>

<ListRow>
  {#snippet leading()}<span class="position">{position}</span>{/snippet}
  <span class:gone={missing}>{name}</span>
  {#snippet trailing()}
    {#if missing}<span title={strings.setlists.missingSong}><StatusBadge tone="warn" label={strings.setlists.missing} /></span>{/if}
    <Button label={strings.setlists.moveUp(name)} disabled={first} onclick={onUp}>↑</Button>
    <Button label={strings.setlists.moveDown(name)} disabled={last} onclick={onDown}>↓</Button>
    <Button kind="danger" label={strings.setlists.remove(name)} onclick={onRemove}>✕</Button>
  {/snippet}
</ListRow>

<style>
  .position { min-width: 2ch; text-align: right; font-variant-numeric: tabular-nums; }
  .gone { color: var(--amber); }
</style>
