<script lang="ts">
  import { strings } from "../lib/strings";
  import { onDestroy } from "svelte";
  import { formatTime, popoverX, progressPercent, seekTarget, type SeekTarget, type SongView } from "../lib/performer";

  let {
    song,
    position,
    onSeek,
  }: { song: SongView; position: number; onSeek?: (target: SeekTarget) => void } = $props();

  let popover = $state<{ x: number; time: number } | null>(null);
  let hideTimer: ReturnType<typeof setTimeout> | undefined;

  function click(event: MouseEvent) {
    if (!onSeek) return;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const clickX = event.clientX - rect.left;
    const target = seekTarget(clickX, rect.width, song);
    popover = { x: popoverX(clickX, rect.width), time: target.position };
    clearTimeout(hideTimer);
    hideTimer = setTimeout(() => (popover = null), 2000);
    onSeek(target);
  }

  onDestroy(() => clearTimeout(hideTimer));
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
<div
  class="track"
  class:seekable={onSeek}
  role="progressbar"
  tabindex="-1"
  aria-valuemin={0}
  aria-valuemax={song.duration}
  aria-valuenow={position}
  onclick={click}
>
  <div class="fill" style="width: {progressPercent(position, song.duration)}%"></div>
  {#each song.cues as cue (cue.position)}
    <div class="mark" style="left: {progressPercent(cue.position, song.duration)}%" title={cue.name}></div>
  {/each}
  {#if song.hardStop}
    <div class="mark stop" style="left: 100%" title={strings.performer.hardStopPoint}></div>
  {/if}
  {#if popover}<div class="popover" style="left: {popover.x}px">{formatTime(popover.time)}</div>{/if}
</div>

<style>
  .track {
    position: relative;
    width: 100%;
    height: 12px;
    background: var(--track);
    border-radius: 6px;
  }
  .seekable { cursor: pointer; }
  .fill {
    height: 100%;
    background: var(--green);
    border-radius: 6px;
    transition: width var(--motion-normal);
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
  .popover {
    position: absolute;
    top: 0;
    padding: 4px 8px;
    border-radius: 4px;
    background: var(--track);
    color: var(--text);
    font: 0.8rem monospace;
    transform: translate(-50%, -100%);
    box-shadow: var(--shadow-soft);
    pointer-events: none;
  }
</style>
