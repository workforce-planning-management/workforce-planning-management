import { json } from "@sveltejs/kit";
import type { RequestHandler } from "./$types";
import { searchDocs } from "#lib/server/content.js";

// A file, not a page: /search/ fetches it, and only when someone searches. Listed in `prerender.entries` because no
// page links to it.
export const prerender = true;
export const trailingSlash = "never";

export const GET: RequestHandler = () => json(searchDocs());
