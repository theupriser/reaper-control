export interface DragPoint {
  x: number;
  y: number;
}

/** Pixels the pointer must travel before a press counts as a drag, so a tap on a button still taps. */
const DRAG_THRESHOLD = 6;

/** Where a dragged row would be dropped: the number of rows whose middle lies above the pointer. */
export const insertionIndex = (rowMiddles: number[], pointerY: number): number => rowMiddles.filter((middle) => middle < pointerY).length;

export interface DragHandlers {
  onStart: () => void;
  onMove: (point: DragPoint) => void;
  onDrop: (point: DragPoint) => void;
  onCancel: () => void;
}

/** Svelte action: press and drag on a handle with mouse, pen or finger; reports the pointer, never moves the element. */
export function dragHandle(node: HTMLElement, handlers: DragHandlers) {
  let current = handlers;
  let origin: DragPoint | null = null;
  let dragging = false;

  const point = (event: PointerEvent): DragPoint => ({ x: event.clientX, y: event.clientY });
  const end = (event: PointerEvent) => {
    if (node.hasPointerCapture(event.pointerId)) node.releasePointerCapture(event.pointerId);
    origin = null;
  };

  const down = (event: PointerEvent) => {
    if (event.button !== 0) return;
    event.preventDefault();
    origin = point(event);
    dragging = false;
    node.setPointerCapture(event.pointerId);
  };
  const move = (event: PointerEvent) => {
    if (!origin) return;
    const here = point(event);
    if (!dragging) {
      if (Math.hypot(here.x - origin.x, here.y - origin.y) < DRAG_THRESHOLD) return;
      dragging = true;
      current.onStart();
    }
    current.onMove(here);
  };
  const up = (event: PointerEvent) => {
    if (origin && dragging) current.onDrop(point(event));
    dragging = false;
    end(event);
  };
  const cancel = (event: PointerEvent) => {
    if (origin && dragging) current.onCancel();
    dragging = false;
    end(event);
  };
  const escape = (event: KeyboardEvent) => {
    if (event.key !== "Escape" || !dragging) return;
    dragging = false;
    origin = null;
    current.onCancel();
  };

  node.addEventListener("pointerdown", down);
  node.addEventListener("pointermove", move);
  node.addEventListener("pointerup", up);
  node.addEventListener("pointercancel", cancel);
  window.addEventListener("keydown", escape);
  return {
    update(next: DragHandlers) {
      current = next;
    },
    destroy() {
      node.removeEventListener("pointerdown", down);
      node.removeEventListener("pointermove", move);
      node.removeEventListener("pointerup", up);
      node.removeEventListener("pointercancel", cancel);
      window.removeEventListener("keydown", escape);
    },
  };
}
