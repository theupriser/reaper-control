import type { ImportOffer } from "./generated/protocol";
import { strings } from "./strings";

const text = strings.settings.transfer;

/** One line for an offered v1 setlist: what will come along and what the project lacks. */
export function offerText(offer: ImportOffer): string {
  const found = text.songs(offer.found);
  if (offer.missing.length === 0) return found;
  return text.notInProject(found, offer.missing.join(", "));
}

/** The message after a restore or an import. */
export function doneText(count: number, what: "restore" | "import"): string {
  if (count === 0) return text.nothingTo(what);
  return text.done(count, what === "restore" ? text.restored : text.imported);
}
