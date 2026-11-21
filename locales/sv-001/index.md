# Arbetskraftsplanering

**En HR-plattform med öppen källkod som täcker medarbetarens hela livscykel, från anställning till pension.**

Arbetskraftsplanering (Workforce Planning Management, WPM) samlar rekrytering, tid och ledighet, medarbetarutveckling, arbetskraftsplanering och lönehantering i en enda applikation. Det är demonstrationsprogramvara, avsedd för utvärdering och som referensdesign.

Språk: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Demonstrationsprogramvara, ännu inte redo för produktionsbruk.** WPM använder endast syntetiska data, och dess lagstadgade beräkningar är exempel. Ange inga verkliga personuppgifter. [Vägen till produktion](path-to-production/index.md) beskriver vad produktionsbruk skulle kräva och hur långt varje del har kommit. Se även [den regulatoriska ståndpunkten](../../spec/regulatory.md).

[Kör demon](start/index.md) [Rollbeskrivningar](for/index.md) [Källkod på GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Fördelar

HR-data ligger ofta i separata verktyg, som ansökningsinkorgar, kalkylblad för ledighet och bemanningstavlor. Varje överföring mellan dem ger manuellt arbete och risk för fel. WPM är utformat för att minska antalet överföringar.

- **Ett enda system.** Introduktion, ledighet, utbildning och lönedata finns i en applikation, vilket minskar dubbelregistrering.
- **Dataminimering.** Data som inte ska lagras har inget fält, och data som inte ska lämnas ut har ingen API-rutt. Det finns till exempel inget diagnosfält, och svar i pulsenkäter har ingen avsändare.
- **Rapporteringsregler.** Okända värden rapporteras inte som noll, sammanställningar identifierar inte enskilda personer och uppskattningar anger sin grund.
- **Lokalisering.** Applikationen finns på 17 språkvarianter och rollbeskrivningarna på 21, inklusive språk som skrivs från höger till vänster. Landsspecifika regler lagras som data. Översättningarna är gjorda av AI och har inte granskats av personer med språket som modersmål.
- **Spårbarhet.** Källkoden är öppen, och varje beteende kan spåras till ett numrerat krav, ett designbeslut och en uppgift.
- **Oberoende driftsättning.** Stacken är ett JSON-API i Rust, en SvelteKit-klient och PostgreSQL. Den byggs från en ny klon, och integrationer med systerapplikationer är valfria.

## Funktioner

- **Rekrytering och register.** Tjänstebehov, ansökningsflöde, introduktion, personalregister och organisationsschema.
- **Tid och schemaläggning.** Tid och ledighet, skift med kontroll av arbetstid, jourschema med byten och påminnelser samt en personalkatalog.
- **Utveckling.** Medarbetarsamtal, 360°-återkoppling, lärande, kompetensgapsanalys, rekommendationer om utbildningstid och obligatorisk utbildning.
- **Planering och rapportering.** Strategisk arbetskraftsplanering, nyckeltal och insikter samt en vd-översikt.
- **Nyanställda och avgående.** Daterade checklistor och en överlämning på sista dagen med spårbarhet.
- **Kommunikation.** Meddelanden med målgrupper, länkar och läsantal.
- **Medarbetares rättigheter och välbefinnande.** Frågor om välbefinnande, en anonym pulsenkät, anpassningar, registerutdrag, radering och gallring.
- **Lön.** Löne- och ersättningshantering med lagstadgade beräkningar som exempel.

## Jämförelse

Tabellen jämför WPM med tre kategorier av HR-system. Det är en allmän orientering, inte ett prestandatest. Se även [jämförelsen per produkt](../../docs/compare/index.md).

| | WPM | Heltäckande sviter (Workday, SAP SuccessFactors, BambooHR) | HR med öppen källkod (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Specialverktyg (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Omfattning** | Hela livscykeln, från anställning till pension | Liknande omfattning på en plattform | Samma kärnfunktioner | En funktion |
| **Licensiering** | Öppen källkod | Kommersiell | Öppen källkod | Kommersiell |
| **Största styrka** | Dataminimering, spårbar design, analys av kompetensgap och utbildningstid | En integrerad plattform med support | Fler integrationer, lokaliseringar och driftsättningar | Djup i en funktion |
| **Typisk användning** | Utvärdering, referensdesign eller vidareutveckling | Organisationer som behöver ett system med support | Organisationer som vill ha öppen källkod i produktion | Organisationer som behöver djup i en funktion och kan integrera resten |

## Målgrupp

- **Ledare.** [Rollbeskrivningarna](for/index.md) beskriver vad WPM erbjuder för varje ledningsroll och hur man läser resultaten.
- **HR-team.** [Demonstrationsguiden](start/index.md) går igenom varje sida med fiktiva data.
- **Utvecklare och arkitekter.** Se [specifikationen](../../spec/index.md) och [installationsguiden](../../INSTALL.md).
- **Bedömare.** [Beredskapsbedömningen](../../spec/readiness-assessment.md) förtecknar luckorna mellan WPM och produktionsbruk.

## Få hjälp

- **Rapportera ett fel eller ställ en fråga** i [ärendehanteringen](https://github.com/workforce-planning-management/workforce-planning-management/issues). Ange sidan, vad du gjorde, vad du förväntade dig och vad som hände. Ta inte med verkliga personuppgifter.
- **Rapportera ett säkerhetsproblem** via e-post, enligt [säkerhetspolicyn](../../SECURITY.md), inte i ett offentligt ärende.
- **Lär dig en uppgift i applikationen.** Länken **Hjälp** i applikationens sidhuvud öppnar hjälpsidan och genomgångarna för medarbetare, chefer och HR. Varje felmeddelande länkar också dit.

## Nästa steg

1. **Läs rollbeskrivningen för din roll.** Varje beskrivning anger vad WPM erbjuder och vad som återstår.
2. **Kör demon.** Rundturen körs på en lokal maskin med fiktiva data.
3. **Skicka återkoppling.** Se [hur du lämnar synpunkter](../../RFC.md) och [hur du bidrar](../../CONTRIBUTING.md).

[Kör demon](start/index.md) [Dokumentation](../../index.md)

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/`) gäller den engelska versionen.
