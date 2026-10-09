export type Colours = Record<string, string>;

export function parseColour(value: string): [number, number, number] | null {
  const short = /^#([0-9a-f])([0-9a-f])([0-9a-f])$/i.exec(value);
  if (short) return [short[1], short[2], short[3]].map((digit) => parseInt(digit + digit, 16)) as [number, number, number];
  const long = /^#([0-9a-f]{2})([0-9a-f]{2})([0-9a-f]{2})$/i.exec(value);
  return long ? ([long[1], long[2], long[3]].map((pair) => parseInt(pair, 16)) as [number, number, number]) : null;
}

function luminance([red, green, blue]: [number, number, number]): number {
  const [r, g, b] = [red, green, blue].map((channel) => {
    const value = channel / 255;
    return value <= 0.03928 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function contrastRatio(foreground: string, background: string): number {
  const front = parseColour(foreground);
  const back = parseColour(background);
  if (!front || !back) throw new Error(`not a plain hex colour: ${foreground} on ${background}`);
  const [high, low] = [luminance(front), luminance(back)].sort((a, b) => b - a);
  return (high + 0.05) / (low + 0.05);
}

function declarations(block: string): Colours {
  return Object.fromEntries([...block.matchAll(/(--[a-z0-9-]+):\s*([^;]+);/g)].map((match) => [match[1], match[2].trim()]));
}

/** The colours of one theme: the dark block first, then the theme's own overrides. */
export function themeColours(css: string, theme: string): Colours {
  const block = (selector: string) => {
    const start = css.indexOf(selector + " {");
    return start < 0 ? "" : css.slice(start, css.indexOf("}", start));
  };
  const base = declarations(block(':root,\n:root[data-theme="dark"]'));
  return theme === "dark" ? base : { ...base, ...declarations(block(`:root[data-theme="${theme}"]`)) };
}
