# Demo-Tutorial

Dieses Tutorial beschreibt, wie Sie WPM auf einem lokalen Rechner starten, fiktive Daten laden und der geführten Tour durch jede Seite folgen.

> **Nur fiktive Daten.** Die Anmeldung ist für die Tour abgeschaltet. Verbinden Sie sie nicht mit einem Dienst, der echte Daten enthält.

## Voraussetzungen

- [Rust](https://www.rust-lang.org/tools/install), stable
- [Node.js](https://nodejs.org/) und [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), mit einer leeren Datenbank für die Tour
- Python 3

Die [Installationsanleitung](../../../INSTALL.md) nennt die erforderlichen Versionen und die Datenbankeinstellungen.

## Schritt 1: Den Code holen

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Schritt 2: Den Dienst starten

Der Dienst liest `DATABASE_URL`. Der Standardwert ist `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; setzen Sie die Variable, wenn Ihre Datenbank einen anderen Namen, Benutzer oder Port hat. Bereiten Sie die leere Datenbank vor und starten Sie die API. Die Anmeldung ist nur für diese Instanz abgeschaltet, und das Ratenlimit ist für die Tour erhöht.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Schritt 3: Die fiktiven Daten laden

Führen Sie in einem zweiten Terminal, vom Stammverzeichnis des Repositorys aus, Folgendes aus:

```sh
workforce-demo-data http://localhost:1241
```

## Schritt 4: Die Anwendung starten

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Schritt 5: Der Tour folgen

Öffnen Sie die von der Anwendung angezeigte Adresse und melden Sie sich als **example@example.com** an. Es wird keine E-Mail versendet. Die Anmeldung öffnet `/demo`, eine Tour durch jede Seite. Ein Banner auf jeder Seite weist darauf hin, dass die Daten fiktiv sind, und bietet die Links Zurück und Weiter.

## Weiterführende Lektüre

- [Rollenbeschreibungen](../for/index.md)
- [Vergleich nach Produkt](../../../docs/compare/index.md)
- [Spezifikation](../../../spec/index.md)
- [Kommentieren](../../../RFC.md)

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-001/start/`).
