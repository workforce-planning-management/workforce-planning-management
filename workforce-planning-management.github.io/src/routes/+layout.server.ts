import type { LayoutServerLoad } from "./$types";
import { counts } from "#lib/server/content.js";

// The figures in the footer, from the copy of the documents (src/content/docs.json).
export const load: LayoutServerLoad = () => ({ counts: counts() });
