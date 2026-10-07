<script lang="ts">
  import { progressPercent, type SongView } from "../lib/performer";

  let { song, position }: { song: SongView; position: number } = $props();
</script>

<div class="track" role="progressbar" aria-valuemin={0} aria-valuemax={song.duration} aria-valuenow={position}>
  <div class="fill" style="width: {progressPercent(position, song.duration)}%"></div>
  {#each song.cues as cue (cue.position)}
    <div class="mark" style="left: {progressPercent(cue.position, song.duration)}%" title={cue.name}></div>
  {/each}
  {#if song.hardStop}
    <div class="mark stop" style="left: 100%" title="Hard stop point"></div>
  {/if}
</div>

<style>
  .track {
    position: relative;
    width: 100%;
    height: 12px;
    background: #333;
    border-radius: 6px;
  }
  .fill {
    height: 100%;
    background: var(--green);
    border-radius: 6px;
    transition: width 0.2s ease-out;
  }
  .mark {
    position: absolute;
    top: -8px;
    width: 4px;
    height: 28px;
    background: var(--amber);
    transform: translateX(-50%);
  }
  .stop {
    background: var(--red);
  }
</style>
