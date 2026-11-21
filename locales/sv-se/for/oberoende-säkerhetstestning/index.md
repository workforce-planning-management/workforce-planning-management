# Oberoende säkerhetstestning

**Perspektiv:** hur får vi människor som inte skrev den här programvaran att angripa den, vad ska vi
peka ut för dem, och vad gör vi med det de hittar?

Inget oberoende test har gjorts. [Hotmodellen](../../../../spec/governance/threat-model.md) skrevs av
samma assistent som skrev programvaran och granskades av ingen annan, och varje kontroll den nämner prövades av
samma upphovsperson. [Beredskapsbedömningen](../../../../spec/readiness-assessment.md) förtecknar ett oberoende penetrationstest
som en grind som bara människor kan stänga (WPM-T201, **inte gjort**). Den här sidan är en beskrivning för den som beställer ett. Den är inte en
testplan från en testare och ersätter inte någon.

## Vad testarna ska få

| Post | Var den finns | Anmärkningar |
| --- | --- | --- |
| En driftsättning att angripa | Din egen: [driftsättningsguiden](../../../../spec/operations/deployment.md) | Produktionsinställningar med inloggning **på**, en identitetsleverantör för test och **endast syntetiska data**. Testa inte mot verkliga personers register |
| Källkoden | Det här arkivet | Ett test med källkoden är grundligare än ett svartlådetest; säg vilket du vill ha |
| Hotmodellen | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Tjugofyra hot, kontrollen som byggd och luckorna. Ge den till dem **efter** ett första svartlådepass om du vill se vad de hittar utan vägledning |
| API-beskrivningen | `GET /api-docs/openapi.json` | Varje rutt, med `Idempotency-Key` på varje operation som ändrar något |
| Inloggningens design | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Personas, attribut och vad policyn avgör enbart efter åtgärd |
| Testidentiteter | Du skapar dem | Åtminstone: en medarbetare utan attribut, en närmaste chef, en chef utanför kedjan, HR, lön, en administratör och en tjänstepartner |

## Vad de ska be dem titta på

Upphovspersonen kunde inte kontrollera dessa. Varje rad anger hotmodellens egen post, så att ett fynd kan spåras.

| Område | Vad som ska prövas | Hot |
| --- | --- | --- |
| Token | Förväxling av algoritm, en nyckeluppsättning som roteras medan anrop pågår, en utgången token eller en som ännu inte gäller, en token för en annan målgrupp | T1, T2 |
| Behov av kännedom | Varje läsrutt som en anropare utan attribut och som en granne; en rutt som klassificeringstabellen inte känner till; en skrivning som policyn tillåter men som hanteraren borde vägra | T3, T4, T6 |
| Privilegierade attribut | Om ett attribut i en token kan läggas till, spelas upp igen eller ärvas; vem som kan skapa ett; en delad eller långlivad administratör | T24 |
| Injektion | Fuzza varje parameter, ruttsegment och kroppsfält; satserna för radering och gallring byggs av typade identifierare och lästes, fuzzades inte | T12 |
| Webbläsarklienten | Sessionshantering i gränssnittsservern, hur den byter en session mot en token, skriptinjektion mellan webbplatser via lagrad text, en förbigången innehållssäkerhetspolicy, innehållshanterarens sida som läser in ett skript från ett offentligt nätverk utan fastlåst version | T13, T14 |
| Utgående anrop | Förfalskning av serversidans begäran via vilken konfigurerad URL som helst (nyckeluppsättningar, systertjänster, den delade cachen) | T15 |
| Gränser | Gränsen för kroppsstorlek, tidsgränsen för begäran, hastighetsbegränsningarna per instans och de vyer som läser in en hel organisations register i minnet | T16, T17 |
| Loggar | Om någon personuppgift, token eller frågesträng når en logg | T19 |
| Idempotensnycklar | En nyckel som används av två anropare; samma nyckel med ändrad kropp, väg eller fråga; en nyckel som nästan är giltig; en mycket stor kropp; två begäranden som tävlar under en nyckel (påståendet är resonerat, inte prövat med ett kapplöpningstest); om ett lagrat svar kan läsas av någon annan än den som anropade | ny |
| Granskningsspåret | Varje sätt att ändra, ta bort eller hoppa över en post; ändpunkten för verifiering av kedjan | T7 |
| Säkerhetskopior | Återställning från en manipulerad säkerhetskopia eller en med fel nyckel; om en raderad person kommer tillbaka | T10, T11 |
| Leveranskedja | Fastlåsning av byggåtgärder, hemlighetsskanning, signering av avbilder | T20, T21 |

## Regler att komma överens om innan de börjar

Fyll i dessa tillsammans med dina egna säkerhets- och juristteam. Inget är avgjort här.

- **Behörighet:** vem som intygar att testet är auktoriserat, och för vilka system och adresser.
- **Tidsfönster:** datum och tider, räknade i kalenderdagar, och vem man ringer för att stoppa det.
- **Omfattning:** vad som ingår (API:et, gränssnittet, inloggningsflödet, skriptet för säkerhetskopiering) och vad som inte ingår (identitetsleverantörens
  egen tjänst, ditt nätverk, allt som delas med andra system).
- **Data:** endast syntetiska. Om en testare hittar verkliga personuppgifter: vem som underrättas och hur de hanteras.
- **Destruktiva tester:** om överbelastningsattacker, massskrivningar och radering är tillåtna, och var.
- **Rapportering:** formatet, allvarlighetsskalan, vem som tar emot den, hur länge de får behålla den och hur hemligheter
  som de hittar hanteras.
- **Omtest:** att avtalet omfattar kontroll av åtgärderna.

## Vad man gör med fynden

1. Dokumentera varje fynd som en uppgift i [tasks.md](../../../../spec/tasks.md) med testarens referens, belägget och
   den allvarlighet de angav. Ett fynd som bestrids förblir dokumenterat som bestritt, med skälet.
2. Åtgärda med ett test som först misslyckas. Där åtgärden är en regel, lägg till den i hotmodellens lucka-kolumn och stäng den
   där först när omtestet godkänns.
3. Markera inte WPM-T201 som klar förrän rapporten finns i handen, varje fynd har en ansvarig och ett omtest är genomfört. Uppdatera
   [beredskapsbedömningen](../../../../spec/readiness-assessment.md) och listan över grindar (WPM-T203) i samma ändring.
4. Håll rapporten utanför arkivet om den innehåller något som kan utnyttjas; länka till var den förvaras.

## Vad en ren rapport betyder och inte betyder

Ett test utan fynd visar att dessa testare, under detta tidsfönster, med denna omfattning och dessa
identiteter, inte hittade något.
Det visar inte att brister saknas, det omfattar inte en konfiguration som du ändrade efteråt,
och det ersätter inte de andra grindarna: en tillgänglighetsgranskning, din godkända konsekvensbedömning
avseende dataskydd och en återställningsövning på din plattform.

## Vad den här sidan inte gör

Den nämner ingen leverantör, sätter inget pris och utlovar ingen standard. Vilket system eller vilken ackreditering din bransch förväntar sig av en
testare är en fråga för dig.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/for/`) gäller den engelska versionen.
