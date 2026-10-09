<script lang="ts">
  import { strings } from "../lib/strings";
  let {
    playing,
    canPrevious,
    canNext,
    canRewind,
    canPlay,
    onPlayPause,
    onPrevious,
    onRewind,
    onNext,
  }: {
    playing: boolean;
    canPrevious: boolean;
    canNext: boolean;
    canRewind: boolean;
    canPlay: boolean;
    onPlayPause: () => void;
    onPrevious: () => void;
    onRewind: () => void;
    onNext: () => void;
  } = $props();
</script>

<div class="controls">
  <button aria-label={strings.performer.previous} disabled={!canPrevious} onclick={onPrevious}>
    <svg viewBox="0 0 24 24" width="36" height="36"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z" fill="currentColor" /></svg>
  </button>
  <button aria-label={strings.performer.rewind} disabled={!canRewind} onclick={onRewind}>
    <svg viewBox="0 0 24 24" width="36" height="36"><path d="M12 5V1L7 6l5 5V7c3.31 0 6 2.69 6 6s-2.69 6-6 6-6-2.69-6-6H4c0 4.42 3.58 8 8 8s8-3.58 8-8-3.58-8-8-8z" fill="currentColor" /></svg>
  </button>
  <button class="play" aria-label={playing ? strings.performer.pause : strings.performer.play} disabled={!canPlay} onclick={onPlayPause}>
    {#if playing}
      <svg viewBox="0 0 24 24" width="48" height="48"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z" fill="currentColor" /></svg>
    {:else}
      <svg viewBox="0 0 24 24" width="48" height="48"><path d="M8 5v14l11-7z" fill="currentColor" /></svg>
    {/if}
  </button>
  <button aria-label={strings.performer.nextSong} disabled={!canNext} onclick={onNext}>
    <svg viewBox="0 0 24 24" width="36" height="36"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z" fill="currentColor" /></svg>
  </button>
</div>

<style>
  .controls {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 2rem;
  }
  button {
    background: none;
    border: none;
    color: var(--text);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    border-radius: 50%;
    cursor: pointer;
  }
  button:hover:enabled {
    background: var(--tint-hover);
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .play {
    background: var(--track);
    width: 100px;
    height: 100px;
  }
  .play:hover:enabled {
    background: var(--track-hover);
  }
</style>
