# Onafhankelijke beveiligingstest

**Perspectief:** hoe krijgen we mensen die deze software niet hebben geschreven zover dat ze haar aanvallen, wat moeten we hun aanwijzen en wat doen we met wat ze vinden?

Er is geen onafhankelijke test uitgevoerd. Het [dreigingsmodel](../../../../spec/governance/threat-model.md) is geschreven door dezelfde assistent die de software schreef en is door niemand anders beoordeeld, en elke beheersmaatregel die het noemt is getest door dezelfde auteur. De [gereedheidsbeoordeling](../../../../spec/readiness-assessment.md) noemt een onafhankelijke penetratietest als een poort die alleen mensen kunnen sluiten (WPM-T201, **niet gedaan**). Deze pagina is een beschrijving voor wie er een in opdracht geeft. Het is geen testplan van een tester en vervangt er geen.

## Wat de testers moeten krijgen

| Onderdeel | Waar het is | Opmerkingen |
| --- | --- | --- |
| Een inzet om aan te vallen | Uw eigen: de [inzetgids](../../../../spec/operations/deployment.md) | Productie-instellingen met aanmelden **aan**, een identiteitsprovider voor tests en **uitsluitend synthetische gegevens**. Test niet tegen de dossiers van echte personen |
| De broncode | Deze repository | Een test met broncode is grondiger dan een zwartedoostest; zeg welke u wilt |
| Het dreigingsmodel | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Vierentwintig dreigingen, de beheersmaatregel zoals gebouwd en de hiaten. Geef het hun **na** een eerste zwartedoosronde als u wilt zien wat ze zonder houvast vinden |
| De API-beschrijving | `GET /api-docs/openapi.json` | Elke route, met `Idempotency-Key` bij elke bewerking die iets wijzigt |
| Het ontwerp van het aanmelden | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Persona's, kenmerken en wat het beleid uitsluitend op actie beslist |
| Testidentiteiten | U maakt ze | Minstens: een medewerker zonder kenmerken, een directe leidinggevende, een leidinggevende buiten de lijn, HR, loonadministratie, een beheerder en een servicepartner |

## Wat hun te laten bekijken

De auteur kon deze zaken niet controleren. Elke rij noemt het item uit het dreigingsmodel zelf, zodat een bevinding te herleiden is.

| Gebied | Wat te beproeven | Dreiging |
| --- | --- | --- |
| Tokens | Verwarring van algoritmen, een sleutelset die tijdens aanroepen roteert, een verlopen token of een dat nog niet geldt, een token voor een andere doelgroep | T1, T2 |
| Need-to-know | Elke leesroute als aanroeper zonder kenmerken en als buur; een route die de classificatietabel niet kent; een schrijfactie die het beleid toestaat maar die de handler zou moeten weigeren | T3, T4, T6 |
| Bevoorrechte kenmerken | Of een kenmerk in een token kan worden toegevoegd, opnieuw afgespeeld of overgeërfd; wie er een kan maken; een gedeelde of langlevende beheerder | T24 |
| Injectie | Fuzz elke parameter, routesegment en inhoudsveld; de opdrachten voor wissen en opschonen zijn gebouwd uit getypeerde identificatoren en zijn gelezen, niet gefuzzd | T12 |
| De browserclient | Sessiebeheer in de interfaceserver, hoe die een sessie omwisselt voor een token, cross-site scripting via opgeslagen tekst, een omzeilde inhoudsbeveiligingsbeleid, de pagina van de inhoudsbeheerder die een script van een openbaar netwerk laadt zonder vastgezette versie | T13, T14 |
| Uitgaande aanroepen | Server-side request forgery via elke ingestelde URL (sleutelsets, zusterdiensten, de gedeelde cache) | T15 |
| Grenzen | De grens op de grootte van de inhoud, de time-out van het verzoek, de aanvraaglimieten per instantie en de weergaven die alle dossiers van een organisatie in het geheugen laden | T16, T17 |
| Logs | Of enig persoonsgegeven, token of queryreeks in een log terechtkomt | T19 |
| Idempotentiesleutels | Een sleutel die door twee aanroepers wordt gebruikt; dezelfde sleutel met gewijzigde inhoud, pad of query; een sleutel die bijna geldig is; een zeer grote inhoud; twee verzoeken die om dezelfde sleutel racen (de bewering is beredeneerd, niet beproefd met een racetest); of een opgeslagen antwoord kan worden gelezen door een ander dan de aanroeper | nieuw |
| Het controlespoor | Elke manier om een item te wijzigen, te verwijderen of over te slaan; het eindpunt voor het verifiëren van de keten | T7 |
| Back-ups | Herstel vanuit een gemanipuleerde back-up of een met de verkeerde sleutel; of een gewiste persoon terugkomt | T10, T11 |
| Toeleveringsketen | Het vastzetten van bouwacties, het scannen op geheimen, het ondertekenen van images | T20, T21 |

