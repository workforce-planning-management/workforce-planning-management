# Voor de directeur informatiebeveiliging (CISO)

**Perspectief:** hoe is dit beschermd, hoe zouden we merken dat het wordt aangevallen of misbruikt, en wat heeft niemand getest?

## Wat u krijgt

| Aandachtspunt | Wat er is | Lees |
| --- | --- | --- |
| Bedreigingen | Een dreigingsmodel: bedrijfsmiddelen, actoren, vertrouwensgrenzen, de beheersmaatregel zoals gebouwd en wat **niet is beoordeeld** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Authenticatie | Aanmelden standaard aan; productie weigert te starten als het uit staat of er geen bron voor tokensleutels is; aanmelden via de bedrijfsmap (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Autorisatie | Het beleid beslist uitsluitend op actie; een geclassificeerde tabel beslist over need-to-know op elke leesroute, en een bronscantest faalt bij een niet-geclassificeerde route | [auth.md](../../../../spec/auth.md) |
| Controlespoor | Een spoor dat alleen kan worden aangevuld en met hashes is geketend, met een verificatie-eindpunt; items bevatten geen redentekst en geen waarde | [audit.md](../../../../spec/audit.md) |
| Transport en browser | Beveiligingskoppen bij elk API-antwoord, een expliciete CORS-toelatingslijst waarmee het opstarten faalt als die ontbreekt, `no-store` op `/api` | WPM-R73 in [production-readiness.md](../../../../spec/production-readiness.md) |
| Misbruik | Aanvraaglimieten per aanroeper, strenger voor wissen, importeren, exporteren en tokenroutes | WPM-R74, hetzelfde bestand |
| Onveilige configuratie | Het opstarten weigert een niet-versleutelde databaseverbinding of niet-versleutelde Redis naar een andere host, en een ontbrekende CORS-oorsprong | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Gegevens in rust | Versleutelde back-ups; de gedeelde cache bevat alleen niet-persoonlijke opzoekingen en valt open bij een storing | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Toeleveringsketen | Controles op licenties en verboden met `cargo deny`, op versie vastgezette gedeelde crates, een software bill of materials bij elke release | [production-readiness.md](../../../../spec/production-readiness.md) |
| Incidenten | Een draaiboek met eerste handelingen; u voegt contactpersonen en meldplichten toe | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Zelfbeoordeling | Een checklist van beheersmaatregelen: voldaan, gedeeltelijk voldaan, niet voldaan, door de inzetter te beantwoorden | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Bekende hiaten, zoals vastgelegd

- Er is geen onafhankelijke penetratietest uitgevoerd. De tests zijn door de auteur geschreven.
- Acties zijn vastgezet op tag, niet op digest, en images zijn niet ondertekend.
- Aanvraaglimieten gelden per instantie.
- De applicatie gebruikt elke databaserol die zij krijgt; er is geen rolset met minimale rechten meegeleverd.
- Intrekking van tokens en de maximale leeftijd van een token zijn aan uw identiteitsprovider.
- Het pad van de inhoudsbeheerder laadt een script van een openbare CDN zonder vastgezette versie of integriteitshash.
- Logs zijn niet gecontroleerd op persoonsgegevens en het controlespoor wordt niet naar een monitoringsysteem geëxporteerd.
- Een beheerde Redis (cluster, sentinel, gebruikers met toegangsbeheer) is niet geprobeerd.

## Wat alleen mensen kunnen doen

De onafhankelijke penetratietest in opdracht geven, het dreigingsmodel voor uw omgeving beoordelen, een hersteloefening op uw platform uitvoeren en branchebescherming instellen op het hostingplatform. Zie de [gereedheidsbeoordeling](../../../../spec/readiness-assessment.md), waarvan het oordeel **niet gereed** is voor een inzet met hoge zekerheidseisen.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/for/`), dan geldt de Engelse versie.
