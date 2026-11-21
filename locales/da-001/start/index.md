# Demovejledning

Denne vejledning beskriver, hvordan du kører WPM på en lokal maskine, indlæser fiktive data og følger den guidede rundtur på hver side.

> **Kun fiktive data.** Login er slået fra under rundturen. Forbind den ikke til en tjeneste med rigtige data.

## Forudsætninger

- [Rust](https://www.rust-lang.org/tools/install), stabil version
- [Node.js](https://nodejs.org/) og [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), med en tom database til rundturen
- Python 3

[Installationsvejledningen](../../../INSTALL.md) angiver de krævede versioner og databaseindstillingerne.

## Trin 1: Hent koden

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Trin 2: Start tjenesten

Tjenesten læser `DATABASE_URL`. Standardværdien er `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; sæt variablen, hvis din database har et andet navn, en anden bruger eller en anden port. Klargør den tomme database og start API'et. Login er kun slået fra for denne instans, og hastighedsgrænsen er sat op til rundturen.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Trin 3: Indlæs de fiktive data

I en anden terminal, fra repositoriets rod:

```sh
workforce-demo-data http://localhost:1241
```

## Trin 4: Start applikationen

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Trin 5: Følg rundturen

Åbn den adresse, applikationen viser, og log ind som **example@example.com**. Der sendes ingen e-mail. Login åbner `/demo`, en rundtur gennem alle sider. Et banner på hver side oplyser, at dataene er fiktive, og har links til Forrige og Næste.

## Læs videre

- [Rollebeskrivelser](../for/index.md)
- [Sammenligning produkt for produkt](../../../docs/compare/index.md)
- [Specifikation](../../../spec/index.md)
- [Sådan kommenterer du](../../../RFC.md)

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/start/`), er det den engelske version, der gælder.
