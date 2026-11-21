# För driftchefen

**Perspektiv:** har vi rätt människor på rätt plats vid rätt tidpunkt, och
vad går sönder om någon är borta?

## Vad du får

| Du vill veta | Var WPM svarar | Läs |
| --- | --- | --- |
| Om ett team kan täcka frånvaro | Ersättare per person, jourschemat med byten och påminnelser samt en täckningsvy som säger rakt ut när ingen är tillgänglig | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Tid, närvaro och ledighet | Tidsregistreringar, ledighetsansökningar med godkännandekedja, intjäning och skift | [workforce-management.md](../../../../spec/workforce-management.md) |
| Om gränserna för arbetstid följs | Rådgivande skyddsräcken för arbetstid; de varnar men blockerar inte | WPM-R27, samma fil |
| Kapacitet per kompetens, månad för månad | Kompetenspooler, utbud och programmens behov, åtaganden från partner, flaskhalspoolen och en startkontroll som är ett förslag, med dokumenterade startbeslut | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Vad vi kommer att behöva och var glappet finns | Arbetskraftsplaner, prognos och gapanalys per avdelning och roll | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Vilka avtal som snart upphör | Ett slutdatum per uppdragsgrund, förlängningar med historik, påminnelser före ett slut, en lista över uppdrag som saknar slutdatum och omvandlingsplaner | [engagements.md](../../../../spec/engagements.md) |
| Var inhyrd personal utgör en stor andel | Andelen heltidsekvivalenter som är konsulter och som är visstidsanställda, var för sig, och den finansiering som tar slut månad för månad | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Nyanställda och avgående | Daterade checklistor och en överlämning på sista dagen | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Kompetensgap och utbildningstid | Gapanalys med det underlag en rekommendation vilar på | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Så läser du kapacitetsvyn

Kapaciteten planeras per kompetenspool, inte per person. En poolmånad får ett av fyra utfall, och en ej angiven
kompetens är *okänd*, inte ett underskott. Startkontrollen är ett förslag, och en människa dokumenterar beslutet att starta.
Schemaläggning med kritisk kedja och kritisk väg finns inte här: den hör till project-portfolio-management.

## Begränsningar

- Andelen inhyrd personal har ännu ingen trend: den kräver ögonblicksbilder av personalstyrkan uppdelade efter grund (WPM-T139, inte byggd).
- En pool för vyn över inhyrd personal är avdelningen tills planerna definierar pooler.
- Schemaläggning av skift och beräkningar av löneskatt ersätter inte dina system för bemanning eller lön.
- Flera vyer läser in en organisations register i minnet; ingen belastningssiffra har uppmätts
  ([readiness-assessment.md](../../../../spec/readiness-assessment.md), lucka 8).

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
