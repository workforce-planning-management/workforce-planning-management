# Demonstrationsguide

Den här guiden beskriver hur du kör WPM på en lokal maskin, läser in fiktiva data och följer den guidade rundturen genom varje sida.

> **Endast fiktiva data.** Inloggningen är avstängd under rundturen. Anslut den inte till en tjänst som innehåller verkliga data.

## Förutsättningar

- [Rust](https://www.rust-lang.org/tools/install), stabil version
- [Node.js](https://nodejs.org/) och [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), med en tom databas för rundturen
- Python 3

[Installationsguiden](../../../INSTALL.md) anger vilka versioner som krävs och databasinställningarna.

## Steg 1: Hämta koden

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Steg 2: Starta tjänsten

Tjänsten läser `DATABASE_URL`. Standardvärdet är `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; ange variabeln om din databas har ett annat namn, en annan användare eller en annan port. Förbered den tomma databasen och starta API:et. Inloggningen är avstängd endast för den här instansen, och hastighetsbegränsningen är höjd för rundturen.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Steg 3: Läs in de fiktiva data

Kör följande i en andra terminal, från arkivets rot:

```sh
workforce-demo-data http://localhost:1241
```

## Steg 4: Starta applikationen

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Steg 5: Följ rundturen

Öppna adressen som applikationen visar och logga in som **example@example.com**. Inget e-postmeddelande skickas. Inloggningen öppnar `/demo`, en rundtur genom varje sida. Ett fält på varje sida anger att data är fiktiva och har länkar till Föregående och Nästa.

## Läs vidare

- [Rollbeskrivningar](../for/index.md)
- [Jämförelse per produkt](../../../docs/compare/index.md)
- [Specifikation](../../../spec/index.md)
- [Hur du lämnar synpunkter](../../../RFC.md)

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/start/`) gäller den engelska versionen.
