<script lang="ts">
  import { strings } from "../lib/strings";
  let { title }: { title: string } = $props();
</script>

<section>
  <h1>{title}</h1>
  <p>{strings.screens.comingSoon}</p>
</section>

<style>
  section {
    padding: 28px 32px;
  }
  h1 {
    margin: 0 0 12px 0;
    font-size: 28px;
    font-weight: 800;
  }
  p {
    margin: 0;
    color: var(--muted);
  }
</style>
