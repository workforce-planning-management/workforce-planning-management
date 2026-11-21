# Uafhængig sikkerhedstest

**Perspektiv:** hvordan får vi folk, som ikke har skrevet denne software, til at angribe den, hvad skal
vi pege dem på, og hvad gør vi med det, de finder?

Der er ikke udført nogen uafhængig test. [Trusselsmodellen](../../../../spec/governance/threat-model.md) er skrevet af
den samme assistent, der skrev softwaren, og er ikke gennemgået af andre, og hver kontrol, den nævner, er afprøvet af
den forfatter. [Parathedsvurderingen](../../../../spec/readiness-assessment.md) opregner en uafhængig penetrationstest
som en port, kun mennesker kan lukke (WPM-T201, **ikke udført**). Denne side er en beskrivelse til den, der bestiller en. Den er ikke
en testplan fra en tester og erstatter ikke en.

## Hvad du skal give testerne

| Punkt | Hvor det findes | Bemærkninger |
| --- | --- | --- |
| En driftssætning at angribe | Din egen: [driftsvejledningen](../../../../spec/operations/deployment.md) | Produktionsindstillinger med login **slået til**, en test-identitetsudbyder og **kun syntetiske data**. Test ikke mod rigtige menneskers poster |
| Kildekoden | Dette repositorium | En test med kildekoden er grundigere end en blackbox-test; sig, hvilken du vil have |
| Trusselsmodellen | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Fireogtyve trusler, kontrollen som bygget og manglerne. Giv den til dem **efter** et første blackbox-gennemløb, hvis du vil se, hvad de finder uden hjælp |
| API-beskrivelsen | `GET /api-docs/openapi.json` | Hver route, med `Idempotency-Key` på hver handling, der ændrer noget |
| Designet af login | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Personaer, attributter og det, politikken kun afgør ud fra handlingen |
| Testidentiteter | Du opretter dem selv | Som minimum: en medarbejder uden attributter, en nærmeste leder, en leder uden for kæden, HR, løn, en administrator og en tjeneste på samme niveau |

## Hvad du skal bede dem se på

Forfatteren kunne ikke kontrollere disse. Hver række nævner trusselsmodellens egen post, så et fund kan spores.

| Område | Hvad man bør prøve | Trussel |
| --- | --- | --- |
| Tokens | Forveksling af algoritmer, et nøglesæt, der roteres, mens anmodninger er undervejs, et udløbet eller endnu ikke gyldigt token, et token til en anden målgruppe | T1, T2 |
| Behov for at vide | Hver læseroute som en kalder uden attributter og som en nabo; en route, som klassifikationstabellen ikke kender; en skrivning, som politikken tillader, men som handleren burde afvise | T3, T4, T6 |
| Privilegerede attributter | Om en attribut i et token kan tilføjes, afspilles igen eller nedarves; hvem der kan udstede en; en delt eller langlivet administrator | T24 |
| Injektion | Fuzz hver parameter, hvert stisegment og hvert felt i indholdet; sletnings- og opbevaringsudsagnene er bygget af typede identifikatorer og er læst, ikke fuzzet | T12 |
| Browserklienten | Håndtering af sessioner i brugerfladens server, hvordan den veksler en session til et token, cross-site scripting gennem gemt tekst, omgåelse af indholdssikkerhedspolitikken, indholdsstyringens side, der indlæser et script fra et offentligt netværk uden fastlåst version | T13, T14 |
| Udgående anmodninger | Forfalskning af anmodninger fra serversiden gennem enhver konfigureret URL (nøglesæt, søstertjenesterne, den delte cache) | T15 |
| Grænser | Grænsen for indholdets størrelse, tidsgrænsen for anmodninger, hastighedsgrænserne pr. instans og de visninger, der indlæser en hel organisations poster i hukommelsen | T16, T17 |
| Logge | Om personoplysninger, tokens eller forespørgselsstrenge når ind i en log | T19 |
| Idempotensnøgler | En nøgle brugt af to kaldere; samme nøgle med ændret indhold, sti eller forespørgsel; en nøgle, der næsten er gyldig; meget stort indhold; to anmodninger, der løber om kap under én nøgle (påstanden er ræsonneret, ikke afprøvet med en kapløbstest); om et gemt svar kan læses af andre end kalderen | ny |
| Revisionssporet | Enhver måde at ændre, fjerne eller springe en post over; slutpunktet til verificering af kæden | T7 |
| Sikkerhedskopier | Genopretning fra en manipuleret eller forkert nøglet sikkerhedskopi; om en slettet person vender tilbage | T10, T11 |
| Forsyningskæde | Fastlåsning af build-actions, scanning efter hemmeligheder, signering af images | T20, T21 |

## Regler, der skal på plads, før de begynder

Udfyld disse sammen med dine egne teams for sikkerhed og jura. Intet er afgjort her.

- **Bemyndigelse:** hvem underskriver, at testen er godkendt, og for hvilke systemer og adresser.
- **Tidsvindue:** datoer og tidspunkter, regnet i kalenderdage, og hvem man ringer til for at stoppe den.
- **Omfang:** hvad der er med (API'et, brugerfladen, login-forløbet, scriptet til sikkerhedskopi), og hvad der ikke er med (identitetsudbyderens
  egen tjeneste, dit netværk, alt, der deles med andre systemer).
- **Data:** kun syntetiske. Hvis en tester finder rigtige personoplysninger: hvem der får besked, og hvordan de håndteres.
- **Destruktive tests:** om overbelastningsangreb, massive skrivninger og sletning er tilladt, og hvor.
- **Rapportering:** formatet, skalaen for alvorlighed, hvem der modtager den, hvor længe de må beholde den, og hvordan hemmeligheder,
  de finder, håndteres.
- **Gentest:** at kontrakten omfatter kontrol af rettelserne.

## Hvad du gør med fundene

1. Registrér hvert fund som en opgave i [tasks.md](../../../../spec/tasks.md) med testerens reference, beviset og den
   alvorlighed, de angav. Et fund, der bestrides, forbliver registreret som bestridt, med begrundelsen.
2. Ret med en test, der fejler først. Hvor rettelsen er en regel, føjes den til trusselsmodellens kolonne om mangler og lukkes
   først dér, når gentesten består.
3. Markér ikke WPM-T201 som udført, før rapporten er i hånden, hvert fund har en ejer, og en gentest er gennemført. Opdatér
   [parathedsvurderingen](../../../../spec/readiness-assessment.md) og listen over porte (WPM-T203) i samme ændring.
4. Hold rapporten ude af repositoriet, hvis den rummer noget, der kan udnyttes; link til det sted, hvor den opbevares.

## Hvad en ren rapport betyder og ikke betyder

En test uden fund viser, at disse testere i dette tidsvindue, med dette omfang og disse
identiteter ikke fandt noget.
Den viser ikke, at der ingen fejl er, den dækker ikke en konfiguration, du har ændret bagefter,
og den erstatter ikke de andre porte: en tilgængelighedsrevision, din godkendte konsekvensanalyse
for databeskyttelse og en genopretningsøvelse på din platform.

## Hvad denne side ikke gør

Den nævner ingen leverandør, fastsætter ingen pris og lover ingen standard. Hvilken ordning eller akkreditering din sektor forventer af en
tester, er op til dig at afgøre.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
