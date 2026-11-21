# Demo-handleiding

Deze handleiding beschrijft hoe u WPM op een lokale machine draait, fictieve gegevens laadt en de rondleiding langs elke pagina volgt.

> **Uitsluitend fictieve gegevens.** Aanmelden staat uit voor de rondleiding. Koppel de demo niet aan een dienst met echte gegevens.

## Vereisten

- [Rust](https://www.rust-lang.org/tools/install), stabiele versie
- [Node.js](https://nodejs.org/) en [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), met een lege database voor de rondleiding
- Python 3

De [installatiehandleiding](../../../INSTALL.md) noemt de vereiste versies en de database-instellingen.

## Stap 1: De code ophalen

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Stap 2: De service starten

De service leest `DATABASE_URL`. De standaardwaarde is `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; stel de variabele in als uw database een andere naam, gebruiker of poort heeft. Bereid de lege database voor en start de API. Aanmelden staat alleen voor dit exemplaar uit en de limiet voor aanvragen is voor de rondleiding verhoogd.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Stap 3: De fictieve gegevens laden

Voer in een tweede terminal, vanuit de hoofdmap van de repository, uit:

```sh
workforce-demo-data http://localhost:1241
```

## Stap 4: De applicatie starten

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Stap 5: De rondleiding volgen

Open het adres dat de applicatie toont en meld u aan als **example@example.com**. Er wordt geen e-mail verstuurd. Na het aanmelden opent `/demo`, een rondleiding langs elke pagina. Een banner op elke pagina meldt dat de gegevens fictief zijn en biedt de links Vorige en Volgende.

## Verder lezen

- [Rolbeschrijvingen](../for/index.md)
- [Vergelijking per product](../../../docs/compare/index.md)
- [Specificatie](../../../spec/index.md)
- [Hoe u kunt reageren](../../../RFC.md)

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/start/`), dan geldt de Engelse versie.
