<script lang="ts">
  import Button from "../kit/Button.svelte";
  import Panel from "../kit/Panel.svelte";
  import { strings } from "../lib/strings";

  let { onSetup }: { onSetup: () => void } = $props();
  const text = strings.wizard.connection;
</script>

<Panel title={text.title} hint={text.hint}>
  <div><Button onclick={onSetup}>{text.open}</Button></div>
</Panel>
