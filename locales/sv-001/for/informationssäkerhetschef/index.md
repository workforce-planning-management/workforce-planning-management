# För informationssäkerhetschefen

**Perspektiv:** hur skyddas detta, hur skulle vi veta om det angreps eller missbrukades, och
vad har ingen testat?

## Vad du får

| Fråga | Vad som finns | Läs |
| --- | --- | --- |
| Hot | En hotmodell: tillgångar, aktörer, förtroendegränser, kontrollen som byggd och vad som **inte har bedömts** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Autentisering | Inloggning på som standard; produktion vägrar starta med den avstängd eller utan någon källa för tokennycklar; inloggning via företagskatalog (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Behörighet | Policyn avgör enbart efter åtgärd; en klassificerad tabell avgör behov av kännedom på varje läsrutt, och ett test som skannar källkoden misslyckas vid en oklassificerad rutt | [auth.md](../../../../spec/auth.md) |
| Spårbarhet | Ett spår som bara utökas och är kedjat med hashvärden, med en verifieringsändpunkt; poster har ingen skälstext och inget värde | [audit.md](../../../../spec/audit.md) |
| Transport och webbläsare | Säkerhetshuvuden på varje API-svar, en uttrycklig CORS-tillåtelselista som får starten att misslyckas om den saknas, `no-store` på `/api` | WPM-R73 i [production-readiness.md](../../../../spec/production-readiness.md) |
| Missbruk | Hastighetsbegränsningar per anropare, strängare för radering, import, export och tokenrutter | WPM-R74, samma fil |
| Osäker konfiguration | Starten vägrar en okrypterad databasanslutning eller okrypterad Redis till en annan värd, och ett saknat CORS-ursprung | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Data i vila | Krypterade säkerhetskopior; den delade cachen innehåller bara icke-personliga uppslag och faller tillbaka till öppet läge | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Leveranskedja | Kontroller av licenser och förbud med `cargo deny`, versionsfastlåsta delade bibliotek, en programvarustycklista vid utgivning | [production-readiness.md](../../../../spec/production-readiness.md) |
| Incidenter | En driftbok med första åtgärder; du lägger till kontaktpersoner och anmälningsskyldigheter | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Självbedömning | En kontrollchecklista: uppfyllt, delvis uppfyllt, inte uppfyllt, den som driftsätter besvarar | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Kända luckor, som de dokumenterats

- Inget oberoende penetrationstest har gjorts. Testerna skrevs av upphovspersonen.
- Åtgärder är fastlåsta med tagg, inte med digest, och avbilder är inte signerade.
- Hastighetsbegränsningarna gäller per instans.
- Applikationen använder den databasroll den får; ingen uppsättning roller med minsta möjliga behörighet tillhandahålls.
- Återkallelse av token och tokens maximala ålder hanteras av din identitetsleverantör.
- Innehållshanterarens väg läser in ett skript från ett offentligt CDN utan fastlåst version eller integritetshash.
- Loggar har inte kontrollerats avseende personuppgifter, och granskningsspåret exporteras inte till något övervakningssystem.
- En hanterad Redis (kluster, sentinel, användare med åtkomstkontroll) har inte prövats.

## Vad bara människor kan göra

Beställa det oberoende penetrationstestet, granska hotmodellen för din miljö, genomföra en återställningsövning på
din plattform och ställa in grenskydd på värdplattformen. Se
[beredskapsbedömningen](../../../../spec/readiness-assessment.md), vars utlåtande är **inte redo** för en driftsättning med höga krav på säkerhet.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
