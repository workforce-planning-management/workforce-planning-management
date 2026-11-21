# Til teknologidirektøren

**Perspektiv:** er det bygget godt, kan mit team ændre det sikkert, og hvordan ved jeg det?

## Hvad du får

| Emne | Hvad der findes | Læs |
| --- | --- | --- |
| Teknologistak | Rust + Axum + Axa + PostgreSQL 18 til API'et; SvelteKit 2 med Svelte 5 og TypeScript til klienten | [architecture.md](../../../../spec/architecture.md) |
| Designregel | En ren kerne: regler er uden database og uden ur, testes udtømmende og kobles først derefter på controllerne | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Specifikation først | En ændring er et krav, en beslutning, en opgave, kode og tests sammen; hver opgave siger, hvad der **ikke** er verificeret | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Tests | Enhedstests af reglerne, anmodningstests med database, kørsel af håndhævende programmer med login slået til, enhedstests af frontenden og browsertests. Mutationstjek: bryd koden med vilje og bekræft, at en test fejler | [testing.md](../../../../spec/testing.md) |
| Tal | Genereret fra filerne, aldrig kopieret i hånden | [implementation-status.md](../../../../spec/implementation-status.md) |
| Forsyningskæde | `cargo deny` til licenser og forbud, delte crates med fastlåst version, en stykliste over software ved udgivelse | [production-readiness.md](../../../../spec/production-readiness.md) |
| API-kontrakt | OpenAPI og en test, der fejler, hvis en registreret route ikke er dokumenteret | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Adgangsstyring | Politikken afgør kun ud fra handlingen; handlere og en klassificeret routetabel afgør behov for at vide, og en kildeskanning fejler ved en uklassificeret læseroute | [auth.md](../../../../spec/auth.md) |
| Ydeevne | [BENCHMARKS.md](../../../../BENCHMARKS.md) registrerer, hvad der blev målt; et framework til distribueret belastningstest er oprettet | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Sådan bidrager du

Læs først [AGENTS.md](../../../../AGENTS.md): fem regler og derefter et kort over opgaver.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
