#!/usr/bin/env python3
"""Fetch ESCO through its public web-service API and write the three CSV files
`import_esco` reads, in the layout of the official CSV download:

    skills_<lang>.csv   occupations_<lang>.csv   occupationSkillRelations_<lang>.csv

Use this when you do not have the emailed download package. It crawls the
taxonomy (the API's search stops after 200 results) and makes roughly 3,700
polite requests (a few threads, with retries), caching every response
under <out>/.cache so an interrupted run resumes.

    scripts/esco-fetch.py --out /path/to/esco-csv [--lang en] [--version v1.2.1]

The API reports a skill's type as "skill" / "knowledge" and its reuse level as
a long title ("sector specific skills and competences"); the CSV download says
"skill/competence" and "sector-specific", so this script translates them.

ESCO is reused under Commission Decision 2011/833/EU with attribution
(spec/esco/index.md).
"""
import argparse, concurrent.futures, csv, hashlib, json, os, subprocess, sys, time, urllib.parse

API = "https://ec.europa.eu/esco/api"
UA = "workforce-planning-management-esco-fetch/1.0 (reference data import)"


def http_get(url):
    """GET with curl: it keeps full certificate verification and, unlike some
    Python builds, completes this server's certificate chain."""
    done = subprocess.run(
        ["curl", "-sS", "-f", "-m", "60", "-A", UA, "-H", "Accept: application/json", url],
        capture_output=True,
    )
    if done.returncode != 0:
        raise RuntimeError(done.stderr.decode("utf8", "replace").strip() or f"curl exit {done.returncode}")
    return done.stdout

REUSE = {
    "transversal skills and competences": "transversal",
    "cross-sector skills and competences": "cross-sector",
    "sector specific skills and competences": "sector-specific",
    "occupation specific skills and competences": "occupation-specific",
}
SKILL_TYPE = {"skill": "skill/competence", "knowledge": "knowledge"}


class Client:
    def __init__(self, cache, version, lang):
        self.cache, self.version, self.lang = cache, version, lang
        self.threads = 4
        os.makedirs(cache, exist_ok=True)

    def get(self, path, **params):
        params.update(language=self.lang, selectedVersion=self.version)
        return self.fetch(f"{API}/{path}?{urllib.parse.urlencode(params, safe=':/')}")

    def get_raw(self, path_and_query):
        """A request whose query is already built (the bulk skill form)."""
        return self.fetch(f"{API}/{path_and_query}&language={self.lang}&selectedVersion={self.version}")

    def fetch(self, url):
        key = os.path.join(self.cache, hashlib.sha1(url.encode()).hexdigest() + ".json")
        if os.path.exists(key):
            with open(key, encoding="utf8") as f:
                return json.load(f)
        for attempt in range(6):
            try:
                data = json.loads(http_get(url))
                with open(key, "w", encoding="utf8") as f:
                    json.dump(data, f)
                return data
            except Exception as error:  # noqa: BLE001 - retry any transport error
                wait = 2 ** attempt
                print(f"retry {attempt + 1} after {wait}s: {error}", file=sys.stderr)
                time.sleep(wait)
        raise SystemExit(f"giving up on {url}")


def literal(value, lang):
    """A description / label field as plain text."""
    if isinstance(value, dict):
        inner = value.get(lang) or next(iter(value.values()), None)
        return literal(inner, lang)
    if isinstance(value, list):
        return "\n".join(literal(v, lang) for v in value)
    return (value or "") if isinstance(value, str) else (value or {}).get("literal", "")


def crawl_occupations(client):
    """Every occupation URI: from the occupations scheme down through the ISCO
    groups (narrowerConcept) to their occupations (narrowerOccupation)."""
    scheme = client.get("resource/taxonomy", uri="http://data.europa.eu/esco/concept-scheme/occupations")
    queue = [t["uri"] for t in scheme["_links"]["hasTopConcept"]]
    groups, occupations = set(), []
    seen_occupation = set()
    while queue:
        uri = queue.pop()
        if uri in groups:
            continue
        groups.add(uri)
        links = client.get("resource/concept", uri=uri)["_links"]
        queue.extend(c["uri"] for c in links.get("narrowerConcept") or [])
        for o in links.get("narrowerOccupation") or []:
            if o["uri"] not in seen_occupation:
                seen_occupation.add(o["uri"])
                occupations.append(o["uri"])
    print(f"  {len(groups)} ISCO groups, {len(occupations)} occupations", file=sys.stderr)
    return occupations


def crawl_skills(client):
    """Every skill URI, from the skills hierarchy: groups (narrowerConcept)
    down to their skills (narrowerSkill). The `member-skills` list alone
    misses some."""
    scheme = client.get("resource/taxonomy", uri="http://data.europa.eu/esco/concept-scheme/skills-hierarchy")
    frontier = [t["uri"] for t in scheme["_links"]["hasTopConcept"]]
    groups, skills = set(), set()

    def node(uri):
        return client.get("resource/concept", uri=uri)["_links"]

    with concurrent.futures.ThreadPoolExecutor(max_workers=client.threads) as pool:
        while frontier:
            batch = [u for u in dict.fromkeys(frontier) if u not in groups]
            frontier = []
            for uri, links in zip(batch, pool.map(node, batch)):
                groups.add(uri)
                frontier.extend(c["uri"] for c in links.get("narrowerConcept") or [])
                skills.update(sk["uri"] for sk in links.get("narrowerSkill") or [])
    print(f"  {len(groups)} skill groups, {len(skills)} skills in the hierarchy", file=sys.stderr)
    return skills


