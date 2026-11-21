# Voor de operationeel directeur (COO)

**Perspectief:** hebben we de juiste mensen, op de juiste plaats, op het juiste moment, en wat valt om als iemand afwezig is?

## Wat u krijgt

| U wilt weten | Waar WPM antwoord geeft | Lees |
| --- | --- | --- |
| Of een team afwezigheid kan opvangen | Vervangers per persoon, het bereikbaarheidsrooster met ruilen en herinneringen, en een dekkingsoverzicht dat zonder omwegen zegt wanneer niemand beschikbaar is | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Tijd, aanwezigheid en verlof | Tijdregistraties, verlofaanvragen met een goedkeuringsroute, opbouw en diensten | [workforce-management.md](../../../../spec/workforce-management.md) |
| Of de grenzen aan arbeidstijd worden gerespecteerd | Adviserende vangrails voor arbeidstijd; ze waarschuwen en blokkeren niet | WPM-R27, hetzelfde bestand |
| Capaciteit per vaardigheid, maand voor maand | Vaardigheidspools, aanbod en programmavraag, toezeggingen van partners, de beperkende pool en een startcontrole die een suggestie is, met vastgelegde startbeslissingen | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Wat we nodig zullen hebben en waar het tekort zit | Personeelsplannen, prognose en tekortanalyse per afdeling en per rol | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Welke contracten binnenkort aflopen | Een einddatum per contractgrondslag, verlengingen met geschiedenis, herinneringen vóór een einde, een lijst van overeenkomsten zonder einddatum en omzettingsplannen | [engagements.md](../../../../spec/engagements.md) |
| Waar flexibele krachten een groot aandeel vormen | Het aandeel fte dat inhuur is en dat voor bepaalde tijd is, elk apart, en de financiering die maand voor maand afloopt | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| In- en uitdienst | Gedateerde checklists en een overdracht op de laatste dag | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Vaardigheidstekorten en opleidingstijd | Tekortanalyse met het bewijs waarop een aanbeveling berust | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Hoe u het capaciteitsoverzicht leest

Capaciteit wordt per vaardigheidspool gepland, niet per persoon. Een poolmaand levert een van vier uitkomsten op, en een niet opgegeven vaardigheid is *onbekend*, geen tekort. De startcontrole is een suggestie en een persoon legt de beslissing om te starten vast. Planning met kritieke keten en kritiek pad zit hier niet: die hoort bij project-portfolio-management.

## Beperkingen

- Het flexibele aandeel heeft nog geen trend: daarvoor zijn momentopnamen van de personeelsomvang per grondslag nodig (WPM-T139, niet gebouwd).
- Een pool voor het overzicht van flexibele krachten is de afdeling, totdat plannen pools definiëren.
- Dienstroosters en berekeningen van loonbelasting vervangen uw rooster- of loonsystemen niet.
- Verschillende overzichten laden de dossiers van een organisatie in het geheugen; er is geen belastingcijfer gemeten ([readiness-assessment.md](../../../../spec/readiness-assessment.md), hiaat 8).

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/for/`), dan geldt de Engelse versie.
