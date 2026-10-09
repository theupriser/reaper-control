/** Reads a view now and then every `intervalMilliseconds`; returns the function that stops it. */
export function watchView<View>(
  read: () => Promise<View>,
  onView: (view: View) => void,
  intervalMilliseconds: number,
): () => void {
  let stopped = false;
  const look = async () => {
    try {
      const view = await read();
      if (!stopped) onView(view);
    } catch {
      // the next look tries again
    }
  };
  void look();
  const timer = setInterval(look, intervalMilliseconds);
  return () => {
    stopped = true;
    clearInterval(timer);
  };
}