def bulk_skills(client, uris, size=25):
    """Skill resources, `size` per request (the API's bulk form)."""
    out = {}
    chunks = [uris[i:i + size] for i in range(0, len(uris), size)]

    def fetch(chunk):
        query = "&".join("uris=" + urllib.parse.quote(u, safe=":/") for u in chunk)
        # Bulk URL carries repeated `uris`; build it by hand and cache on it.
        return client.get_raw("resource/skill?" + query)

    with concurrent.futures.ThreadPoolExecutor(max_workers=client.threads) as pool:
        for i, page in enumerate(pool.map(fetch, chunks), 1):
            if i % 50 == 0:
                print(f"  skills {i * size}/{len(uris)}", file=sys.stderr, flush=True)
            for uri, resource in (page.get("_embedded") or {}).items():
                out[uri] = resource
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--lang", default="en")
    ap.add_argument("--version", default="v1.2.1")
    ap.add_argument("--threads", type=int, default=4)
    args = ap.parse_args()
    client = Client(os.path.join(args.out, ".cache"), args.version, args.lang)
    client.threads = args.threads

    print("occupations…", file=sys.stderr)
    occupation_uris = crawl_occupations(client)

    def detail(uri):
        return client.get("resource/occupation", uri=uri)

    occupation_rows, relations = [], []
    skill_uris = set()
    done, frontier = set(), list(occupation_uris)
    # An occupation can have narrower occupations of its own, so close over
    # `narrowerOccupation` until nothing new appears.
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.threads) as pool:
        while frontier:
            batch = [u for u in dict.fromkeys(frontier) if u not in done]
            frontier = []
            for d in pool.map(detail, batch):
                done.add(d["uri"])
                if len(done) % 200 == 0:
                    print(f"  occupation detail {len(done)}", file=sys.stderr, flush=True)
                links = d.get("_links", {})
                frontier.extend(o["uri"] for o in links.get("narrowerOccupation") or [])
                isco = (links.get("broaderIscoGroup") or [{}])[0].get("code", "")
                if not isco and d.get("code", "")[:4].isdigit():
                    # A narrower occupation has no ISCO link of its own; its ESCO
                    # code ("2512.4.1") starts with the ISCO unit group.
                    isco = d["code"][:4]
                occupation_rows.append([
                    "Occupation", d["uri"], isco, d["title"], "", "", d.get("status", ""), "", "", "", "", "",
                    literal(d.get("description"), args.lang), d.get("code", ""),
                ])
                for kind, key in (("essential", "hasEssentialSkill"), ("optional", "hasOptionalSkill")):
                    for sk in links.get(key) or []:
                        skill_uris.add(sk["uri"])
                        relations.append([d["uri"], kind, SKILL_TYPE.get(sk.get("skillType", "").rsplit("/", 1)[-1], ""), sk["uri"]])
    print(f"  {len(occupation_rows)} occupations", file=sys.stderr)

    print("skills…", file=sys.stderr)
    skill_uris.update(crawl_skills(client))
    resources = bulk_skills(client, sorted(skill_uris))
    # Close over skill → optional-skill links, in case some skills hang off others.
    extra = {l["uri"] for r in resources.values() for l in (r.get("_links", {}).get("hasOptionalSkill") or [])} - set(resources)
    while extra:
        resources.update(bulk_skills(client, sorted(extra)))
        extra = {l["uri"] for r in resources.values() for l in (r.get("_links", {}).get("hasOptionalSkill") or [])} - set(resources)
    skill_rows = []
    for uri, s in sorted(resources.items(), key=lambda kv: kv[1].get("title", "")):
        links = s.get("_links", {})
        kind = (links.get("hasSkillType") or [{}])[0].get("title", "")
        reuse = (links.get("hasReuseLevel") or [{}])[0].get("title", "")
        skill_rows.append([
            "KnowledgeSkillCompetence", uri, SKILL_TYPE.get(kind, kind), REUSE.get(reuse, reuse),
            s.get("title", ""), "", "", s.get("status", ""), "", "", literal(s.get("description"), args.lang),
        ])
    print(f"  {len(skill_rows)} skills", file=sys.stderr)

    def write(name, header, rows):
        path = os.path.join(args.out, f"{name}_{args.lang}.csv")
        with open(path, "w", encoding="utf8", newline="") as f:
            w = csv.writer(f)
            w.writerow(header)
            w.writerows(rows)
        print(f"wrote {path}: {len(rows)} rows")

    write("skills", ["conceptType", "conceptUri", "skillType", "reuseLevel", "preferredLabel", "altLabels",
                     "hiddenLabels", "status", "modifiedDate", "scopeNote", "description"], skill_rows)
    write("occupations", ["conceptType", "conceptUri", "iscoGroup", "preferredLabel", "altLabels", "hiddenLabels",
                          "status", "modifiedDate", "regulatedProfessionNote", "scopeNote", "definition",
                          "inScheme", "description", "code"], occupation_rows)
    write("occupationSkillRelations", ["occupationUri", "relationType", "skillType", "skillUri"], relations)


if __name__ == "__main__":
    main()
