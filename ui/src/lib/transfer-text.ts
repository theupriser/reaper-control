import type { ImportOffer } from "./generated/protocol";

const songs = (count: number): string => `${count} ${count === 1 ? "song" : "songs"}`;

/** One line for an offered v1 setlist: what will come along and what the project lacks. */
export function offerText(offer: ImportOffer): string {
  const found = songs(offer.found);
  if (offer.missing.length === 0) return `${found}`;
  return `${found}; not in this project: ${offer.missing.join(", ")}`;
}

/** The message after a restore or an import. */
export function doneText(count: number, what: string): string {
  if (count === 0) return `Nothing to ${what}.`;
  return `${count} ${count === 1 ? "setlist" : "setlists"} ${what === "restore" ? "restored" : "imported"}.`;
}
