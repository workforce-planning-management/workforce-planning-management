# Styrning, risk och regelefterlevnad

**Perspektiv:** vem ansvarar för vad, vad kan gå fel och vad hindrar det, och
vilka belägg kan jag visa en bedömare? Den här sidan samlar material om styrning, risk och regelefterlevnad på ett ställe.
Rollsidorna för riskchefen och den juridiska chefen behandlar
samma områden utifrån respektive roll.

## Styrning: vem beslutar, och hur det dokumenteras

| Ämne | Vad som finns | Läs |
| --- | --- | --- |
| Hur arbetet styrs | Specifikationen först: ett krav, ett beslut, en uppgift, kod och tester levereras tillsammans; varje uppgift säger vad som **inte** har verifierats | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Projektets styrning | Styrning, underhållare, en RFC-process och en fil med kodansvariga | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Beslut om människor | En människa beslutar; poäng ger underlag. Omvandlingar, uppsägningar och önskemål om flexibelt arbete dokumenteras som en människas beslut | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Åtkomst och roller | Roller och attribut avgör åtkomsten; en klassificerad tabell avgör behov av kännedom på varje läsrutt | [auth.md](../../../../spec/auth.md) |
| AI-assisterat författarskap | En redogörelse för hur AI har använts | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Risk: vad kan gå fel

| Ämne | Vad som finns | Läs |
| --- | --- | --- |
| Utlåtande | **Inte redo** för en driftsättning med höga krav på säkerhet; ett hinder (läsningar byggde inte på behov av kännedom) hittades genom sondering och åtgärdades | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Hot | En hotmodell: tillgångar, aktörer, gränser, kontrollen som byggd och vad som **inte har bedömts** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Spår av manipulation | Ett spår som bara utökas och är kedjat med hashvärden, med en verifieringsändpunkt; poster innehåller ingen skälstext och inget värde | [audit.md](../../../../spec/audit.md) |
| Säker start | Produktion vägrar starta med inloggning avstängd, utan tokennycklar, med en okrypterad databasanslutning till en annan värd eller med en okrypterad fjärr-Redis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Återställning | Krypterade säkerhetskopior, en återställningsövning som CI kör, en raderingslogg så att en återställning inte för tillbaka en raderad person | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Incidenter | En driftbok med första åtgärder; den som driftsätter lägger till kontaktpersoner och eskaleringsvägar | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Partiskhet i poängsättning | En mall för konsekvenser för jämlikhet och ett register över varje poäng, var och en med en preliminär bedömning | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Öppna risker | Fastlåsning och signering av leveranskedjan, belastningsbevis, loggkontroller avseende personuppgifter, delade hastighetsbegränsningar, en uppsättning databasroller med minsta möjliga behörighet, vägledning om återkallelse av token | [readiness-assessment.md](../../../../spec/readiness-assessment.md), avsnitt 3 |

## Regelefterlevnad: vad du kan visa en bedömare

| Ämne | Vad som finns | Läs |
| --- | --- | --- |
| Självbedömning av kontroller | En checklista där varje svar är uppfyllt, delvis uppfyllt, inte uppfyllt eller för den som driftsätter att besvara | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Konsekvensbedömning avseende dataskydd | En mall, ifylld för demon, med de delar du själv måste fylla i markerade | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Register över behandling | Ett anpassningsbart register | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Dataflöde | Var personuppgifter förflyttas | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Lagringstid | Ett schema per typ av register | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Den registrerades rättigheter | Registerutdrag och radering | [regulatory.md](../../../../spec/regulatory.md) |
| Särskilda kategorier av personuppgifter | Jämlikhetsuppföljning och krav på arbetsplatshälsa är avstängda tills en rättslig grund dokumenterats, och därefter begränsade | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Tillgänglighet och språk | En tillgänglighetsredogörelse och en redogörelse om officiella språk | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Licenser och beroenden | Licensfiler, en policy för beroenden som kontrolleras med `cargo deny` och en programvarustycklista vid utgivning | [production-readiness.md](../../../../spec/production-readiness.md) |

## Vad bara människor kan göra

Det här är grindar, inte kod, och arkivet markerar dem inte som klara: ett oberoende penetrationstest, en
oberoende tillgänglighetsgranskning, din konsekvensbedömning avseende dataskydd godkänd av ditt dataskyddsombud, en återställningsövning
på din plattform, en professionell granskning av översättningarna och ditt godkännande av bedömningarna av konsekvenser för jämlikhet.

## Den här sidans begränsningar

Tabellerna pekar på dokument. Om varje dokument stämmer för din driftsättning är en fråga för dig och din bedömare.
Upphovspersonens tester är ingen oberoende granskning, och inget här är juridisk rådgivning eller en certifiering.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
