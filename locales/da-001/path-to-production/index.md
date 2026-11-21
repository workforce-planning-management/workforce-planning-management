# Vejen til produktion

WPM er demonstrationssoftware. Denne side siger, hvad produktionsbrug ville betyde for den, og opregner, hvad der står mellem WPM og den brug. Hvert punkt har et skøn over, hvor klart det er, fra 0 til 100 procent.

## Hvad produktion betyder

Produktionsbrug betyder, at en organisation stoler på WPM til rigtigt arbejde:

- posterne er om rigtige mennesker: medarbejdere, ansøgere og deres nødkontakter;
- løn, som WPM beregner, udbetales, og lovgivningen på arbejdsstedet gælder for den;
- folk logger ind hver dag, og noget af det, de læser, er følsomt, for eksempel fravær på grund af sygdom;
- organisationen svarer over for en tilsynsmyndighed, en revisor og de mennesker, der står i posterne, for, hvordan den drives.

Denne side måler WPM mod det sværeste tilfælde, det blev vurderet for: en stor medicinsk eller offentlig organisation ([parathedsvurderingen](../../../spec/readiness-assessment.md)). En mindre organisation kan have brug for mindre af nogle punkter, men den har brug for de samme slags dokumentation.

Produktion kræver tre slags dokumentation, og software alene leverer kun den første:

1. **Software**, der gør arbejdet sikkert, med tests, der viser det.
2. **En driftsansvarlig**, der hoster og driver den: sikkerhedskopier, overvågning, support og håndtering af hændelser.
3. **Andre end forfatteren**, som har kontrolleret den: en penetrationstester, en revisor af tilgængelighed, oversættere og den driftsansvarliges databeskyttelsesrådgiver.

## Sådan læser du skønnene

- Et skøn er forfatterens vurdering den 2026-11-13, lavet ud fra specifikationen og testene. Det er ikke en måling og ikke en certificering.
- 0 procent betyder, at intet findes endnu. 100 procent betyder bygget, testet og kontrolleret af en uafhængig i en rigtig driftssætning. Intet punkt er på 100 procent.
- Hvert punkt siger, hvad dets skøn bygger på, og hvad der mangler.
- Skønnene beregnes ikke som gennemsnit. Ét punkt med et lavt skøn er nok til at holde WPM ude af produktion.

## Softwaren

- **75%** **Login og adgang efter behov for at vide.** Login er slået til som standard, og tjenesten nægter at starte i produktion uden det (WPM-R72). Læsning er begrænset til folk med behov for at vide (WPM-T188), og et token har en maksimal alder (WPM-T200). Bygger på: de håndhævende tests, der kører i CI. Mangler: der er ingen liste over tilbagekaldte tokens, og ingen uafhængig penetrationstest er kørt.
- **70%** **Et komplet og uændret revisionsspor.** Revisionssporet kan kun føjes til og er kædet med hashværdier og kan eksporteres til et overvågningssystem (WPM-T189, WPM-T197). Bygger på: tests, der med vilje blev brudt for at vise, at de fejler. Mangler: en afskåret hale af kæden opdages ikke, og en databaseejer kan stadig slå triggerne fra.
- **70%** **Databeskyttelse gennem design.** Mennesker identificeres ved henvisninger, der er ingen demografiske kolonner, og der er indsigtsret, sletning og opbevaring for hver slags post. Bygger på: testene og afsnit 1 i parathedsvurderingen. Mangler: den driftsansvarliges egen vurdering og retlige grundlag (nedenfor).
- **75%** **Krypterede forbindelser og sikkerhedskopier.** I produktion afviser tjenesten en databaseforbindelse uden kryptering og afbryder en anmodning, der hænger (WPM-T191), og sikkerhedskopier er krypterede (WPM-T192). Bygger på: testene af begge. Mangler: håndteringen af nøgler og certifikater hører under den driftsansvarlige.
- **10%** **Lovbestemte beregninger.** De lovbestemte beregninger er illustrative, og lønskat er en pladsholder ([den regulatoriske position](../../../spec/regulatory.md)). Bygger på: specifikationen, som siger det. Mangler: reglerne for hvert arbejdssted, skrevet og kontrolleret af en kvalificeret person. Udbetal ikke løn til nogen fra WPM.
- **50%** **Skala.** På én bærbar fejler kataloget og kapacitetsvisningen ikke ved 50.000 medarbejdere, og planlægningsvisningerne afviser flere end 10.000 (WPM-T196, WPM-T230). Bygger på: én målt kørsel ([BENCHMARKS.md](../../../BENCHMARKS.md)). Mangler: kapacitetsvisningen vokser stadig i forhold til arbejdsstyrken, og intet er målt på den driftsansvarliges hardware.
- **65%** **Forsyningskæde.** Hver CI-action er fastlåst til en commit, og historikken scannes for hemmeligheder. Container-imaget er signeret og bærer sin proveniens, og det er kørt for udgivelse v0.1.2 (WPM-T194, WPM-T195). Bygger på: den udgivelseskørsel. Mangler: image-pakken er privat, så ingen uden for projektet kan endnu verificere den.
- **60%** **Overvågning.** Hvert svar bærer et anmodnings-id, og loglinjer indeholder ingen forespørgselsstreng eller personlig identifikator (WPM-T197). Bygger på: testene af anmodningsloggen. Mangler: dashboards og alarmer hører under den driftsansvarlige, og hastighedsgrænser gælder kun én instans (WPM-T198).
- **60%** **Sikkerhedskopi og genopretning.** Krypterede sikkerhedskopier, en genopretningsøvelse, der kører i CI, og en driftsvejledning. Bygger på: øvelsen. Mangler: en genopretningsøvelse på den driftsansvarliges egen platform inden for de seneste 90 kalenderdage (port WPM-G4).
- **40%** **Kontrol af tilgængelighed.** Automatiske kontroller mod WCAG 2.2 AA kører på applikationen og på dette websted, i lyst og mørkt tema. Bygger på: Playwright- og axe-testene. Mangler: den uafhængige revision (nedenfor); ingen skærmlæser er brugt.
- **20%** **Oversættelser.** Applikationen findes på 17 lokaliteter og rollebeskrivelserne på 21. Bygger på: teksten findes. Mangler: en AI skrev oversættelserne, og ingen har gennemgået dem professionelt (WPM-T155).

