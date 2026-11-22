# Planlægning af arbejdsstyrken

**En open source HR-platform, der dækker medarbejderens hele livscyklus, fra ansættelse til pension.**

Planlægning af arbejdsstyrken (Workforce Planning Management, WPM) samler rekruttering, tid og fravær, medarbejderudvikling, arbejdsstyrkeplanlægning og løn i én applikation. Det er demonstrationssoftware, beregnet til evaluering og som referencedesign.

Sprog: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Demonstrationssoftware, endnu ikke klar til produktion.** WPM bruger kun syntetiske data, og de lovbestemte beregninger er eksempler. Indtast ikke rigtige personoplysninger. [Vejen til produktion](path-to-production/index.md) beskriver, hvad produktionsbrug ville kræve, og hvor langt hver del er nået. Se også [den regulatoriske position](../../spec/regulatory.md).

[Kør demoen](start/index.md) [Rollebeskrivelser](for/index.md) [Kildekode på GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Fordele

HR-data ligger ofte i adskilte værktøjer, såsom ansøgerindbakker, regneark til fravær og vagtplaner. Hver overførsel mellem dem giver manuelt arbejde og risiko for fejl. WPM er designet til at mindske disse overførsler.

- **Ét system.** Onboarding, fravær, uddannelse og lønoplysninger ligger i én applikation, hvilket mindsker genindtastning.
- **Dataminimering.** Data, der ikke bør opbevares, har intet felt, og data, der ikke bør udleveres, har ingen API-rute. Der er for eksempel intet felt til diagnose, og besvarelser af pulsmålinger har ingen afsender.
- **Regler for rapportering.** Ukendte værdier rapporteres ikke som nul, sammentællinger identificerer ikke enkeltpersoner, og skøn oplyser, hvad de bygger på.
- **Lokalisering.** Applikationen findes på 25 lokaliteter og rollebeskrivelserne på 21, herunder sprog, der skrives fra højre mod venstre. Landespecifikke regler opbevares som data. Oversættelserne er lavet af AI og er ikke gennemgået af personer med sproget som modersmål.
- **Sporbarhed.** Kildekoden er åben, og hver adfærd kan spores til et nummereret krav, en designbeslutning og en opgave.
- **Uafhængig drift.** Teknologistakken er et JSON-API i Rust, en SvelteKit-klient og PostgreSQL. Den kan bygges fra en frisk klon, og integrationer med søsterapplikationer er valgfri.

## Funktioner

- **Rekruttering og personaleoplysninger.** Stillingsopslag, ansøgerforløb, onboarding, medarbejderposter og organisationsdiagram.
- **Tid og planlægning.** Tid og fravær, vagter med kontrol af arbejdstid, vagtplan for rådighedsvagter med bytte og påmindelser samt en medarbejderfortegnelse.
- **Udvikling.** Medarbejdersamtaler, 360°-feedback, læring, analyse af kompetencegab, anbefalinger om uddannelsestid og obligatorisk læring.
- **Planlægning og rapportering.** Strategisk arbejdsstyrkeplanlægning, nøgletal og indsigter samt et dashboard til direktionen.
- **Nye og fratrædende.** Daterede tjeklister og en overdragelse på sidste arbejdsdag med revisionsspor.
- **Kommunikation.** Meddelelser med målgrupper, links og antal læsninger.
- **Medarbejderes rettigheder og trivsel.** Trivselsspørgsmål, en anonym pulsmåling, tilpasninger, indsigtsret, sletning og opbevaring.
- **Løn.** Løn og kompensation med illustrative lovbestemte beregninger.

## Sammenligning

Tabellen sammenligner WPM med tre kategorier af HR-systemer. Den er en generel orientering og ikke en benchmark. Se også [sammenligningen produkt for produkt](../../docs/compare/index.md).

| | WPM | Samlede suiter (Workday, SAP SuccessFactors, BambooHR) | Open source HR (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Specialværktøjer (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Omfang** | Hele livscyklussen, fra ansættelse til pension | Tilsvarende omfang på én platform | De samme kernefunktioner | Én funktion |
| **Licens** | Open source | Kommerciel | Open source | Kommerciel |
| **Største styrke** | Dataminimering, sporbart design, analyse af kompetencegab og uddannelsestid | Én integreret og understøttet platform | Flere integrationer, lokaliseringer og driftsmiljøer | Dybde i én funktion |
| **Typisk anvendelse** | Evaluering, referencedesign eller videreudvikling | Organisationer, der har brug for et understøttet system | Organisationer, der vil have open source i produktion | Organisationer, der har brug for dybde i én funktion og selv kan integrere resten |

## Målgruppe

- **Direktionen.** [Rollebeskrivelserne](for/index.md) viser, hvad WPM giver til hver ledelsesrolle, og hvordan man læser dets resultater.
- **HR-teams.** [Demovejledningen](start/index.md) gennemgår hver side med fiktive data.
- **Udviklere og arkitekter.** Se [specifikationen](../../spec/index.md) og [installationsvejledningen](../../INSTALL.md).
- **Vurderingspersoner.** [Parathedsvurderingen](../../spec/readiness-assessment.md) opregner forskellene mellem WPM og produktionsbrug.

## Hjælp

- **Meld en fejl eller stil et spørgsmål** i [sagsstyringen](https://github.com/workforce-planning-management/workforce-planning-management/issues). Angiv siden, hvad du gjorde, hvad du forventede, og hvad der skete. Medtag ikke rigtige personoplysninger.
- **Meld et sikkerhedsproblem** pr. e-mail, som beskrevet i [sikkerhedspolitikken](../../SECURITY.md), og ikke i en offentlig sag.
- **Lær en opgave i applikationen.** Linket **Hjælp** i applikationens sidehoved åbner hjælpesiden og gennemgangene for medarbejdere, ledere og HR. Hver fejlmeddelelse linker også dertil.

## Næste skridt

1. **Læs beskrivelsen for din rolle.** Hver beskrivelse forklarer, hvad WPM giver, og hvad der stadig er åbent.
2. **Kør demoen.** Rundturen kører på en lokal maskine med fiktive data.
3. **Send feedback.** Se [hvordan du kommenterer](../../RFC.md) og [hvordan du bidrager](../../CONTRIBUTING.md).

[Kør demoen](start/index.md) [Dokumentation](../../index.md)

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/`), er det den engelske version, der gælder.
