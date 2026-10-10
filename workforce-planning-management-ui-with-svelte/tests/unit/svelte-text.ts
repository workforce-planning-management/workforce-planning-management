// A small scanner for the visible text in Svelte templates, shared by the
// "no untranslated text" test (WPM-R94). It reads the template part of a component (not its
// `<script>` or `<style>`), and reports each run of literal text and each literal value of an
// attribute that a person reads (placeholder, title, aria-label, alt, label). It tracks tags,
// quoted attribute values and `{expression}` blocks, so an arrow `=>` or a comparison `>` inside
// an expression is not mistaken for the end of a tag.

const VOID = new Set([
  "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr",
]);
const SKIP_PARENT = new Set(["code", "pre", "kbd", "samp", "script", "style"]);
const ATTRIBUTES = ["placeholder", "title", "aria-label", "alt", "label", "aria-description"];

/** Index just past the `}` matching the `{` at `i`. */
function scanBraces(s: string, i: number): number {
  let depth = 0;
  let j = i;
  while (j < s.length) {
    const c = s[j];
    if (c === '"' || c === "'" || c === "`") {
      const quote = c;
      j++;
      while (j < s.length && s[j] !== quote) {
        if (s[j] === "\\") j++;
        else if (quote === "`" && s[j] === "$" && s[j + 1] === "{") j = scanBraces(s, j + 1) - 1;
        j++;
      }
    } else if (c === "{") depth++;
    else if (c === "}") {
      depth--;
      if (depth === 0) return j + 1;
    }
    j++;
  }
  throw new Error(`unbalanced { at ${i}`);
}

/** Index just past the `>` that closes the tag opening at `i`. */
function scanTag(s: string, i: number): number {
  let j = i + 1;
  while (j < s.length) {
    const c = s[j];
    if (c === '"' || c === "'") {
      const quote = c;
      j++;
      while (j < s.length && s[j] !== quote) {
        if (s[j] === "{") j = scanBraces(s, j) - 1;
        j++;
      }
    } else if (c === "{") j = scanBraces(s, j) - 1;
    else if (c === ">") return j + 1;
    j++;
  }
  throw new Error(`unterminated tag at ${i}`);
}

type Token = { kind: "text" | "expr" | "block" | "tag"; value: string };

function tokenize(s: string): Token[] {
  const out: Token[] = [];
  let buf = "";
  const flush = () => {
    if (buf) out.push({ kind: "text", value: buf });
    buf = "";
  };
  let i = 0;
  while (i < s.length) {
    const c = s[i];
    if (c === "<" && s.startsWith("<!--", i)) {
      flush();
      const j = s.indexOf("-->", i) + 3;
      out.push({ kind: "tag", value: s.slice(i, j) });
      i = j;
    } else if (c === "<" && /[A-Za-z/!:]/.test(s[i + 1] ?? "")) {
      flush();
      const j = scanTag(s, i);
      const tag = s.slice(i, j);
      const m = /^<(script|style)\b/.exec(tag);
      if (m) {
        const end = s.indexOf(`</${m[1]}>`, j) + m[1]!.length + 3;
        out.push({ kind: "tag", value: s.slice(i, end) });
        i = end;
        continue;
      }
      out.push({ kind: "tag", value: tag });
      i = j;
    } else if (c === "{") {
      flush();
      const j = scanBraces(s, i);
      const tok = s.slice(i, j);
      out.push({ kind: /^\{[#:/@]/.test(tok) ? "block" : "expr", value: tok });
      i = j;
    } else {
      buf += c;
      i++;
    }
  }
  flush();
  return out;
}

const tagName = (tag: string) => /^<\/?\s*([A-Za-z][\w:.-]*)/.exec(tag)?.[1]?.toLowerCase() ?? "";

/** Letters a reader would read: two or more in a row, once entities and `{x}` are removed. */
export function hasWords(text: string): boolean {
  const plain = text
    .replace(/&[a-z]+;|&#\d+;/gi, " ")
    .replace(/\{[^{}]*\}/g, " ");
  return /[A-Za-zÀ-ɏЀ-ӿ؀-ۿऀ-ॿঀ-৿一-鿿]{2,}/.test(plain);
}

export interface Finding {
  kind: "text" | "attribute";
  text: string;
}

/** Every literal visible string in a component's template. */
export function literalText(source: string): Finding[] {
  const found: Finding[] = [];
  const stack: string[] = [];
  // Group consecutive text and expression tokens into one run, as a reader sees them.
  let run: Token[] = [];
  const endRun = () => {
    if (run.length === 0) return;
    const text = run.filter((t) => t.kind === "text").map((t) => t.value).join("");
    const parent = stack[stack.length - 1] ?? "";
    if (!SKIP_PARENT.has(parent) && hasWords(text) && !/^\s*https?:\/\/\S*\s*$/.test(text)) {
      found.push({ kind: "text", text: text.replace(/\s+/g, " ").trim() });
    }
    run = [];
  };
  for (const token of tokenize(source)) {
    if (token.kind === "text" || token.kind === "expr") {
      run.push(token);
      continue;
    }
    endRun();
    if (token.kind === "block") continue;
    const tag = token.value;
    if (tag.startsWith("<!") || /^<(script|style)\b/.test(tag)) continue;
    const name = tagName(tag);
    for (const attribute of ATTRIBUTES) {
      const pattern = new RegExp(`\\b${attribute}="([^"]*)"`, "g");
      for (const m of tag.matchAll(pattern)) {
        if (hasWords(m[1]!)) found.push({ kind: "attribute", text: `${attribute}="${m[1]}"` });
      }
    }
    if (tag.startsWith("</")) {
      if (stack[stack.length - 1] === name) stack.pop();
    } else if (!tag.trimEnd().endsWith("/>") && !VOID.has(name)) {
      stack.push(name);
    }
  }
  endRun();
  return found;
}
