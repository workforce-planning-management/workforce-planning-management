# Til HR-direktøren

**Perspektiv:** understøtter systemet hele medarbejderens livscyklus, og kan jeg bruge det uden at
opbevare flere oplysninger om mennesker, end jeg bør?

## Hvad du får

| Fase | Hvad WPM rummer | Læs |
| --- | --- | --- |
| Ansættelse | Stillingsopslag, et ansøgerforløb, en kandidatpulje, tjeklister til onboarding | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| Personalesagen | Én ansættelsespost pr. medarbejder, organisationsdiagrammet, rapporteringslinjer, grupper, overflytning mellem organisationer | [hr-core.md](../../../../spec/hr-core.md) |
| Selvbetjening | En medarbejder ser og ændrer sine egne oplysninger, anmoder om fleksibel arbejdstid, opsiger og ser sin frihed | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Arbejdslivet | Fravær, tid, vagter, personalegoder, trivselsspørgsmål, rimelige tilpasninger, ergonomiske vurderinger, den anonyme pulsmåling | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Udvikling | Medarbejdersamtaler, 360-vurderinger, kurser gennem kursusregistret, vurderinger, mentorordninger, udviklingsplaner, succession, tidlig karriere | [talent-development.md](../../../../spec/talent-development.md) |
| Løn | Lønkørsler, lønsedler, benchmarking af løn, opslag i en national lønskala, jobniveauer | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Ikke-fastansatte | Slutdatoer efter grundlag, forlængelser, vurderinger af status, planer for omlægning, som et menneske beslutter, finansiering ved siden af kontraktens udløb | [engagements.md](../../../../spec/engagements.md) |
| Fratræden | Opsigelse med varsel, en overdragelse på sidste arbejdsdag, opbevaring pr. slags post, sletning | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Sådan beskytter det mennesker

- **Behov for at vide ved læsning.** En leder ser sine egne medarbejdere, en medarbejder ser sig selv, HR ser det, HR har brug for.
  En fremmed, der er logget ind, ser intet om en kollega ([auth.md](../../../../spec/auth.md)).
- **Ingen diagnose gemmes.** Sygefravær registrerer en art, ikke en årsag.
- **Funktioner til særlige kategorier af oplysninger er slået fra som standard.** Overvågning af ligestilling og krav til sundhed på arbejdspladsen kræver et
  registreret lovligt grundlag, er frivillige eller rummes af arbejdsmedicinsk rolle og viser kun grupper over en nedre grænse.
  De indgår aldrig i nogen score eller beslutning om en person.
- **Score rådgiver.** Parathed til succession, kompetencegab og prognoser foreslår; et menneske beslutter, og beslutninger som
  omlægning af en konsulent registreres som et menneskes beslutning, aldrig som et automatisk udfald.
- **Den registrerede kan se og slette.** Eksport ved indsigtsret og sletning dækker ovenstående poster, med en protokol, så
  en genopretning ikke bringer en slettet person tilbage.

## Begrænsninger

- Endnu ikke bygget: en brugerflade til planer for omlægning og finansiering, tendensen for tilkaldte og medarbejderens
  side "mine oplysninger". Se status i [tasks.md](../../../../spec/tasks.md).
- Oversættelserne er maskinunderstøttede; en professionel gennemgang er en åben port.
- Kollektive overenskomster og anerkendte organer er kun undersøgelse og forslag ([worker-unions.md](../../../../spec/worker-unions.md)).
- Lønskat er en pladsholder.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
