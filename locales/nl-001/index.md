# Personeelsplanning

**Een open-source HR-platform dat de hele levenscyclus van een medewerker dekt, van indiensttreding tot pensioen.**

Personeelsplanning (Workforce Planning Management, WPM) brengt werving, tijd en verlof, medewerkersontwikkeling, personeelsplanning en loonadministratie samen in één applicatie. Het is demonstratiesoftware, bedoeld voor evaluatie en als referentieontwerp.

Taal: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Demonstratiesoftware, nog niet geschikt voor productiegebruik.** WPM gebruikt uitsluitend synthetische gegevens en de wettelijke berekeningen zijn illustratief. Voer geen echte persoonsgegevens in. [De weg naar productie](path-to-production/index.md) beschrijft wat productiegebruik zou vereisen en hoe ver elk onderdeel is. Zie ook [het regelgevende standpunt](../../spec/regulatory.md).

[Demo uitvoeren](start/index.md) [Rolbeschrijvingen](for/index.md) [Broncode op GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Voordelen

HR-gegevens staan vaak verspreid over losse hulpmiddelen, zoals postvakken met sollicitaties, verlofspreadsheets en roosterborden. Elke overdracht daartussen kost handwerk en brengt het risico op fouten met zich mee. WPM is ontworpen om die overdrachten te verminderen.

- **Eén systeem.** Gegevens over indiensttreding, verlof, opleiding en loon staan in één applicatie, zodat er minder opnieuw ingevoerd hoeft te worden.
- **Gegevensminimalisatie.** Gegevens die niet bewaard mogen worden, hebben geen veld, en gegevens die niet bekendgemaakt mogen worden, hebben geen API-route. Er is bijvoorbeeld geen veld voor een diagnose en antwoorden op pulsonderzoeken hebben geen auteur.
- **Rapportageregels.** Onbekende waarden worden niet als nul gerapporteerd, aggregaten maken geen individuen herkenbaar en schattingen vermelden waarop ze berusten.
- **Lokalisatie.** De applicatie is beschikbaar in 17 taalvarianten en de rolbeschrijvingen in 21, waaronder talen die van rechts naar links worden geschreven. Landspecifieke regels worden als gegevens vastgelegd. De vertalingen zijn door AI gemaakt en niet beoordeeld door moedertaalsprekers.
- **Traceerbaarheid.** De broncode is open en elk gedrag is herleidbaar tot een genummerde eis, ontwerpbeslissing en taak.
- **Onafhankelijke inzet.** De stack bestaat uit een Rust-JSON-API, een SvelteKit-client en PostgreSQL. Hij bouwt vanaf een verse kloon en koppelingen met zusterapplicaties zijn optioneel.

## Functies

- **Werving en dossiers.** Vacatures, sollicitatiepijplijn, onboarding, medewerkerdossiers en organigram.
- **Tijd en roostering.** Tijd en verlof, diensten met controles op arbeidstijd, het bereikbaarheidsrooster met ruilen en herinneringen, en een personeelsgids.
- **Ontwikkeling.** Beoordelingen, 360°-feedback, leren, analyse van vaardigheidstekorten, aanbevelingen voor opleidingstijd en verplichte opleidingen.
- **Planning en rapportage.** Strategische personeelsplanning, meetwaarden en inzichten, en een dashboard voor de CEO.
- **In- en uitdienst.** Gedateerde checklists en een overdracht op de laatste dag, met een controlespoor.
- **Communicatie.** Mededelingen met doelgroepen, links en leestellers.
- **Rechten en welzijn van medewerkers.** Welzijnsvragen, een anoniem pulsonderzoek, aanpassingen, inzageverzoeken, wissing en bewaartermijnen.
- **Loon.** Loonadministratie en beloning, met illustratieve wettelijke berekeningen.

## Vergelijking

De tabel vergelijkt WPM met drie categorieën HR-systemen. Het is een algemene oriëntatie, geen benchmark. Zie ook de [vergelijking per product](../../docs/compare/index.md).

| | WPM | Alles-in-één-suites (Workday, SAP SuccessFactors, BambooHR) | Open-source HR (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Puntoplossingen (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Reikwijdte** | Volledige levenscyclus, van indiensttreding tot pensioen | Vergelijkbare reikwijdte op één platform | Dezelfde kernfuncties | Eén functie |
| **Licentie** | Open source | Commercieel | Open source | Commercieel |
| **Belangrijkste sterkte** | Gegevensminimalisatie, traceerbaar ontwerp, analyse van vaardigheidstekorten en opleidingstijd | Eén geïntegreerd, ondersteund platform | Meer koppelingen, lokalisaties en implementaties | Diepgang in één functie |
| **Typische toepassing** | Evaluatie, referentieontwerp of verdere ontwikkeling | Organisaties die een ondersteund systeem nodig hebben | Organisaties die open source in productie willen | Organisaties die diepgang in één functie nodig hebben en de rest kunnen koppelen |

## Doelgroep

- **Bestuurders.** De [rolbeschrijvingen](for/index.md) leggen uit wat WPM voor elke leidinggevende rol biedt en hoe de uitvoer te lezen is.
- **HR-teams.** De [demo-handleiding](start/index.md) behandelt elke pagina aan de hand van fictieve gegevens.
- **Ontwikkelaars en architecten.** Zie de [specificatie](../../spec/index.md) en de [installatiehandleiding](../../INSTALL.md).
- **Beoordelaars.** De [gereedheidsbeoordeling](../../spec/readiness-assessment.md) somt de hiaten op tussen WPM en productiegebruik.

## Hulp krijgen

- **Meld een bug of stel een vraag** in de [issuetracker](https://github.com/workforce-planning-management/workforce-planning-management/issues). Vermeld de pagina, wat u deed, wat u verwachtte en wat er gebeurde. Voeg geen echte persoonsgegevens toe.
- **Meld een beveiligingsprobleem** per e-mail, zoals beschreven in het [beveiligingsbeleid](../../SECURITY.md), niet in een openbaar issue.
- **Leer een taak in de applicatie.** De link **Help** in de kop van de applicatie opent de helppagina en de stappenplannen voor medewerkers, leidinggevenden en HR. Elke foutmelding verwijst er ook naartoe.

## Volgende stappen

1. **Lees de beschrijving voor uw rol.** Elke beschrijving legt uit wat WPM biedt en wat nog openstaat.
2. **Voer de demo uit.** De rondleiding draait op een lokale machine met fictieve gegevens.
3. **Geef feedback.** Zie [hoe u kunt reageren](../../RFC.md) en [hoe u kunt bijdragen](../../CONTRIBUTING.md).

[Demo uitvoeren](start/index.md) [Documentatie](../../index.md)

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/`), dan geldt de Engelse versie.
