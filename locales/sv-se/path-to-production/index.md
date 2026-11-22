# Vägen till produktion

WPM är demonstrationsprogramvara. Den här sidan beskriver vad produktionsbruk skulle innebära för WPM och förtecknar vad som står mellan WPM och ett sådant bruk. Varje punkt har en uppskattning av hur långt den har kommit, från 0 till 100 procent.

## Vad produktion innebär

Produktionsbruk innebär att en organisation förlitar sig på WPM för verkligt arbete:

- registren gäller verkliga människor: anställda, sökande och deras nödkontakter;
- lön som WPM beräknar betalas ut, och lagen på arbetsplatsens ort gäller för den;
- människor loggar in varje dag, och en del av det de läser är känsligt, till exempel frånvaro på grund av sjukdom;
- organisationen svarar inför en tillsynsmyndighet, en revisor och de människor som finns i registren för hur den drivs.

Den här sidan mäter WPM mot det svåraste fall det bedömdes för: en stor medicinsk eller statlig organisation ([beredskapsbedömningen](../../../spec/readiness-assessment.md)). En mindre organisation kan behöva mindre av vissa punkter, men den behöver samma slags belägg.

Produktion kräver tre slags belägg, och programvaran ensam ger bara det första:

1. **Programvara** som gör jobbet på ett säkert sätt, med tester som visar det.
2. **En driftsättare** som är värd för den och driftar den: säkerhetskopior, övervakning, support och incidenthantering.
3. **Andra personer än upphovspersonen** som har granskat den: en penetrationstestare, en tillgänglighetsgranskare, översättare och driftsättarens dataskyddsombud.

## Så läser du uppskattningarna

- En uppskattning är upphovspersonens bedömning den 2026-11-13, gjord utifrån specifikationen och testerna. Den är ingen mätning och ingen certifiering.
- 0 procent betyder att inget finns ännu. 100 procent betyder byggt, testat och granskat av någon oberoende i en verklig driftsättning. Ingen punkt ligger på 100 procent.
- Varje punkt anger vad dess uppskattning vilar på och vad som återstår.
- Uppskattningarna medelvärdesbildas inte. En enda punkt med låg uppskattning räcker för att hålla WPM utanför produktion.

## Programvaran

- **75%** **Inloggning och åtkomst efter behov av kännedom.** Inloggning är på som standard, och tjänsten vägrar starta i produktion utan den (WPM-R72). Läsning är begränsad till personer med behov av kännedom (WPM-T188), och en token har en maximal ålder (WPM-T200). Vilar på: de tester av tillämpning som körs i CI. Återstår: det finns ingen lista över återkallade token, och inget oberoende penetrationstest har körts.
- **70%** **Ett fullständigt och oförändrat granskningsspår.** Granskningsspåret kan bara utökas och är kedjat med hashvärden, och kan exporteras till ett övervakningssystem (WPM-T189, WPM-T197). Vilar på: tester som medvetet bröts för att visa att de misslyckas. Återstår: en avkortad svans av kedjan upptäcks inte, och en databasägare kan fortfarande stänga av utlösarna.
- **70%** **Dataskydd från början.** Människor identifieras med referenser, det finns inga demografiska kolumner, och det finns registerutdrag, radering och gallring för varje typ av register. Vilar på: testerna och avsnitt 1 i beredskapsbedömningen. Återstår: driftsättarens egen bedömning och rättsliga grund (nedan).
- **75%** **Krypterade anslutningar och säkerhetskopior.** I produktion vägrar tjänsten en okrypterad databasanslutning och avbryter en begäran som hänger sig (WPM-T191), och säkerhetskopior är krypterade (WPM-T192). Vilar på: testerna av båda. Återstår: hanteringen av nycklar och certifikat är driftsättarens.
- **10%** **Lagstadgade beräkningar.** De lagstadgade beräkningarna är exempel, och löneskatten är en platshållare ([den regulatoriska ståndpunkten](../../../spec/regulatory.md)). Vilar på: specifikationen, som säger det. Återstår: reglerna för varje arbetsplats ort, skrivna och kontrollerade av en kvalificerad person. Betala ingen lön från WPM.
- **50%** **Skalbarhet.** På en bärbar dator klarar katalogsidorna och kapacitetsvyn 50 000 medarbetare, och planeringsvyerna vägrar fler än 10 000 (WPM-T196, WPM-T230). Vilar på: en uppmätt körning ([BENCHMARKS.md](../../../BENCHMARKS.md)). Återstår: kapacitetsvyn växer fortfarande i proportion till arbetskraften, och inget har mätts på driftsättarens maskinvara.
- **65%** **Leveranskedja.** Varje CI-åtgärd är fastlåst till en incheckning och historiken skannas efter hemligheter. Containeravbilden är signerad och bär sitt ursprung, och detta kördes för utgåva v0.1.2 (WPM-T194, WPM-T195). Vilar på: den utgivningskörningen. Återstår: avbildspaketet är privat, så ingen utanför projektet kan ännu verifiera det.
- **60%** **Övervakning.** Varje svar bär ett begärans-id, och loggrader innehåller ingen frågesträng och inget personligt id (WPM-T197). Vilar på: testerna av begäranloggen. Återstår: instrumentpaneler och larm är driftsättarens, och hastighetsbegränsningar gäller bara en instans (WPM-T198).
- **60%** **Säkerhetskopiering och återställning.** Krypterade säkerhetskopior, en återställningsövning som körs i CI och en driftbok. Vilar på: övningen. Återstår: en återställningsövning på driftsättarens egen plattform inom de senaste 90 kalenderdagarna (grind WPM-G4).
- **40%** **Tillgänglighetskontroller.** Automatiska kontroller mot WCAG 2.2 AA körs på applikationen och på den här webbplatsen, i ljust och mörkt tema. Vilar på: testerna med Playwright och axe. Återstår: den oberoende granskningen (nedan); ingen skärmläsare har använts.
- **20%** **Översättningar.** Applikationen har 25 språkvarianter och rollbeskrivningarna 21. Vilar på: texten finns. Återstår: en AI skrev översättningarna, och ingen har granskat dem professionellt (WPM-T155).

