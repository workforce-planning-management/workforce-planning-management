# Til IT-direktøren

**Perspektiv:** hvordan passer det ind i vores it-landskab, hvem kan logge ind, hvor ligger dataene,
og hvordan driver og genopretter vi det?

## Hvad du får

| Emne | Hvad der findes | Læs |
| --- | --- | --- |
| Opbygning | Et JSON-API og en separat browserklient; PostgreSQL rummer dataene | [architecture.md](../../../../spec/architecture.md) |
| Identitet | WPM rummer ingen identiteter. En person, en medarbejder og en organisation er henvisninger til andre tjenester, slås kun op på navn og lægges i cache | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Login | Slået til som standard. Login via virksomhedens kataloge (OIDC, kompatibelt med Entra, direkte eller gennem Keycloak); roller og attributter afgør adgangen | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Drift | Et container-build og en compose-fil til en demonstration; til produktion en proxy, der afslutter TLS, krævede indstillinger og opstartskontroller, der afviser en usikker opsætning | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Delt cache | Valgfri Redis kun til opslag uden personoplysninger; alle kald fejler åbent | `REDIS_URL` i driftsvejledningen (WPM-D78) |
| Sikkerhedskopi og genopretning | Krypterede sikkerhedskopier, et genopretningsscript og en genopretningsøvelse, som CI kører | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Opgradering | Migreringer og en opgraderingsvejledning | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Når noget går galt | En driftsvejledning med de første handlinger | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Datastrømme | Et dokument om dataflow | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Ting at vide, før du planlægger

- UI'et er **ikke i en container**; du vælger en adapter til din platform.
- Hastighedsgrænser gælder pr. instans. Et delt lager skal du selv stille til rådighed.
- Applikationen bruger den databaserolle, som `DATABASE_URL` angiver; et sæt roller med mindst mulige rettigheder findes endnu ikke.
- Tilbagekaldelse af tokens og tokens maksimale alder hører under din identitetsudbyder.
- Administreret Redis (klynge, sentinel, brugere med adgangsstyring) er ikke afprøvet.
- Der findes ingen belastningsmåling, og nogle visninger læser en hel organisation ind i hukommelsen.
- Image-build og compose-filen er skrevet, men bør prøves først i dit eget miljø.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
