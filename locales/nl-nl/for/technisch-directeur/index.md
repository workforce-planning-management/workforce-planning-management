# Voor de technisch directeur (CTO)

**Perspectief:** is het goed gebouwd, kan mijn team het veilig wijzigen en hoe weet ik dat?

## Wat u krijgt

| Aandachtspunt | Wat er is | Lees |
| --- | --- | --- |
| Stack | Rust + Axum + Axa + PostgreSQL 18 voor de API; SvelteKit 2 met Svelte 5 en TypeScript voor de client | [architecture.md](../../../../spec/architecture.md) |
| Ontwerpregel | Een pure kern: regels zijn vrij van database en klok, uitputtend getest en daarna in controllers aangesloten | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Eerst de specificatie | Een wijziging is samen een eis, een beslissing, een taak, code en tests; elke taak zegt wat **niet** is geverifieerd | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Tests | Eenheidstests op de regels, verzoektests met database, afdwingingsprogramma's die met aanmelden aan draaien, front-endeenheidstests en browsertests. Mutatiecontroles: breek de code met opzet en controleer dat een test faalt | [testing.md](../../../../spec/testing.md) |
| Aantallen | Gegenereerd uit de bestanden, nooit met de hand overgenomen | [implementation-status.md](../../../../spec/implementation-status.md) |
| Toeleveringsketen | `cargo deny` voor licenties en verboden, op versie vastgezette gedeelde crates, een software bill of materials bij elke release | [production-readiness.md](../../../../spec/production-readiness.md) |
| API-contract | OpenAPI, en een test die faalt als een geregistreerde route niet is gedocumenteerd | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Toegangsbeheer | Het beleid beslist uitsluitend op actie; handlers en een geclassificeerde routetabel beslissen over need-to-know, en een bronscan faalt bij een niet-geclassificeerde leesroute | [auth.md](../../../../spec/auth.md) |
| Prestaties | [BENCHMARKS.md](../../../../BENCHMARKS.md) legt vast wat is gemeten; een raamwerk voor gedistribueerde belastingtests is gemaakt | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Hoe u kunt bijdragen

Lees eerst [AGENTS.md](../../../../AGENTS.md): vijf regels, daarna een kaart per taak.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/for/`), dan geldt de Engelse versie.
