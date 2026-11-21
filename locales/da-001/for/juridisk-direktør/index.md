# Til den juridiske direktør

**Perspektiv:** hvilke juridiske pligter skaber eller hjælper denne software med, og hvad skal
vi beslutte, før rigtige data kommer ind?

> Intet her er juridisk rådgivning. Specifikationen angiver en holdning, ikke en konklusion for noget retsområde.

## Hvad du får

| Emne | Hvad der findes | Læs |
| --- | --- | --- |
| Holdning | Holdning til UK GDPR og ansættelsesoplysninger; status som demo; hvad produktion yderligere ville kræve | [regulatory.md](../../../../spec/regulatory.md) |
| Konsekvensanalyse | En skabelon til konsekvensanalyse for databeskyttelse, udfyldt for demoen, med de dele, du selv skal udfylde, markeret | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Fortegnelse over behandling og dataflow | Dokumenter, der kan tilpasses | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md), [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Opbevaring | En plan for hver slags post, med en oprydning for nogle af dem | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Den registreredes rettigheder | Eksport ved indsigtsret, sletning og en sletteprotokol | [regulatory.md](../../../../spec/regulatory.md) (WPM-R30) |
| Særlige kategorier af oplysninger | Overvågning af ligestilling og krav til sundhed på arbejdspladsen er slået fra, indtil du registrerer et lovligt grundlag, og er derefter afgrænset: frivillige, kun for én selv eller begrænset til roller, sammentællinger over en nedre grænse, aldrig brugt i en beslutning | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Lovbestemte rettigheder | Anmodninger om fleksibel arbejdstid med en beslutningsfrist og begrundelser, du selv konfigurerer; opsigelsesvarsel, du selv konfigurerer | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Pligter om ligestilling | En skabelon til konsekvensanalyse for ligestilling og et register over hver score | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| Tilgængelighed og sprog | En tilgængelighedserklæring og en erklæring om officielle sprog | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Kollektive organer | Undersøgelse og forslag om anerkendte organer og høring, aldrig om den enkeltes medlemskab | [worker-unions.md](../../../../spec/worker-unions.md) |
| Konsulenter | Grundlaget for engagementet, leverandøren, vejen og den maskerede takst registreres. Om et engagement er lovligt eller korrekt klassificeret, afgør softwaren ikke | [engagements.md](../../../../spec/engagements.md) |
| Licenser | Licensfiler, en politik for afhængigheder og en stykliste over software ved udgivelse | [production-readiness.md](../../../../spec/production-readiness.md) |

## Standardværdier, der ikke er lov

Flere indstillinger er standardværdier og ikke juridisk rådgivning: opsigelsesvarslet, beslutningsfristen og
klagefristen for fleksibel arbejdstid samt begrundelserne for afslag. Driftsvejledningen siger, at du skal **bekræfte dem**
for dit retsområde. Lovbestemte beregninger som lønskat er ikke implementeret.

## Beslutninger, der er dine

Lovligt grundlag for hvert formål med behandlingen, konsekvensanalysen for databeskyttelse og dens godkendelse, databehandleraftaler,
den opbevaringsplan, du vedtager, og om nogen funktion med særlige kategorier af oplysninger skal slås til.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
