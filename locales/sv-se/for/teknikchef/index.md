# För teknikchefen

**Perspektiv:** är det välbyggt, kan mitt team ändra det på ett säkert sätt, och hur vet jag det?

## Vad du får

| Fråga | Vad som finns | Läs |
| --- | --- | --- |
| Teknikstack | Rust + Axum + Axa + PostgreSQL 18 för API:et; SvelteKit 2 med Svelte 5 och TypeScript för klienten | [architecture.md](../../../../spec/architecture.md) |
| Designregel | En ren kärna: reglerna är fria från databas och klocka, testas uttömmande och kopplas sedan in i styrenheterna | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Specifikationen först | En ändring är ett krav, ett beslut, en uppgift, kod och tester tillsammans; varje uppgiftspost säger vad som **inte** har verifierats | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Tester | Enhetstester av reglerna, databasstödda begärantester, körbara filer som prövar tillämpning med inloggning på, enhetstester för gränssnittet och webbläsartester. Mutationskontroller: bryt koden med avsikt och bekräfta att ett test misslyckas | [testing.md](../../../../spec/testing.md) |
| Räknare | Genereras från filerna, kopieras aldrig för hand | [implementation-status.md](../../../../spec/implementation-status.md) |
| Leveranskedja | `cargo deny` för licenser och förbud, versionsfastlåsta delade bibliotek, en programvarustycklista vid utgivning | [production-readiness.md](../../../../spec/production-readiness.md) |
| API-kontrakt | OpenAPI och ett test som misslyckas om en registrerad rutt saknar dokumentation | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Åtkomstkontroll | Policyn avgör enbart efter åtgärd; hanterare och en klassificerad ruttabell avgör behov av kännedom, och en källkodsskanning misslyckas vid en oklassificerad läsrutt | [auth.md](../../../../spec/auth.md) |
| Prestanda | [BENCHMARKS.md](../../../../BENCHMARKS.md) dokumenterar vad som mätts; ett ramverk för distribuerade belastningstester har skapats | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Så bidrar du

Läs först [AGENTS.md](../../../../AGENTS.md): fem regler, sedan en karta efter uppgift.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
