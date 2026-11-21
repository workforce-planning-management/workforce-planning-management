# För personalchefen

**Perspektiv:** stöder detta hela medarbetarens livscykel, och kan jag använda det utan att
lagra mer om människor än jag bör?

## Vad du får

| Skede | Vad WPM lagrar | Läs |
| --- | --- | --- |
| Rekrytering | Tjänstebehov, ett ansökningsflöde, en kandidatpool, checklistor för introduktion | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| Registret | Ett anställningsregister per medarbetare, organisationsschemat, rapporteringslinjer, grupper, förflyttningar mellan organisationer | [hr-core.md](../../../../spec/hr-core.md) |
| Självservice | En medarbetare ser och uppdaterar sina egna uppgifter, begär flexibelt arbete, säger upp sig och ser sin ledighet | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Arbetslivet | Ledighet, tid, skift, förmåner, frågor om välbefinnande, rimliga anpassningar, ergonomiska bedömningar, den anonyma pulsen | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Utveckling | Medarbetarsamtal, 360-bedömningar, kurser via kursregistret, bedömningar, mentorskap, utvecklingsplaner, succession, tidiga karriärer | [talent-development.md](../../../../spec/talent-development.md) |
| Lön | Lönekörningar, lönebesked, lönejämförelser, uppslag i nationell lönetrappa, tjänstenivåer | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Personal utan tillsvidareanställning | Slutdatum per grund, förlängningar, statusbedömningar, omvandlingsplaner som beslutas av en människa, finansiering vid sidan av avtalets slut | [engagements.md](../../../../spec/engagements.md) |
| Avslut | Uppsägning med uppsägningstid, en överlämning på sista dagen, gallring per typ av register, radering | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Så skyddar det människor

- **Behov av kännedom vid läsning.** En chef ser sina egna medarbetare, en medarbetare ser sig själv, HR ser det HR behöver.
  En inloggad främling ser ingenting om en kollega ([auth.md](../../../../spec/auth.md)).
- **Ingen diagnos lagras.** Sjukfrånvaro registrerar en typ, inte ett skäl.
- **Funktioner för särskilda kategorier av personuppgifter är avstängda som standard.** Jämlikhetsuppföljning och krav på arbetsplatshälsa kräver en
  dokumenterad rättslig grund, är frivilliga eller förs av företagshälsovården, och visar bara grupper över en tröskel.
  De används aldrig som indata till någon poäng eller något beslut om en person.
- **Poäng ger underlag.** Successionsberedskap, kompetensgap och prognoser föreslår; en människa beslutar, och beslut som
  att göra en konsult till anställd dokumenteras som en människas beslut, aldrig som ett automatiskt utfall.
- **Den registrerade kan se och radera.** Registerutdrag och radering omfattar registren ovan, med en logg så att
  en återställning inte för tillbaka en raderad person.

## Begränsningar

- Ännu inte byggt: ett användargränssnitt för omvandlingsplaner och finansiering, trenden för inhyrd personal och medarbetarens
  sida "mina uppgifter". Se [tasks.md](../../../../spec/tasks.md) för aktuellt läge.
- Översättningarna är maskinassisterade; en professionell granskning är en öppen grind.
- Kollektivavtal och erkända organ är endast utredning och förslag ([worker-unions.md](../../../../spec/worker-unions.md)).
- Löneskatten är en platshållare.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
