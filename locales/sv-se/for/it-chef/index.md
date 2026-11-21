# För it-chefen

**Perspektiv:** hur passar detta in i vår it-miljö, vem kan logga in, var finns data,
och hur driftar och återställer vi det?

## Vad du får

| Fråga | Vad som finns | Läs |
| --- | --- | --- |
| Uppbyggnad | Ett JSON-API och en separat webbläsarklient; PostgreSQL lagrar data | [architecture.md](../../../../spec/architecture.md) |
| Identitet | WPM lagrar inga identiteter. En person, en medarbetare och en organisation är referenser till andra tjänster, löses enbart via namn och cachas | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Inloggning | På som standard. Inloggning via företagskatalog (OIDC, Entra-kompatibel, direkt eller via Keycloak); roller och attribut avgör åtkomsten | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Driftsättning | Ett containerbygge och en compose-fil för en demonstration; för produktion en proxy som avslutar TLS, obligatoriska inställningar och startkontroller som vägrar en osäker konfiguration | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Delad cache | Valfri Redis enbart för icke-personliga uppslag; varje anrop faller tillbaka till öppet läge | `REDIS_URL` i driftsättningsguiden (WPM-D78) |
| Säkerhetskopiering och återställning | Krypterade säkerhetskopior, ett återställningsskript och en återställningsövning som CI kör | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Uppgradering | Migreringar och en uppgraderingsguide | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| När något går fel | En driftbok med första åtgärder | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Dataflöden | Ett dokument om dataflödet | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Bra att veta innan du planerar

- Användargränssnittet är **inte containeriserat**; du väljer en adapter för din plattform.
- Hastighetsbegränsningarna gäller per instans. Ett delat lager får du själv tillhandahålla.
- Applikationen använder den databasroll som `DATABASE_URL` anger; en uppsättning roller med minsta möjliga behörighet finns ännu inte.
- Återkallelse av token och tokens maximala ålder hanteras av din identitetsleverantör.
- Hanterad Redis (kluster, sentinel, användare med åtkomstkontroll) har inte prövats.
- Ingen belastningsmätning finns, och vissa vyer läser in en hel organisation i minnet.
- Containerbygget och compose-filen är skrivna men bör prövas i er miljö först.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
