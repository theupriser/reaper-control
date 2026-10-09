<script lang="ts">
  import { onMount, tick } from "svelte";
  import ComingSoon from "./components/ComingSoon.svelte";
  import Notices from "./components/Notices.svelte";
  import PerformerScreen from "./components/PerformerScreen.svelte";
  import SetlistsScreen from "./components/SetlistsScreen.svelte";
  import SettingsScreen from "./components/SettingsScreen.svelte";
  import Sidebar from "./components/Sidebar.svelte";
  import { createAppStore } from "./lib/app-store";
  import { connectionBadge } from "./lib/connection";
  import { backend } from "./lib/ipc";
  import { screenLabel, type ScreenId } from "./lib/screens";
  import { fixtureFor } from "./lib/performer-fixtures";
  import { keyAction, toKeyPress } from "./lib/keyboard";
  import { seekCommand, type PerformerPhase, type SeekTarget } from "./lib/performer";
  import { performerView } from "./lib/performer-view";
  import { strings } from "./lib/strings";
  import { isLocked, lock, open, settle, tapUnlock, type StageLock } from "./lib/stage-lock";

  const appState = createAppStore(backend);
  let screen = $state<ScreenId>("player");
  let performerMode = $state(false);
  let stageLock = $state<StageLock>(open);
  let content = $state<HTMLElement>();

  const select = async (id: ScreenId) => {
    screen = id;
    await tick();
    content?.focus();
  };

  const phases: PerformerPhase[] = ["Idle", "Playing", "Paused", "CountingIn", "HardStopped"];
  const forced = new URLSearchParams(location.search).get("phase") as PerformerPhase | null;
  const live = $derived($appState.link.live);
  const view = $derived(forced && phases.includes(forced) ? fixtureFor(forced) : performerView($appState.link, $appState.problem));

  const connection = $derived(connectionBadge($appState.link.status, $appState.problem));

  const send = appState.send;

  const playPause = () => send(live?.phase === "Playing" ? "Pause" : "Play");
  const previous = () => send("Previous");
  const rewind = () => send("RestartSong");
  const next = () => send("Next");
  const seek = (target: SeekTarget) => send(seekCommand(target, live?.count_in ?? false));
  const toggleAutoResume = () => send("ToggleAutoResume");

  function onKeydown(event: KeyboardEvent) {
    const action = keyAction(toKeyPress(event), performerMode, isLocked(stageLock));
    if (!action) return;
    event.preventDefault();
    if (action === "PlayPause") playPause();
    else if (action === "Previous") previous();
    else if (action === "Next") next();
    else if (action === "ExitPerformer") performerMode = false;
    else toggleAutoResume();
  }

  const enterPerformer = () => {
    stageLock = open;
    performerMode = true;
  };

  $effect(() => {
    if (stageLock.kind !== "unlocking") return;
    const timer = setTimeout(() => (stageLock = settle(stageLock, Date.now())), stageLock.until - Date.now() + 1);
    return () => clearTimeout(timer);
  });

  onMount(() => appState.start());
</script>

<svelte:window onkeydown={onKeydown} />

<Notices notices={$appState.notices} onDismiss={appState.dismissNotice} />

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
    {stageLock}
    onLock={() => (stageLock = lock())}
    onUnlock={() => (stageLock = tapUnlock(stageLock, Date.now()))}
  />
{:else}
  <div class="layout">
    <Sidebar active={screen} onSelect={select} onPerformer={enterPerformer} {connection} />
    <main class="content" tabindex="-1" bind:this={content}>
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
        {#if $appState.error}<p class="error" role="alert">{$appState.error}</p>{/if}
        {#if $appState.pending.length > 0}<p class="pending" role="status">{strings.pending.sending}</p>{/if}
      {:else if screen === "setlists"}
        <SetlistsScreen catalog={$appState.link.catalog} {send} />
      {:else if screen === "settings"}
        <SettingsScreen />
      {:else}
        <ComingSoon title={screenLabel(screen)} />
      {/if}
    </main>
  </div>
{/if}

<style>
  .layout {
    display: flex;
    height: 100vh;
  }
  .pending {
    color: var(--text-dim, inherit);
    padding: 0 2rem;
  }
  .error {
    color: var(--red);
    padding: 0 2rem;
  }
  .content:focus { outline: none; }
  .content {
    flex: 1;
    min-width: 0;
    overflow-y: auto;
  }
</style>
