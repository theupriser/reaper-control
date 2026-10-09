<script lang="ts">
  import type { PerformerView, SeekTarget } from "../lib/performer";
  import type { PlayerRow } from "../lib/player-rows";
  import { strings } from "../lib/strings";
  import PlayerTransport from "./PlayerTransport.svelte";
  import SetlistPicker from "./SetlistPicker.svelte";
  import SongList from "./SongList.svelte";

  let {
    view,
    rows,
    setlists,
    active,
    tempo,
    onChooseSetlist,
    onPlayPause,
    onPrevious,
    onRewind,
    onNext,
    onChooseSong,
    onSeek,
    onToggleAutoResume,
    onToggleCountIn,
    onToggleRecord,
  }: {
    view: PerformerView;
    rows: PlayerRow[];
    setlists: { id: string; name: string }[];
    active: string | null;
    tempo: number | null;
    onChooseSetlist: (id: string | null) => void;
    onPlayPause: () => void;
    onPrevious: () => void;
    onRewind: () => void;
    onNext: () => void;
    onChooseSong: (index: number) => void;
    onSeek: (target: SeekTarget) => void;
    onToggleAutoResume: () => void;
    onToggleCountIn: () => void;
    onToggleRecord: () => void;
  } = $props();

  const position = $derived(rows.findIndex((row) => row.state === "current") + 1);
</script>

<section class="player">
  <header>
    <h1>{strings.screens.player}</h1>
    <SetlistPicker {setlists} {active} onChoose={onChooseSetlist} />
  </header>

  <PlayerTransport {view} {position} count={rows.length} {tempo} {onPlayPause} {onPrevious} {onRewind} {onNext} {onSeek} {onToggleAutoResume} {onToggleCountIn} {onToggleRecord} />

  <section class="songs" aria-label={strings.player.songsTitle}>
    <div class="songs-head">
      <h2>{strings.player.songsTitle} <span>{strings.player.inSetlist(rows.length)}</span></h2>
    </div>
    <div class="list">
      {#if rows.length === 0}<p class="empty">{strings.player.noSetlist}</p>{:else}<SongList {rows} onChoose={onChooseSong} />{/if}
    </div>
  </section>
</section>

<style>
  .player { display: flex; flex-direction: column; gap: var(--space-4); padding: 28px 32px; min-height: 0; }
  header { display: flex; align-items: center; justify-content: space-between; gap: var(--space-4); }
  h1 { margin: 0; font-size: 28px; font-weight: 800; }
  .songs { display: flex; flex-direction: column; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; overflow: hidden; }
  .songs-head { padding: 14px 24px; border-bottom: 1px solid var(--line); }
  h2 { margin: 0; font-size: 16px; font-weight: 700; }
  h2 span { color: var(--muted); font-weight: 500; }
  .list { overflow-y: auto; }
  .empty { margin: 0; padding: var(--space-4) 24px; color: var(--muted); }
</style>
