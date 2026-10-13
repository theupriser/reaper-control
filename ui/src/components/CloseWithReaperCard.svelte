<script lang="ts">
  import Panel from "../kit/Panel.svelte";
  import Switch from "../kit/Switch.svelte";
  import { strings } from "../lib/strings";

  let { on, onToggle }: { on: boolean; onToggle: () => void } = $props();
</script>

<Panel title={strings.closeWithReaper.title} hint={strings.closeWithReaper.hint}>
  <Switch {on} label={strings.closeWithReaper.switchLabel} onchange={onToggle} />
</Panel>
