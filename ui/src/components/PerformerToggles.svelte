<script lang="ts">
  let { autoResume, countInOnMarker }: { autoResume: boolean; countInOnMarker: boolean } = $props();
  const word = (on: boolean) => (on ? "ON" : "OFF");
</script>

<div class="toggles">
  <div class="item" class:on={autoResume}>Auto-resume: {word(autoResume)}</div>
  <div class="item" class:on={countInOnMarker}>Count-in when pressing marker: {word(countInOnMarker)}</div>
</div>

<style>
  .toggles {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }
  .item {
    width: 100%;
    max-width: 500px;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    font-size: 1.5rem;
    font-weight: bold;
    text-align: center;
    background: rgba(244, 67, 54, 0.2);
    color: #f44336;
  }
  .item.on {
    background: rgba(76, 175, 80, 0.2);
    color: var(--green);
  }
</style>
