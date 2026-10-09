<script lang="ts">
  import { formatTime, type PerformerView, type SeekTarget } from "../lib/performer";
  import { strings } from "../lib/strings";
  import PlayerControls from "./PlayerControls.svelte";
  import PlayerOptions from "./PlayerOptions.svelte";
  import SongProgress from "./SongProgress.svelte";

  let {
    view,
    position,
    count,
    tempo,
    onPlayPause,
    onPrevious,
    onRewind,
    onNext,
    onSeek,
    onToggleAutoResume,
    onToggleCountIn,
    onToggleRecord,
  }: {
    view: PerformerView;
    position: number;
    count: number;
    tempo: number | null;
    onPlayPause: () => void;
    onPrevious: () => void;
    onRewind: () => void;
    onNext: () => void;
    onSeek: (target: SeekTarget) => void;
    onToggleAutoResume: () => void;
    onToggleCountIn: () => void;
    onToggleRecord: () => void;
  } = $props();

  const playing = $derived(view.phase === "Playing" || view.phase === "CountingIn");
  const length = $derived(view.song?.duration ?? 0);
</script>

<section class="transport" aria-label={strings.player.controls}>
  <div class="head">
    <div class="title">
      <div class="label">{position > 0 ? strings.player.nowPlaying(position, count) : strings.player.ready(count)}</div>
      <h2>{view.song?.name ?? strings.performer.noSong}</h2>
    </div>
    <div class="figures">
      {#if tempo !== null}
        <div class="figure"><span class="caption">{strings.player.tempo}</span><span class="value">{Math.round(tempo)} <small>{strings.player.bpmUnit}</small></span></div>
      {/if}
      <div class="figure"><span class="caption">{strings.player.time}</span><span class="value">{formatTime(view.songPosition)} <small>/ {formatTime(length)}</small></span></div>
    </div>
  </div>
  {#if view.song}<SongProgress song={view.song} position={view.songPosition} {onSeek} />{/if}
  <PlayerControls
    {playing}
    canPrevious={view.hasPrevious}
    canNext={view.nextSong !== null}
    canRewind={view.song !== null}
    canPlay={view.song !== null && !(view.nextSong === null && view.phase === "HardStopped")}
    {onPlayPause}
    {onPrevious}
    {onRewind}
    {onNext}
  />
  <PlayerOptions recordArmed={view.recordArmed} autoResume={view.autoResume} countInOnMarker={view.countInOnMarker} {onToggleRecord} {onToggleAutoResume} {onToggleCountIn} />
</section>

<style>
  .transport { display: flex; flex-direction: column; gap: 14px; padding: 20px 24px 12px; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; }
  .head { display: flex; justify-content: space-between; align-items: flex-end; gap: var(--space-5); }
  .title { min-width: 0; }
  .label { font-size: 13px; font-weight: 700; letter-spacing: 1px; color: var(--green); }
  h2 { margin: 4px 0 0; font-size: 34px; font-weight: 800; line-height: 1.15; overflow-wrap: anywhere; }
  .figures { display: flex; align-items: flex-end; gap: 28px; flex-shrink: 0; }
  .figure { display: flex; flex-direction: column; align-items: flex-end; }
  .caption { font-size: 13px; font-weight: 600; color: var(--muted); }
  .value { font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 26px; font-weight: 700; }
  small { font-size: 14px; color: var(--muted); }
</style>
