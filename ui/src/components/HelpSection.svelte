<script lang="ts">
  type Card = { code: string; name: string; text: string; tone: "stop" | "length" | "tempo" };
  let {
    id,
    title,
    intro,
    steps,
    cards,
    example,
  }: { id: string; title: string; intro?: string; steps?: readonly string[]; cards?: readonly Card[]; example?: string } = $props();
</script>

<section {id} class="card">
  <h2>{title}</h2>
  {#if intro}<p class="intro">{intro}</p>{/if}
  {#if steps}
    <ul>
      {#each steps as step}<li>{step}</li>{/each}
    </ul>
  {/if}
  {#if cards}
    <div class="cards">
      {#each cards as card (card.code)}
        <div class="token">
          <code class={card.tone}>{card.code}</code>
          <h3>{card.name}</h3>
          <p>{card.text}</p>
        </div>
      {/each}
    </div>
  {/if}
  {#if example}<code class="example">{example}</code>{/if}
</section>

<style>
  .card { display: flex; flex-direction: column; gap: 10px; padding: 22px 24px; background: var(--panel); border: 1px solid var(--line); border-radius: 16px; scroll-margin-top: var(--space-4); }
  h2 { margin: 0; font-size: var(--text-title); font-weight: 800; }
  .intro, ul, p { margin: 0; font-size: var(--text-body); line-height: 1.6; color: var(--text-soft); }
  ul { padding-left: 20px; display: flex; flex-direction: column; gap: var(--space-2); }
  .cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: var(--space-3); }
  .token { display: flex; flex-direction: column; align-items: flex-start; gap: var(--space-2); padding: 14px var(--space-4); border-radius: 12px; background: var(--raised); }
  h3 { margin: 0; font-size: var(--text-body); font-weight: 700; }
  .token p { font-size: 14px; color: var(--muted); line-height: 1.5; }
  code { padding: 4px 10px; border-radius: var(--space-2); font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: var(--text-body); font-weight: 700; }
  .stop { color: var(--red); background: color-mix(in srgb, var(--red) 14%, transparent); }
  .length { color: var(--amber); background: color-mix(in srgb, var(--amber) 14%, transparent); }
  .tempo { color: var(--info); background: color-mix(in srgb, var(--info) 14%, transparent); }
  .example { align-self: flex-start; color: var(--amber); background: var(--bg); border: 1px solid var(--line); }
</style>
