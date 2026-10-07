<script lang="ts">
  let {
    autoResume,
    countInOnMarker,
    onAutoResume,
    onCountIn,
  }: { autoResume: boolean; countInOnMarker: boolean; onAutoResume: () => void; onCountIn: () => void } = $props();
  const word = (on: boolean) => (on ? "ON" : "OFF");
</script>

<div class="toggles">
  <button class="item" class:on={autoResume} aria-pressed={autoResume} onclick={onAutoResume}>
    Auto-resume: {word(autoResume)}
  </button>
  <button class="item" class:on={countInOnMarker} aria-pressed={countInOnMarker} onclick={onCountIn}>
    Count-in when pressing marker: {word(countInOnMarker)}
  </button>
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
    border: none;
    border-radius: 4px;
    font: inherit;
    font-size: 1.5rem;
    font-weight: bold;
    text-align: center;
    cursor: pointer;
    background: rgba(244, 67, 54, 0.2);
    color: #f44336;
  }
  .item:active { transform: scale(0.98); }
  .item.on {
    background: rgba(76, 175, 80, 0.2);
    color: var(--green);
  }
</style>
