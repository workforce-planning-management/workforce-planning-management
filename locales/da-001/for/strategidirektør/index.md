# Til strategidirektøren

**Perspektiv:** er vores arbejdsstyrke formet til at tjene strategien, hvad får vi brug for, og hvad
bør vi gøre ved forskellen?

## Hvad du får

| Du vil vide | Hvor WPM svarer | Læs |
| --- | --- | --- |
| Hvad en rolle kræver | Rolleprofiler med krævede kompetencer og et mindsteniveau, så et gab måles mod en rolle og ikke en person | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Hvad vi får brug for | Arbejdsstyrkeplaner som kladdeverdener, flere side om side, med angivne forudsætninger og behov pr. afdeling, rolle og periode | WPM-R36, samme fil |
| Hvor forskellen er | Forventet udbud mod planlagt behov: et gab i bemanding og et gab i kompetencer, med løftestænger oplistet som forslag og deres dokumentation | WPM-R37 |
| Om det tjener strategien | Målsætninger knyttet til behov: målsætninger uden ressourcer, behov uden tilknytning, kritiske roller uden reserve | WPM-R38 |
| Om vi kan levere et program | Kompetencepuljer, programmets behov, partneres tilsagn, den begrænsende pulje, et startjek, der er et forslag | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Hvor meget vi er afhængige af ikke-fastansatte | Andelen af fuldtidsækvivalenter, der er konsulenter og tidsbegrænsede, hver for sig, og hvor meget finansiering der udløber i hver af de næste tolv måneder | `GET /api/workforce-intelligence/contingent` (WPM-T109, delvist) |
| Hvad vi planlægger at gøre ved kontrakter, der udløber | Planer for omlægning: et menneskes beslutning om at omlægge, forlænge eller afslutte, med finansieringen ved siden af kontraktens udløb | [engagements.md](../../../../spec/engagements.md) |
| Hvad der ellers er planlagt | Køreplanen og den åbne opgaveliste | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Sådan læser du en prognose

- Den er en angivet regel anvendt på angivne inputdata, ikke en model, og hvert resultat nævner sine forudsætninger.
- Med for lidt historik for bemandingen siger den `insufficient_history` i stedet for at ekstrapolere.
- En plan rummer kun hypotetisk bemanding. Den læser eller ændrer aldrig en persons post.
- Løftestænger er forslag. Planlæggeren vælger, og softwaren beslutter intet.

## Begrænsninger

- Løftestangen "omlæg tilkaldte til fast" i planer er specificeret (WPM-R63) og endnu ikke i koden.
- Andelen af tilkaldte har endnu ingen tendens: den kræver øjebliksbilleder delt op efter grundlag (WPM-T139, ikke bygget).
- Planlægning efter kritisk kæde og kritisk sti hører til project-portfolio-management, ikke her.
- Der findes endnu ingen brugerflade til finansiering eller planer for omlægning.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
