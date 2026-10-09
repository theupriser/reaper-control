/** Runs in the page before the app: a stand-in for the Tauri backend, so the built UI runs in a plain browser. */
export function tauriStandIn(initial: unknown): void {
  const w = window as unknown as Record<string, any>;
  const callbacks = new Map<number, (event: unknown) => void>();
  const listeners = new Map<string, number[]>();
  let next = 1;
  const answers: Record<string, unknown> = {
    current_view: initial,
    current_problem: null,
    current_system_stats: { machine_cpu_percent: 12, memory_used_megabytes: 4000, memory_total_megabytes: 16000, app_memory_megabytes: 90, reaper_memory_megabytes: 600 },
    current_settings: {
      settings: { queue_repeat_window_milliseconds: 250, queue_timeout_milliseconds: 2000, queue_capacity: 32, midi_enabled: false, midi_device_name: null, midi_channel: null, midi_debounce_milliseconds: 50, log_level: "info" },
      devices: [],
      notes: [{ note: 60, action: "Play" }],
    },
    current_transfer: { restorable: 0, offers: [] },
  };
  w.__TAURI_INTERNALS__ = {
    transformCallback(callback: (event: unknown) => void) {
      const id = next++;
      callbacks.set(id, callback);
      return id;
    },
    async invoke(command: string, args: any) {
      if (command === "plugin:event|listen") {
        listeners.set(args.event, [...(listeners.get(args.event) ?? []), args.handler]);
        return args.handler;
      }
      return command in answers ? answers[command] : null;
    },
  };
  w.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
  w.__pushEvent = (event: string, payload: unknown) =>
    (listeners.get(event) ?? []).forEach((id) => callbacks.get(id)?.({ event, id, payload }));
}

/** Records, for every key press and click, the time until the frame after it was painted. */
export function inputToPaintRecorder(slowDownMilliseconds: number): void {
  const w = window as unknown as Record<string, any>;
  w.__samples = [] as number[];
  w.__mark = (label: string, start: number) =>
    requestAnimationFrame(() => requestAnimationFrame(() => w.__samples.push({ label, milliseconds: performance.now() - start })));
  for (const type of ["keydown", "pointerdown"]) {
    window.addEventListener(type, (event) => w.__mark(type, event.timeStamp), true);
  }
  if (slowDownMilliseconds > 0) {
    window.addEventListener("keydown", () => { const end = performance.now() + slowDownMilliseconds; while (performance.now() < end); });
  }
}