## Kontroller av andra än upphovspersonen

- **0%** **Oberoende penetrationstest.** Inget penetrationstest av en driftsatt instans har körts (WPM-T201). Vilar på: uppgiften är öppen. Återstår: testet och att åtgärda det som hittas.
- **0%** **Oberoende tillgänglighetsgranskning.** Ingen granskning mot WCAG 2.2 AA tillsammans med personer som använder hjälpmedel har gjorts (WPM-T202). Vilar på: uppgiften är öppen. Återstår: granskningen och att åtgärda det som hittas.
- **25%** **Hotmodell.** En första hotmodell finns ([threat-model.md](../../../spec/governance/threat-model.md)), skriven av upphovspersonen. Vilar på: dokumentet. Återstår: ett oberoende team gör om den för en verklig driftsättning.

## Driftsättarens del

- **20%** **Konsekvensbedömning avseende dataskydd.** Ett paket att anpassa finns: en konsekvensbedömning, ett dataflöde, ett register över behandling och ett lagringsschema ([spec/governance](../../../spec/governance/dpia.md)). Vilar på: paketet. Återstår: det finns ingen driftsättare, så inget dataskyddsombud har godkänt en bedömning, en rättslig grund, ett lagringsschema eller ett personuppgiftsbiträdesavtal (grind WPM-G5).
- **15%** **Bedömningar av konsekvenser för jämlikhet.** En mall och ett register finns ([equality-impact](../../../spec/equality-impact/template.md)). Vilar på: mallen. Återstår: driftsättaren skriver och godkänner en bedömning för eget bruk.
- **40%** **Hosting och drift.** En produktionskonfiguration, en containeravbild och dokument för driftsättning och uppgraderingar finns. Vilar på: de dokumenten och återställningsövningen. Återstår: inget har körts på en härdad värd, och en driftsättare måste driva tjänsten.
- **10%** **Support.** Det finns en ärendehantering och en säkerhetspolicy ([SECURITY.md](../../../SECURITY.md)). Vilar på: de två. Återstår: det finns inget supportavtal, ingen servicenivå, inget jourschema och ingen organisation bakom WPM.

## Vad den här sidan inte hävdar

Sidan dokumenterar inget beslut att gå i drift. Det beslutet tillhör en driftsättare och fattas utanför WPM. Uppskattningarna omfattar bara de punkter som förtecknas, och beredskapsbedömningens egen lista över vad den inte kontrollerade gäller även här. Där ett kliniskt system är anslutet är en klinisk säkerhetsgranskning driftsättarens beslut.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/path-to-production/`) gäller den engelska versionen.
