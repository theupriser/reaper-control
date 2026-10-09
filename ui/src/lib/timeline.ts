export interface TimelineSpan {
  start: number;
  end: number;
}

export interface TimelineMark {
  percent: number;
  label: string;
}

/** Where `time` sits in the span as a percentage, clamped to 0..100. */
export const percentOf = (time: number, span: TimelineSpan): number => {
  const length = span.end - span.start;
  if (!(length > 0) || !Number.isFinite(time)) return 0;
  return Math.min(100, Math.max(0, ((time - span.start) / length) * 100));
};

export const markOf = (time: number, label: string, span: TimelineSpan): TimelineMark => ({
  percent: percentOf(time, span),
  label,
});
