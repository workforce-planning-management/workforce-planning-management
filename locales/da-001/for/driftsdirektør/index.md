# Til driftsdirektøren

**Perspektiv:** har vi de rette medarbejdere, på det rette sted, på det rette tidspunkt, og
hvad går i stykker, hvis nogen er væk?

## Hvad du får

| Du vil vide | Hvor WPM svarer | Læs |
| --- | --- | --- |
| Kan et team dække fravær | Stedfortrædere pr. person, vagtplanen for rådighedsvagter med bytte og påmindelser og en dækningsvisning, der siger klart, når ingen er til rådighed | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Tid, fremmøde og fravær | Tidsregistreringer, fraværsanmodninger med en godkendelsesvej, optjening og vagter | [workforce-management.md](../../../../spec/workforce-management.md) |
| Overholdes grænserne for arbejdstid | Vejledende værn om arbejdstid; de advarer og blokerer ikke | WPM-R27, samme fil |
| Kapacitet pr. kompetence, måned for måned | Kompetencepuljer, udbud og programmets behov, partneres tilsagn, den begrænsende pulje og et startjek, der er et forslag, med registrerede beslutninger om start | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Hvad vi får brug for, og hvor forskellen er | Arbejdsstyrkeplaner, prognose og analyse af forskelle pr. afdeling og pr. rolle | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Hvilke kontrakter udløber snart | En slutdato pr. grundlag for engagementet, forlængelser med historik, påmindelser før en udløbsdato, en liste over engagementer uden slutdato og planer for omlægning | [engagements.md](../../../../spec/engagements.md) |
| Hvor tilkaldte medarbejdere udgør en stor andel | Andelen af fuldtidsækvivalenter, der er konsulenter, og andelen, der er tidsbegrænset, hver for sig, og den finansiering, der udløber måned for måned | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Nye og fratrædende | Daterede tjeklister og en overdragelse på sidste arbejdsdag | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Kompetencegab og uddannelsestid | Analyse af gab med de beviser, en anbefaling bygger på | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Sådan læser du kapacitetsvisningen

Kapacitet planlægges pr. kompetencepulje, ikke pr. person. En pulje-måned ender i et af fire resultater, og en ikke-angivet
kompetence er *ukendt*, ikke en mangel. Startjekket er et forslag, og et menneske registrerer beslutningen om at starte.
Planlægning efter kritisk kæde og kritisk sti findes ikke her: den hører hjemme i project-portfolio-management.

## Begrænsninger

- Andelen af tilkaldte har endnu ingen tendens: den kræver øjebliksbilleder af bemandingen delt op efter grundlag (WPM-T139, ikke bygget).
- En pulje i visningen af tilkaldte er afdelingen, indtil planerne definerer puljer.
- Vagtplanlægning og beregning af lønskat er ikke en erstatning for dine systemer til vagtplanlægning eller løn.
- Flere visninger indlæser en organisations poster i hukommelsen; der er ikke målt nogen belastning
  ([readiness-assessment.md](../../../../spec/readiness-assessment.md), mangel 8).

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
