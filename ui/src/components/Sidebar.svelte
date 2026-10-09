<script lang="ts">
  import { strings } from "../lib/strings";
  import { screens, type ScreenId } from "../lib/screens";

  let {
    active,
    onSelect,
    onPerformer,
    connection,
  }: {
    active: ScreenId;
    onSelect: (id: ScreenId) => void;
    onPerformer: () => void;
    connection: { label: string; detail: string; tone: "ok" | "error" };
  } = $props();
</script>

<nav aria-label={strings.screens.mainNavigation}>
  <div class="brand">
    <div class="logo">
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" style="stroke: var(--on-accent)" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12h3l2-7 4 14 3-10 2 3h4" /></svg>
    </div>
    <div class="name">{strings.app.name}</div>
  </div>

  {#each screens as screen (screen.id)}
    <button class="item" class:active={screen.id === active} aria-current={screen.id === active ? "page" : undefined} onclick={() => onSelect(screen.id)}>
      <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d={screen.icon} /></svg>
      <span>{screen.label}</span>
    </button>
  {/each}

  <div class="spacer"></div>

  <button class="performer" onclick={onPerformer}>
    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor"><path d="M8 5v14l11-7z" /></svg>
    {strings.screens.performerMode}
  </button>

  <div class="connection">
    <div class="state"><span class="dot {connection.tone}"></span>{connection.label}</div>
    <div class="detail">{connection.detail}</div>
  </div>
</nav>

<style>
  nav {
    width: 240px;
    flex-shrink: 0;
    box-sizing: border-box;
    min-height: 100vh;
    background: var(--sidebar);
    border-right: 1px solid var(--line);
    padding: 24px 16px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 0 8px 24px 8px;
  }
  .logo {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: var(--green);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .name {
    font-size: 16px;
    font-weight: 700;
    letter-spacing: 0.2px;
  }
  .item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border: none;
    border-radius: 10px;
    background: transparent;
    color: var(--text-soft);
    font-size: 15px;
    font-weight: 600;
    min-height: 44px;
    box-sizing: border-box;
    cursor: pointer;
    text-align: left;
  }
  .item.active {
    background: var(--raised);
    color: var(--green);
  }
  .spacer {
    flex: 1;
  }
  .performer {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    padding: 14px 16px;
    border: none;
    border-radius: 12px;
    background: var(--green);
    color: var(--on-accent);
    font-size: 15px;
    font-weight: 800;
    min-height: 48px;
    box-sizing: border-box;
    margin-bottom: 16px;
    cursor: pointer;
  }
  .connection {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 12px;
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 700;
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
  }
  .dot.ok {
    background: var(--green);
  }
  .dot.error {
    background: var(--red);
  }
  .detail {
    font-size: 13px;
    color: var(--muted);
    font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
</style>
