<script lang="ts">
  import Panel from "../kit/Panel.svelte";
  import { formatLongTime, formatTime, type PerformerView, type SeekTarget } from "../lib/performer";
  import type { PlayerRow } from "../lib/player-rows";
  import { strings } from "../lib/strings";
  import PerformerControls from "./PerformerControls.svelte";
  import PerformerToggles from "./PerformerToggles.svelte";
  import RecordDot from "./RecordDot.svelte";
  import SongList from "./SongList.svelte";
  import SongProgress from "./SongProgress.svelte";

  let {
    view,
    rows,
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
    rows: PlayerRow[];
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

<section class="player">
  <header>
    <h1>{view.setlistName ?? strings.screens.player}</h1>
    <RecordDot armed={view.recordArmed} onToggle={onToggleRecord} />
  </header>

  <div class="grid">
    <div class="column">
      <Panel>
        <h2 class="song">{view.song?.name ?? strings.performer.noSong}</h2>
        <div class="times">
          <span>{formatTime(view.songPosition)} / {formatTime(length)}</span>
          {#if tempo !== null}<span class="tempo">{strings.player.bpm(tempo)}</span>{/if}
          <span class="total">{strings.performer.totalTime} {formatLongTime(view.totalElapsed)} / {formatLongTime(view.totalDuration)}</span>
        </div>
        {#if view.song}<SongProgress song={view.song} position={view.songPosition} {onSeek} />{/if}
        <PerformerControls
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
        <PerformerToggles autoResume={view.autoResume} countInOnMarker={view.countInOnMarker} onAutoResume={onToggleAutoResume} onCountIn={onToggleCountIn} />
      </Panel>
    </div>
    <div class="column">
      <Panel title={strings.player.songs}>
        {#if rows.length === 0}<p class="empty">{strings.player.noSetlist}</p>{:else}<SongList {rows} />{/if}
      </Panel>
    </div>
  </div>
</section>

<style>
  .player { padding: clamp(var(--space-3), 3vw, var(--space-5)); display: flex; flex-direction: column; gap: var(--space-4); container-type: inline-size; }
  header { display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); }
  h1 { margin: 0; font-size: clamp(1.25rem, 3vw, 1.75rem); font-weight: 800; }
  .grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: var(--space-4); align-items: start; }
  @container (max-width: 760px) { .grid { grid-template-columns: minmax(0, 1fr); } }
  .column { min-width: 0; }
  .song { margin: 0; font-size: clamp(1.5rem, 5vw, 3rem); line-height: 1.15; text-align: center; overflow-wrap: anywhere; }
  .times { display: flex; flex-wrap: wrap; justify-content: space-between; gap: var(--space-2) var(--space-4); font-family: ui-monospace, Menlo, monospace; font-size: clamp(0.95rem, 2vw, 1.25rem); }
  .tempo { color: var(--amber); }
  .total { color: var(--muted); }
  .empty { margin: 0; color: var(--muted); }
</style>
