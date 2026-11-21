# Voor de strategiedirecteur (CSO)

**Perspectief:** is ons personeelsbestand zo gevormd dat het de strategie dient, wat hebben we nodig en wat moeten we aan het tekort doen?

## Wat u krijgt

| U wilt weten | Waar WPM antwoord geeft | Lees |
| --- | --- | --- |
| Wat een rol vereist | Functieprofielen met vereiste vaardigheden en een minimaal niveau, zodat een tekort wordt afgemeten aan een rol en niet aan een persoon | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Wat we nodig zullen hebben | Personeelsplannen als conceptwerelden, meerdere naast elkaar, met vermelde aannames en vraag per afdeling, rol en periode | WPM-R36, hetzelfde bestand |
| Waar het tekort zit | Geprojecteerd aanbod tegenover geplande vraag: een tekort in personeelsomvang en een competentietekort, met hefbomen als suggesties en hun onderbouwing | WPM-R37 |
| Of het de strategie dient | Doelstellingen gekoppeld aan vraag: doelstellingen zonder middelen, niet-afgestemde vraag, kritieke rollen zonder reserve | WPM-R38 |
| Of we een programma kunnen leveren | Vaardigheidspools, programmavraag, toezeggingen van partners, de beperkende pool, een startcontrole die een suggestie is | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Hoezeer we op niet-vast personeel steunen | Het aandeel fte dat inhuur en dat voor bepaalde tijd is, elk apart, en hoeveel financiering in elk van de komende twaalf maanden afloopt | `GET /api/workforce-intelligence/contingent` (WPM-T109, deels) |
| Wat we plannen te doen met aflopende contracten | Omzettingsplannen: de beslissing van een persoon om om te zetten, te verlengen of te beëindigen, met de financiering naast het einde van het contract | [engagements.md](../../../../spec/engagements.md) |
| Wat er verder gepland is | De routekaart en de lijst met openstaande taken | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Hoe u een prognose leest

- Het is een vermelde regel toegepast op vermelde invoer, geen model, en elk resultaat noemt zijn aannames.
- Bij te weinig geschiedenis van de personeelsomvang zegt het `insufficient_history` in plaats van te extrapoleren.
- Een plan bevat uitsluitend hypothetische personeelsomvang. Het leest of wijzigt nooit het dossier van een persoon.
- Hefbomen zijn suggesties. De planner kiest, en de software besluit niets.

## Beperkingen

- De hefboom "flexibel omzetten naar vast" in plannen is gespecificeerd (WPM-R63) en nog niet in code.
- Het flexibele aandeel heeft nog geen trend: daarvoor zijn momentopnamen per grondslag nodig (WPM-T139, niet gebouwd).
- Planning met kritieke keten en kritiek pad hoort bij project-portfolio-management, niet hier.
- Er is nog geen gebruikersinterface voor financiering of omzettingsplannen.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/for/`), dan geldt de Engelse versie.