## Kontrol udført af andre end forfatteren

- **0%** **Uafhængig penetrationstest.** Ingen penetrationstest af en driftssat instans er kørt (WPM-T201). Bygger på: opgaven er åben. Mangler: testen og rettelse af det, den finder.
- **0%** **Uafhængig tilgængelighedsrevision.** Ingen revision mod WCAG 2.2 AA sammen med folk, der bruger hjælpeteknologi, er udført (WPM-T202). Bygger på: opgaven er åben. Mangler: revisionen og rettelse af det, den finder.
- **25%** **Trusselsmodel.** En indledende trusselsmodel findes ([threat-model.md](../../../spec/governance/threat-model.md)), skrevet af forfatteren. Bygger på: dokumentet. Mangler: et uafhængigt team laver den om for en rigtig driftssætning.

## Den driftsansvarliges del

- **20%** **Konsekvensanalyse for databeskyttelse.** En pakke til tilpasning findes: en konsekvensanalyse, et dataflow, en fortegnelse over behandling og en opbevaringsplan ([spec/governance](../../../spec/governance/dpia.md)). Bygger på: pakken. Mangler: der er ingen driftsansvarlig, så ingen databeskyttelsesrådgiver har godkendt en vurdering, et retligt grundlag, en opbevaringsplan eller en databehandleraftale (port WPM-G5).
- **15%** **Konsekvensanalyser for ligestilling.** En skabelon og et register findes ([equality-impact](../../../spec/equality-impact/template.md)). Bygger på: skabelonen. Mangler: den driftsansvarlige skriver og godkender en vurdering til sin egen brug.
- **40%** **Hosting og drift.** En produktionskonfiguration, et container-image og dokumenter om driftssætning og opgraderinger findes. Bygger på: de dokumenter og genopretningsøvelsen. Mangler: intet er kørt på en hærdet vært, og en driftsansvarlig skal drive tjenesten.
- **10%** **Support.** Der findes en sagsstyring og en sikkerhedspolitik ([SECURITY.md](../../../SECURITY.md)). Bygger på: de to. Mangler: der er ingen supportaftale, serviceniveau, vagtplan eller organisation bag WPM.

## Hvad denne side ikke hævder

Siden registrerer ingen beslutning om at gå i drift. Den beslutning tilhører en driftsansvarlig og træffes uden for WPM. Skønnene dækker kun de opregnede punkter, og parathedsvurderingens egen liste over, hvad den ikke kontrollerede, gælder også her. Hvor et klinisk system er tilsluttet, er en gennemgang af klinisk sikkerhed den driftsansvarliges beslutning.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/path-to-production/`), er det den engelske version, der gælder.
