# För riskchefen

**Perspektiv:** vad kan gå fel med det här systemet, vad hindrar det, och vad har ingen
kontrollerat?

## Den ärliga utgångspunkten

[Beredskapsbedömningen](../../../../spec/readiness-assessment.md) från 2026-10-11 ger utlåtandet **inte redo** för en
driftsättning med höga krav på säkerhet. Den fann ett hinder genom att sondera den körande tjänsten: vilken inloggad person som helst kunde läsa en
kollegas sjukfrånvaro och dess fritextskäl. Det är åtgärdat och testat. De grindar som bara människor kan stänga är
fortfarande öppna.

## Vad du får

| Fråga | Vad som finns | Läs |
| --- | --- | --- |
| Hot vid varje gräns | En hotmodell: tillgångar, aktörer, gränser, kontrollen som byggd och vad som **inte har bedömts** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Obehöriga läsningar | Ett skydd för behov av kännedom som klassificerar varje läsrutt; ett test misslyckas vid en oklassificerad | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Manipulation och ansvarsskyldighet | Ett spår som bara utökas och är kedjat med hashvärden, med en verifieringsändpunkt; poster innehåller ingen skälstext och inget värde | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Känsliga data | Ingen diagnos lagras; funktioner för särskilda kategorier är avstängda som standard; en tröskel för små grupper | [regulatory.md](../../../../spec/regulatory.md) |
| Osäkra vägar | Produktion vägrar starta med inloggning avstängd, ingen källa för tokennycklar, en okrypterad databasanslutning till en annan värd eller en okrypterad fjärr-Redis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Dataförlust | Krypterade säkerhetskopior, en återställningsövning som körs i CI, en raderingslogg | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Partiskhet i poängsättning | En mall för konsekvenser för jämlikhet och ett register över varje poäng med en preliminär bedömning | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| En självbedömning av kontroller | En checklista där varje svar är uppfyllt, delvis uppfyllt, inte uppfyllt eller för den som driftsätter att besvara | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Öppna risker, som de dokumenterats

Fastlåsning och signering av leveranskedjan, belastningsbevis, loggkontroller avseende personuppgifter, delade hastighetsbegränsningar mellan
instanser, en uppsättning databasroller med minsta möjliga behörighet och vägledning om återkallelse av token är öppna punkter i
beredskapsbedömningen, var och en med en uppgift. Testerna skrevs av upphovspersonen; inget är en oberoende granskning.

## Vad bara människor kan göra

Ett oberoende penetrationstest, en oberoende tillgänglighetsgranskning, att den som driftsätter får sin konsekvensbedömning godkänd,
en återställningsövning på din plattform och godkännande av bedömningar av konsekvenser för jämlikhet. Arkivet anger dessa
som grindar och markerar dem inte som klara.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
