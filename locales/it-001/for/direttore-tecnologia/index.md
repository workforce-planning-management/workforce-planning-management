# Per il direttore della tecnologia

**Prospettiva:** è costruito bene, il mio team può modificarlo in sicurezza e come faccio a saperlo?

## Che cosa ottieni

| Esigenza | Che cosa esiste | Leggi |
| --- | --- | --- |
| Stack | Rust + Axum + Axa + PostgreSQL 18 per l'API; SvelteKit 2 con Svelte 5 e TypeScript per il client | [architecture.md](../../../../spec/architecture.md) |
| Regola di progetto | Un nucleo puro: le regole non dipendono dal database né dall'orologio, sono testate in modo esaustivo e poi collegate ai controller | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Prima la specifica | Una modifica è un requisito, una decisione, un'attività, codice e test insieme; ogni voce di attività dice che cosa **non** è stato verificato | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Test | Test unitari sulle regole, test delle richieste con il database, programmi di verifica dell'applicazione eseguiti con l'accesso attivo, test unitari del front-end e test nel browser. Controlli per mutazione: rompi il codice di proposito e conferma che un test fallisca | [testing.md](../../../../spec/testing.md) |
| Conteggi | Generati dai file, mai copiati a mano | [implementation-status.md](../../../../spec/implementation-status.md) |
| Catena di fornitura | `cargo deny` per licenze e divieti, crate condivisi con versione fissata, una distinta dei componenti software a ogni versione | [production-readiness.md](../../../../spec/production-readiness.md) |
| Contratto dell'API | OpenAPI e un test che fallisce se una rotta registrata non è documentata | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Controllo degli accessi | La politica decide solo in base all'azione; i gestori e una tabella delle rotte classificate decidono la necessità di sapere e una scansione del sorgente fallisce su una rotta di lettura non classificata | [auth.md](../../../../spec/auth.md) |
| Prestazioni | [BENCHMARKS.md](../../../../BENCHMARKS.md) registra ciò che è stato misurato; creato un framework per test di carico distribuito | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Come contribuire

Leggi prima [AGENTS.md](../../../../AGENTS.md): cinque regole, poi una mappa per attività.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
