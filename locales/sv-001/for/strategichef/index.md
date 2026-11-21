# För strategichefen

**Perspektiv:** är vår arbetskraft utformad för att tjäna strategin, vad kommer vi att behöva, och vad
bör vi göra åt glappet?

## Vad du får

| Du vill veta | Var WPM svarar | Läs |
| --- | --- | --- |
| Vad en roll kräver | Rollprofiler med nödvändiga kompetenser och en lägsta kunskapsnivå, så att ett gap mäts mot en roll och inte mot en person | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Vad vi kommer att behöva | Arbetskraftsplaner som utkast till olika scenarier, flera sida vid sida, med angivna antaganden och behov per avdelning, roll och period | WPM-R36, samma fil |
| Var glappet finns | Beräknat utbud mot planerat behov: ett gap i personalstyrka och ett kompetensgap, med åtgärder uppräknade som förslag och deras underlag | WPM-R37 |
| Om det tjänar strategin | Mål kopplade till behov: mål utan resurser, behov utan koppling till mål, kritiska roller utan reserv | WPM-R38 |
| Om vi kan leverera ett program | Kompetenspooler, programmens behov, åtaganden från partner, flaskhalspoolen, en startkontroll som är ett förslag | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Hur mycket vi förlitar oss på personal utan tillsvidareanställning | Andelen heltidsekvivalenter som är konsulter och visstidsanställda, var för sig, och hur mycket finansiering som tar slut varje månad under de kommande tolv | `GET /api/workforce-intelligence/contingent` (WPM-T109, delvis) |
| Vad vi planerar att göra när avtal upphör | Omvandlingsplaner: en människas beslut att omvandla, förlänga eller avsluta, med finansieringen vid sidan av avtalets slut | [engagements.md](../../../../spec/engagements.md) |
| Vad som mer är planerat | Färdplanen och den öppna uppgiftslistan | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Så läser du en prognos

- Den är en angiven regel tillämpad på angivna indata, inte en modell, och varje resultat anger sina antaganden.
- Med för kort historik över personalstyrkan säger den `insufficient_history` i stället för att extrapolera.
- En plan innehåller bara hypotetisk personalstyrka. Den läser eller ändrar aldrig en persons register.
- Åtgärder är förslag. Planeraren väljer, och inget avgörs av programvaran.

## Begränsningar

- Åtgärden "omvandla inhyrd personal till tillsvidareanställning" i planer är specificerad (WPM-R63) men finns ännu inte i koden.
- Andelen inhyrd personal har ännu ingen trend: den kräver ögonblicksbilder uppdelade efter grund (WPM-T139, inte byggd).
- Schemaläggning med kritisk kedja och kritisk väg hör till project-portfolio-management, inte hit.
- Något användargränssnitt för finansiering eller omvandlingsplaner finns ännu inte.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
