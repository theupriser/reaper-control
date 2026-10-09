<script lang="ts">
  import { onMount } from "svelte";
  import KitClock from "../kit/Clock.svelte";

  let now = $state(new Date());
  onMount(() => {
    const timer = setInterval(() => (now = new Date()), 1000);
    return () => clearInterval(timer);
  });
</script>

<span data-testid="clock"><KitClock text={now.toLocaleTimeString()} /></span>