## Afspraken om vooraf te maken

Vul deze samen met uw eigen beveiligings- en juridische teams in. Hier is niets beslist.

- **Machtiging:** wie verklaart dat de test geautoriseerd is, en voor welke systemen en adressen.
- **Tijdvenster:** data en tijden, in kalenderdagen, en wie u belt om het te stoppen.
- **Reikwijdte:** wat erbij hoort (de API, de interface, de aanmeldstroom, het back-upscript) en wat er niet bij hoort (de eigen dienst van de identiteitsprovider, uw netwerk, alles wat met andere systemen wordt gedeeld).
- **Gegevens:** uitsluitend synthetisch. Als een tester echte persoonsgegevens vindt: wie wordt ingelicht en hoe ermee wordt omgegaan.
- **Destructieve tests:** of overbelastingsaanvallen, massale schrijfacties en wissingen zijn toegestaan, en waar.
- **Rapportage:** het formaat, de ernstschaal, wie het ontvangt, hoe lang zij het mogen bewaren en hoe geheimen die ze vinden worden behandeld.
- **Hertest:** dat de overeenkomst de controle van de oplossingen omvat.

## Wat u met de bevindingen doet

1. Leg elke bevinding vast als taak in [tasks.md](../../../../spec/tasks.md), met de referentie van de tester, het bewijs en de ernst die zij opgaven. Een bevinding die wordt betwist, blijft vastgelegd als betwist, met de reden.
2. Los op met een test die eerst faalt. Als de oplossing een regel is, voeg die dan toe aan de hiaatkolom van het dreigingsmodel en sluit het hiaat daar pas als de hertest slaagt.
3. Markeer WPM-T201 niet als gereed voordat het rapport in handen is, elke bevinding een eigenaar heeft en een hertest is uitgevoerd. Werk de [gereedheidsbeoordeling](../../../../spec/readiness-assessment.md) en de lijst met poorten (WPM-T203) in dezelfde wijziging bij.
4. Houd het rapport buiten de repository als het iets bevat wat kan worden misbruikt; verwijs naar waar het bewaard wordt.

## Wat een schoon rapport wel en niet betekent

Een test zonder bevindingen toont aan dat deze testers, in dit tijdvenster, met deze reikwijdte en deze identiteiten, niets hebben gevonden. Het toont niet aan dat er geen gebreken zijn, het omvat geen configuratie die u daarna hebt gewijzigd en het vervangt de andere poorten niet: een toegankelijkheidsaudit, uw goedgekeurde gegevensbeschermingseffectbeoordeling en een hersteloefening op uw platform.

## Wat deze pagina niet doet

Zij noemt geen leverancier, stelt geen prijs en belooft geen norm. Welk stelsel of welke accreditatie uw sector van een tester verwacht, is een vraag voor u.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/for/`), dan geldt de Engelse versie.
