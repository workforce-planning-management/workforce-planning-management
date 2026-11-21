# För den juridiska chefen

**Perspektiv:** vilka rättsliga skyldigheter skapar eller underlättar den här programvaran, och vad måste
vi besluta innan verkliga data läggs in?

> Inget här är juridisk rådgivning. Specifikationen anger en hållning, inte en slutsats om någon särskild jurisdiktion.

## Vad du får

| Ämne | Vad som finns | Läs |
| --- | --- | --- |
| Hållning | Hållning utifrån brittisk GDPR och regler om anställningsregister; demonstrationsstatus; vad produktion därutöver skulle kräva | [regulatory.md](../../../../spec/regulatory.md) |
| Konsekvensbedömning | En mall för konsekvensbedömning avseende dataskydd, ifylld för demon, med de delar du själv måste fylla i markerade | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Register över behandling och dataflöde | Anpassningsbara dokument | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md), [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Lagringstid | Ett schema per typ av register, med gallring för vissa | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Den registrerades rättigheter | Registerutdrag, radering och en raderingslogg | [regulatory.md](../../../../spec/regulatory.md) (WPM-R30) |
| Särskilda kategorier av personuppgifter | Jämlikhetsuppföljning och krav på arbetsplatshälsa är avstängda tills du dokumenterar en rättslig grund och är därefter begränsade: frivilliga, endast för en själv eller begränsade till roll, sammanställningar över en tröskel, aldrig använda i ett beslut | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Lagstadgade rättigheter | Önskemål om flexibelt arbete med en beslutsperiod och skäl som du konfigurerar; uppsägningstid som du konfigurerar | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Skyldigheter om jämlikhet | En mall för konsekvenser för jämlikhet och ett register över varje poäng | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| Tillgänglighet och språk | En tillgänglighetsredogörelse och en redogörelse om officiella språk | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Kollektiva organ | Utredning och förslag om erkända organ och samråd, aldrig enskilt medlemskap | [worker-unions.md](../../../../spec/worker-unions.md) |
| Konsulter | Uppdragsgrund, leverantör, väg och maskerat pris registreras. Om ett uppdrag är lagligt eller korrekt klassificerat avgör inte programvaran | [engagements.md](../../../../spec/engagements.md) |
| Licenser | Licensfiler, en policy för beroenden och en programvarustycklista vid utgivning | [production-readiness.md](../../../../spec/production-readiness.md) |

## Standardvärden som inte är lag

Flera inställningar är standardvärden, inte juridisk rådgivning: uppsägningstiden, beslutsperioden för flexibelt arbete,
överklagandeperioden och skälen för avslag. Driftsättningsguiden säger att du ska **bekräfta dem** för din jurisdiktion.
Lagstadgade beräkningar som löneskatt är inte implementerade.

## Beslut som är dina

Rättslig grund för varje ändamål med behandlingen, konsekvensbedömningen avseende dataskydd och dess godkännande, personuppgiftsbiträdesavtal,
det lagringsschema du antar och om du ska slå på någon funktion för särskilda kategorier av personuppgifter.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
