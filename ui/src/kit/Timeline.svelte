<script lang="ts">
  import type { TimelineMark } from "../lib/timeline";
  let { playhead, markers = [], hardStop = null, label, hardStopLabel }: {
    playhead: number;
    markers?: TimelineMark[];
    hardStop?: TimelineMark | null;
    label: string;
    hardStopLabel: string;
  } = $props();
</script>

<div class="timeline" role="progressbar" aria-label={label} aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(playhead)}>
  <div class="fill" style:width="{playhead}%"></div>
  {#each markers as mark (mark.percent + mark.label)}
    <span class="marker" style:left="{mark.percent}%" title={mark.label}></span>
  {/each}
  {#if hardStop}
    <span class="hard-stop" style:left="{hardStop.percent}%" title={hardStopLabel} aria-label={hardStopLabel}></span>
  {/if}
  <span class="playhead" style:left="{playhead}%"></span>
</div>

<style>
  .timeline { position: relative; height: var(--space-4); background: var(--track); border-radius: var(--radius); margin: var(--space-2) 0; }
  .fill { position: absolute; inset: 0 auto 0 0; background: var(--green); border-radius: var(--radius); }
  .marker { position: absolute; top: calc(var(--space-1) * -1); bottom: calc(var(--space-1) * -1); width: 4px; background: var(--info); border: 1px solid var(--bg); border-radius: 2px; transform: translateX(-50%); }
  .hard-stop { position: absolute; top: calc(var(--space-2) * -1); bottom: calc(var(--space-2) * -1); width: 6px; background: var(--red); border: 1px solid var(--bg); border-radius: 3px; transform: translateX(-50%); }
  .playhead { position: absolute; top: calc(var(--space-2) * -1); bottom: calc(var(--space-2) * -1); width: 4px; background: var(--text); border: 1px solid var(--bg); border-radius: 2px; transform: translateX(-50%); }
</style>
