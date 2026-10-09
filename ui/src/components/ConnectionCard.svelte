<script lang="ts">
  import Button from "../kit/Button.svelte";
  import Panel from "../kit/Panel.svelte";
  import StatusBadge from "../kit/StatusBadge.svelte";
  import type { ConnectionBadge } from "../lib/connection";
  import { strings } from "../lib/strings";

  let { connection, onSetup }: { connection: ConnectionBadge; onSetup: () => void } = $props();
  const text = strings.wizard.connection;
</script>

<Panel title={text.title} hint={text.hint}>
  {#snippet action()}<StatusBadge tone={connection.tone} label={connection.label} />{/snippet}
  <div><Button onclick={onSetup}>{text.open}</Button></div>
</Panel>
