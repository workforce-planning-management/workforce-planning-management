# Til informationssikkerhedschefen

**Perspektiv:** hvordan er systemet beskyttet, hvordan ville vi opdage, hvis det blev angrebet eller misbrugt, og
hvad har ingen testet?

## Hvad du får

| Emne | Hvad der findes | Læs |
| --- | --- | --- |
| Trusler | En trusselsmodel: aktiver, aktører, tillidsgrænser, kontrollerne som bygget og det, der **ikke er vurderet** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Autentificering | Login slået til som standard; produktion nægter at starte, hvis det er slået fra, eller hvis der ingen kilde er til tokennøgler; login via virksomhedens kataloge (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Autorisation | Politikken afgør kun ud fra handlingen; en klassificeret tabel afgør behov for at vide på hver læseroute, og en test, der skanner kildekoden, fejler ved en uklassificeret route | [auth.md](../../../../spec/auth.md) |
| Revision | Et spor, der kun kan føjes til, kædet med hashværdier, med et slutpunkt til verificering; poster har ingen begrundelsestekst og ingen værdi | [audit.md](../../../../spec/audit.md) |
| Transport og browser | Sikkerhedshoveder på hvert API-svar, en eksplicit CORS-positivliste, som får opstarten til at fejle, når den mangler, og `no-store` på `/api` | WPM-R73 i [production-readiness.md](../../../../spec/production-readiness.md) |
| Misbrug | Hastighedsgrænser pr. kalder, strengere for sletning, import, eksport og tokenruter | WPM-R74, samme fil |
| Usikker konfiguration | Opstart afviser en databaseforbindelse uden kryptering eller Redis uden kryptering til en anden vært samt en manglende CORS-oprindelse | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Data i hvile | Krypterede sikkerhedskopier; den delte cache rummer kun opslag uden personoplysninger og fejler åbent | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Forsyningskæde | Kontrol af licenser og forbudte pakker med `cargo deny`, delte crates med fastlåst version, en stykliste over software ved udgivelse | [production-readiness.md](../../../../spec/production-readiness.md) |
| Hændelser | En driftsvejledning med de første handlinger; du tilføjer kontaktpersoner og pligter til at underrette | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Selvvurdering | En tjekliste over kontroller: opfyldt, delvist opfyldt, ikke opfyldt, besvares af den driftsansvarlige | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Kendte mangler, som registreret

- Der er ikke udført nogen uafhængig penetrationstest. Testene er skrevet af forfatteren.
- Actions er fastlåst med tag, ikke med digest, og images er ikke signeret.
- Hastighedsgrænser gælder pr. instans.
- Applikationen bruger den databaserolle, den får; der findes intet sæt roller med mindst mulige rettigheder.
- Tilbagekaldelse af tokens og tokens maksimale alder hører under din identitetsudbyder.
- Indholdsstyringens sti indlæser et script fra et offentligt CDN uden fastlåst version og uden integritetshash.
- Logge er ikke kontrolleret for personoplysninger, og revisionssporet eksporteres ikke til et overvågningssystem.
- En administreret Redis (klynge, sentinel, brugere med adgangsstyring) er ikke afprøvet.

## Hvad kun mennesker kan gøre

Bestille den uafhængige penetrationstest, gennemgå trusselsmodellen for dit miljø, køre en genopretningsøvelse på
din platform og sætte grenbeskyttelse op på hostingplatformen. Se
[parathedsvurderingen](../../../../spec/readiness-assessment.md), hvis konklusion er **ikke klar** til en driftssætning med høje krav til sikkerhed.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
