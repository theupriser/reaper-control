<script lang="ts">
  import { onMount } from "svelte";
  import ComingSoon from "./components/ComingSoon.svelte";
  import PerformerScreen from "./components/PerformerScreen.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import { connectionBadge } from "./lib/connection";
  import { currentView, dispatch, onViewChange } from "./lib/ipc";
  import { screenLabel, type ScreenId } from "./lib/screens";
  import { fixtureFor } from "./lib/performer-fixtures";
  import { keyIntent, seekCommand, type PerformerPhase, type SeekTarget } from "./lib/performer";
  import { performerView } from "./lib/performer-view";
  import type { Command, LinkView } from "./lib/generated/protocol";

  let link = $state<LinkView>({
    status: "NotRunning",
    state: {
      phase: "Idle",
      position: 0,
      auto_resume: true,
      count_in_on_marker: false,
      record_armed: false,
      current_song: null,
    },
    catalog: { revision: 0, setlist_revision: 0, songs: [], cues: [], setlists: [], active_setlist: null },
  });
  let error = $state<string | null>(null);
  let screen = $state<ScreenId>("player");
  let performerMode = $state(false);

  const phases: PerformerPhase[] = ["Idle", "Playing", "Paused", "CountingIn", "HardStopped"];
  const forced = new URLSearchParams(location.search).get("phase") as PerformerPhase | null;
  const appState = $derived(link.state);
  const view = $derived(forced && phases.includes(forced) ? fixtureFor(forced) : performerView(link));

  const connection = $derived(connectionBadge(link.status));

  const send = async (command: Command) => {
    try {
      await dispatch(command);
      error = null;
    } catch (e) {
      error = String(e);
    }
  };

  const playPause = () => send(appState.phase === "Playing" ? "Pause" : "Play");
  const previous = () => send("Previous");
  const rewind = () => send("RestartSong");
  const next = () => send("Next");
  const seek = (target: SeekTarget) => send(seekCommand(target, appState.count_in_on_marker));
  const toggleAutoResume = () => send("ToggleAutoResume");

  function onKeydown(event: KeyboardEvent) {
    if (event.repeat || event.metaKey || event.ctrlKey || event.altKey) return;
    const intent = keyIntent(event.key);
    if (!intent) return;
    event.preventDefault();
    if (intent === "PlayPause") playPause();
    else if (intent === "Previous") previous();
    else if (intent === "Next") next();
    else toggleAutoResume();
  }

  onMount(() => {
    const show = (next: LinkView) => (link = next);
    const unlisten = onViewChange(show);
    currentView().then(show, (e) => (error = String(e)));
    return () => {
      unlisten.then((stop) => stop());
    };
  });
</script>

<svelte:window onkeydown={onKeydown} />

{#if performerMode}
  <PerformerScreen
    {view}
    onPlayPause={playPause}
    onPrevious={previous}
    onRewind={rewind}
    onNext={next}
    onSeek={seek}
    onToggleAutoResume={toggleAutoResume}
    onToggleCountIn={() => send("ToggleCountInOnMarker")}
    onToggleRecord={() => send("ToggleRecordArm")}
    onExit={() => (performerMode = false)}
  />
{:else}
  <div class="layout">
    <Sidebar active={screen} onSelect={(id) => (screen = id)} onPerformer={() => (performerMode = true)} {connection} />
    <div class="content">
      {#if screen === "player"}
        <PerformerScreen
          {view}
          onPlayPause={playPause}
    onPrevious={previous}
    onRewind={rewind}
    onNext={next}
          onSeek={seek}
          onToggleAutoResume={toggleAutoResume}
          onToggleCountIn={() => send("ToggleCountInOnMarker")}
          onToggleRecord={() => send("ToggleRecordArm")}
        />
        {#if error}<p class="error">{error}</p>{/if}
      {:else}
        <ComingSoon title={screenLabel(screen)} />
      {/if}
    </div>
  </div>
{/if}

<style>
  .layout {
    display: flex;
  }
  .error {
    color: var(--red);
    padding: 0 2rem;
  }
  .content {
    flex: 1;
    min-width: 0;
  }
</style>
