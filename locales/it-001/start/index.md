# Tutorial della demo

Questo tutorial descrive come eseguire WPM su un computer locale, caricare dati fittizi e seguire il percorso guidato di ogni pagina.

> **Solo dati fittizi.** L'accesso è disattivato per il percorso guidato. Non collegarlo a un servizio che contiene dati reali.

## Prerequisiti

- [Rust](https://www.rust-lang.org/tools/install), versione stabile
- [Node.js](https://nodejs.org/) e [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), con un database vuoto per il percorso guidato
- Python 3

La [guida all'installazione](../../../INSTALL.md) indica le versioni richieste e le impostazioni del database.

## Passo 1: ottenere il codice

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Passo 2: avviare il servizio

Il servizio legge `DATABASE_URL`. Il valore predefinito è `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; imposta la variabile se il tuo database ha un altro nome, utente o porta. Prepara il database vuoto e avvia l'API. L'accesso è disattivato solo per questa istanza e il limite di richieste è aumentato per il percorso guidato.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Passo 3: caricare i dati fittizi

In un secondo terminale, dalla radice del repository:

```sh
workforce-demo-data http://localhost:1241
```

## Passo 4: avviare l'applicazione

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Passo 5: seguire il percorso guidato

Apri l'indirizzo mostrato dall'applicazione e accedi come **example@example.com**. Non viene inviata alcuna email. L'accesso apre `/demo`, un percorso guidato che attraversa ogni pagina. Un banner su ogni pagina precisa che i dati sono fittizi e offre i collegamenti Precedente e Successivo.

## Letture di approfondimento

- [Schede per ruolo](../for/index.md)
- [Confronto per prodotto](../../../docs/compare/index.md)
- [Specifica](../../../spec/index.md)
- [Come commentare](../../../RFC.md)

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/start/`), prevale la versione inglese.
