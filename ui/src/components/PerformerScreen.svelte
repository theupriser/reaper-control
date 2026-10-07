<script lang="ts">
  import Clock from "./Clock.svelte";
  import PerformerControls from "./PerformerControls.svelte";
  import PerformerToggles from "./PerformerToggles.svelte";
  import RecordDot from "./RecordDot.svelte";
  import SongProgress from "./SongProgress.svelte";
  import SystemStats from "./SystemStats.svelte";
  import {
    formatLongTime,
    formatTime,
    isWaitingAtHardStop,
    type PerformerView,
    type SeekTarget,
  } from "../lib/performer";

  let {
    view,
    onPlayPause,
    onSeek,
    onToggleAutoResume,
    onToggleCountIn,
    onToggleRecord,
    onExit,
  }: {
    view: PerformerView;
    onPlayPause: () => void;
    onSeek: (target: SeekTarget) => void;
    onToggleAutoResume: () => void;
    onToggleCountIn: () => void;
    onToggleRecord: () => void;
    onExit?: () => void;
  } = $props();

  const playing = $derived(view.phase === "Playing" || view.phase === "CountingIn");
  const waiting = $derived(isWaitingAtHardStop(view));
  const songLength = $derived(view.song?.duration ?? 0);
</script>

<div class="performer" class:flash={waiting} data-phase={view.phase}>
  <header>
    <span class="setlist">{view.setlistName ? `Setlist: ${view.setlistName}` : ""}</span>
    <span class="clock">
      <RecordDot armed={view.recordArmed} onToggle={onToggleRecord} />
      <Clock />
      <SystemStats stats={view.stats} />
    </span>
  </header>

  <div class="content">
    <h1>{view.song?.name ?? "No Song Selected"}</h1>
    <div class="times">
      <div>
        <span class="label">Song:</span>
        {formatTime(view.songPosition)} <span class="sep">/</span> {formatTime(songLength)}
        <span class="remaining">({formatTime(songLength - view.songPosition)})</span>
      </div>
      <div>
        <span class="label">Total:</span>
        {formatLongTime(view.totalElapsed)} <span class="sep">/</span> {formatLongTime(view.totalDuration)}
        <span class="remaining">({formatLongTime(view.totalDuration - view.totalElapsed)})</span>
      </div>
    </div>
    {#if view.song}<SongProgress song={view.song} position={view.songPosition} {onSeek} />{/if}
    {#if view.phase === "CountingIn"}<p class="count-in">Count-in</p>{/if}

    <div class="next">
      <h2>{view.nextSong ? `Next: ${view.nextSong.name}` : "End of setlist"}</h2>
      <div class="duration">{view.nextSong ? `Duration: ${formatTime(view.nextSong.duration)}` : " "}</div>
      {#if waiting}<div class="hold">Press play to continue</div>{/if}
    </div>
  </div>

  <PerformerControls
    {playing}
    canPrevious={false}
    canNext={false}
    canPlay={view.song !== null && !(view.nextSong === null && view.phase === "HardStopped")}
    {onPlayPause}
  />
  <PerformerToggles
    autoResume={view.autoResume}
    countInOnMarker={view.countInOnMarker}
    onAutoResume={onToggleAutoResume}
    onCountIn={onToggleCountIn}
  />
  {#if onExit}<div class="exit"><button onclick={onExit}>Exit Performer Mode</button></div>{/if}
</div>

<style>
  .performer {
    min-height: 100vh;
    box-sizing: border-box;
    padding: 2rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    background: #121212;
    color: white;
  }
  .flash {
    animation: flash 2s ease-in-out infinite;
  }
  @keyframes flash {
    50% { background: #2a0000; }
  }
  @media (prefers-reduced-motion: reduce) {
    .flash { animation: none; }
  }
  header {
    display: flex;
    justify-content: space-between;
    font-size: 1.5rem;
    opacity: 0.7;
  }
  .setlist {
    font-size: 1.2rem;
    font-style: italic;
  }
  .clock {
    display: flex;
    align-items: center;
    gap: 10px;
    font-family: monospace;
  }
  .content {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1rem;
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
  }
  h1 {
    font-size: 4rem;
    margin: 0;
    line-height: 1.2;
    text-align: center;
  }
  .times {
    display: flex;
    justify-content: space-between;
    font-family: monospace;
    font-size: 1.5rem;
  }
  .label { opacity: 0.7; margin-right: 0.5rem; }
  .sep { opacity: 0.5; }
  .remaining { margin-left: 0.5rem; opacity: 0.6; color: #aaa; font-size: 0.9em; }
  .count-in {
    margin: 0;
    text-align: center;
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--amber);
  }
  .next {
    margin-top: 2rem;
    text-align: center;
  }
  h2 { font-size: 2.5rem; margin: 0 0 0.5rem; opacity: 0.8; }
  .duration { font-size: 1.5rem; opacity: 0.6; }
  .hold { margin-top: 0.5rem; font-size: 1.2rem; font-weight: bold; color: var(--red); }
  .exit { display: flex; justify-content: center; }
  .exit button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    background: #333;
    color: white;
    font-size: 1rem;
    cursor: pointer;
  }
  .exit button:hover { background: #444; }
</style>
