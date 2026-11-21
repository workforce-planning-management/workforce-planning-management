# Mafunzo ya onyesho

Mafunzo haya yanaeleza jinsi ya kuendesha WPM kwenye kompyuta ya hapa nawe, kupakia data bandia na kufuata ziara ya kila ukurasa.

> **Data bandia pekee.** Kuingia kumezimwa kwa ajili ya ziara. Usiiunganishe na huduma inayohifadhi data halisi.

## Mahitaji ya awali

- [Rust](https://www.rust-lang.org/tools/install), toleo thabiti
- [Node.js](https://nodejs.org/) na [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), pamoja na hifadhidata tupu kwa ajili ya ziara
- Python 3

[Mwongozo wa usakinishaji](../../../INSTALL.md) unaeleza matoleo yanayohitajika na mipangilio ya hifadhidata.

## Hatua ya 1: Pata msimbo

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Hatua ya 2: Anzisha huduma

Huduma husoma `DATABASE_URL`. Thamani yake chaguomsingi ni `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; weka kigeugeu hicho ikiwa hifadhidata yako ina jina, mtumiaji au mlango tofauti. Andaa hifadhidata tupu kisha anzisha API. Kuingia kumezimwa kwa mfano huu pekee, na kikomo cha maombi kimeongezwa kwa ajili ya ziara.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Hatua ya 3: Pakia data bandia

Katika kituo cha pili, kutoka kwenye saraka kuu ya hazina:

```sh
workforce-demo-data http://localhost:1241
```

## Hatua ya 4: Anzisha programu

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Hatua ya 5: Fuata ziara

Fungua anwani inayoonyeshwa na programu kisha ingia kama **example@example.com**. Hakuna barua pepe inayotumwa. Kuingia kunafungua `/demo`, ziara ya kila ukurasa. Bango katika kila ukurasa linaeleza kuwa data ni bandia na lina viungo vya Iliyotangulia na Inayofuata.

## Kusoma zaidi

- [Muhtasari wa majukumu](../for/index.md)
- [Ulinganisho kwa bidhaa](../../../docs/compare/index.md)
- [Maelezo ya kiufundi](../../../spec/index.md)
- [Jinsi ya kutoa maoni](../../../RFC.md)

> Tafsiri hii ilitolewa na akili bandia na haijakaguliwa na mzungumzaji asilia wa Kiswahili. Ikitofautiana na toleo la Kiingereza (`locales/en-001/start/`), toleo la Kiingereza ndilo linalotangulia.
