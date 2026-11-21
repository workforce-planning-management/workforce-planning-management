# Voor de risicodirecteur (CRO)

**Perspectief:** wat kan er met dit systeem misgaan, wat houdt het tegen en wat heeft niemand gecontroleerd?

## Het eerlijke uitgangspunt

De [gereedheidsbeoordeling](../../../../spec/readiness-assessment.md) van 2026-10-11 geeft het oordeel **niet gereed** voor een inzet met hoge zekerheidseisen. Door de draaiende service te onderzoeken werd één blokkade gevonden: elke aangemelde persoon kon het ziekteverlof van een collega en de vrije tekst van de reden lezen. Dat is verholpen en getest. De poorten die alleen mensen kunnen sluiten, staan nog open.

## Wat u krijgt

| Aandachtspunt | Wat er is | Lees |
| --- | --- | --- |
| Bedreigingen op elke grens | Een dreigingsmodel: bedrijfsmiddelen, actoren, grenzen, de beheersmaatregel zoals gebouwd en wat **niet is beoordeeld** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Ongeautoriseerd lezen | Een need-to-know-bewaker die elke leesroute classificeert; een test faalt bij een niet-geclassificeerde route | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Manipulatie en verantwoording | Een controlespoor dat alleen kan worden aangevuld en met hashes is geketend, met een verificatie-eindpunt; items bevatten geen redentekst en geen waarde | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Gevoelige gegevens | Geen diagnose bewaard; functies voor bijzondere categorieën standaard uit; een ondergrens voor kleine groepen | [regulatory.md](../../../../spec/regulatory.md) |
| Onveilige wegen | Productie weigert te starten als aanmelden uit staat, er geen bron voor tokensleutels is, een niet-versleutelde databaseverbinding naar een andere host loopt of een niet-versleutelde externe Redis wordt gebruikt | [production-readiness.md](../../../../spec/production-readiness.md) |
| Gegevensverlies | Versleutelde back-ups, een hersteloefening die in CI draait, een wissingsregister | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Vooringenomenheid bij scoring | Een sjabloon voor gelijkheidseffecten en een register van elke score met een conceptbeoordeling | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Een zelfbeoordeling van beheersmaatregelen | Een checklist waarin elk antwoord voldaan, gedeeltelijk voldaan, niet voldaan of door de inzetter te beantwoorden is | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Openstaande risico's, zoals vastgelegd

Het vastzetten en ondertekenen van de toeleveringsketen, bewijs van belasting, logcontroles op persoonsgegevens, gedeelde aanvraaglimieten over instanties heen, een databaserolset met minimale rechten en richtlijnen voor het intrekken van tokens zijn openstaande punten in de gereedheidsbeoordeling, elk met een taak. De tests zijn door de auteur geschreven; geen enkele is een onafhankelijke beoordeling.

## Wat alleen mensen kunnen doen

Een onafhankelijke penetratietest, een onafhankelijke toegankelijkheidsaudit, de goedgekeurde effectbeoordeling van de inzetter, een hersteloefening op uw platform en goedkeuring van de beoordelingen van gelijkheidseffecten. De repository noemt deze als poorten en markeert ze niet als gereed.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/for/`), dan geldt de Engelse versie.
