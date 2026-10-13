<script lang="ts">
  let { text }: { text: string } = $props();

  let frame = $state<HTMLElement>();
  let line = $state<HTMLElement>();

  const START_PAUSE_SECONDS = 2.5;
  const END_PAUSE_SECONDS = 1.5;
  const SLIDE_PIXELS_PER_SECOND = 60;

  // Shows the start of the text, waits, slides to its end, waits and starts over; text that fits stays put.
  $effect(() => {
    text;
    if (!frame || !line) return;
    const shown = frame;
    const moving = line;
    let animation: Animation | undefined;
    const restart = () => {
      animation?.cancel();
      animation = undefined;
      const overflow = Math.ceil(moving.scrollWidth - shown.clientWidth);
      if (overflow <= 0 || matchMedia("(prefers-reduced-motion: reduce)").matches) return;
      const slide = overflow / SLIDE_PIXELS_PER_SECOND;
      const total = START_PAUSE_SECONDS + slide + END_PAUSE_SECONDS;
      const away = `translateX(-${overflow}px)`;
      animation = moving.animate(
        [
          { transform: "translateX(0)", offset: 0 },
          { transform: "translateX(0)", offset: START_PAUSE_SECONDS / total },
          { transform: away, offset: (START_PAUSE_SECONDS + slide) / total },
          { transform: away, offset: 1 },
        ],
        { duration: total * 1000, iterations: Infinity },
      );
    };
    restart();
    const watcher = new ResizeObserver(restart);
    watcher.observe(shown);
    return () => {
      watcher.disconnect();
      animation?.cancel();
    };
  });
</script>

<div class="frame" bind:this={frame} title={text}>
  <span class="line" bind:this={line}>{text}</span>
</div>

<style>
  .frame { overflow: hidden; white-space: nowrap; }
  .line { display: inline-block; }
  @media (prefers-reduced-motion: reduce) {
    .frame { text-overflow: ellipsis; }
  }
</style>
