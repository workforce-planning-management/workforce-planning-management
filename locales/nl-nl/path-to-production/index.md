# De weg naar productie

WPM is demonstratiesoftware. Deze pagina beschrijft wat productiegebruik voor WPM zou betekenen en somt op wat er tussen WPM en dat gebruik staat. Bij elk punt hoort een schatting van hoe ver het is, van 0 tot 100 procent.

## Wat productie betekent

Productiegebruik betekent dat een organisatie voor echt werk op WPM vertrouwt:

- de dossiers gaan over echte mensen: medewerkers, sollicitanten en hun noodcontacten;
- loon dat WPM berekent wordt uitbetaald, en het recht van de plaats van het werk is erop van toepassing;
- mensen melden zich elke dag aan en een deel van wat ze lezen is gevoelig, zoals afwezigheid wegens ziekte;
- de organisatie legt verantwoording af aan een toezichthouder, een accountant en de mensen in de dossiers over hoe zij wordt bestuurd.

Deze pagina meet WPM af aan het zwaarste geval waarvoor het werd beoordeeld: een grote medische of overheidsorganisatie ([de gereedheidsbeoordeling](../../../spec/readiness-assessment.md)). Een kleinere organisatie kan van sommige punten minder nodig hebben, maar zij heeft hetzelfde soort bewijs nodig.

Productie vraagt drie soorten bewijs, en alleen de software levert het eerste:

1. **Software** die het werk veilig doet, met tests die dat aantonen.
2. **Een inzetter** die haar host en beheert: back-ups, monitoring, ondersteuning en incidentafhandeling.
3. **Anderen dan de auteur** die haar hebben beoordeeld: een penetratietester, een toegankelijkheidsbeoordelaar, vertalers en de functionaris voor gegevensbescherming van de inzetter.

## Hoe u de schattingen leest

- Een schatting is de beoordeling van de auteur op 2026-11-13, gemaakt op basis van de specificatie en de tests. Het is geen meting en geen certificering.
- 0 procent betekent dat er nog niets is. 100 procent betekent gebouwd, getest en beoordeeld door een onafhankelijke partij in een echte inzet. Geen enkel punt staat op 100 procent.
- Elk punt zegt waarop zijn schatting berust en wat er nog resteert.
- De schattingen worden niet gemiddeld. Eén enkel punt met een lage schatting is genoeg om WPM uit productie te houden.

## De software

- **75%** **Aanmelden en toegang op basis van need-to-know.** Aanmelden staat standaard aan en de service weigert in productie te starten zonder (WPM-R72). Lezen is beperkt tot personen met need-to-know (WPM-T188) en een token heeft een maximale leeftijd (WPM-T200). Berust op: de afdwingingstests die in CI draaien. Resteert: er is geen lijst van ingetrokken tokens en er is geen onafhankelijke penetratietest uitgevoerd.
- **70%** **Een volledig en ongewijzigd controlespoor.** Het controlespoor kan alleen worden aangevuld, is met hashes geketend en kan naar een monitoringsysteem worden geëxporteerd (WPM-T189, WPM-T197). Berust op: tests die met opzet zijn gebroken om te laten zien dat ze falen. Resteert: een ingekorte staart van de keten wordt niet opgemerkt en een eigenaar van de database kan de triggers nog uitschakelen.
- **70%** **Gegevensbescherming vanaf het ontwerp.** Mensen worden aangeduid met verwijzingen, er zijn geen demografische kolommen en er zijn inzage, wissing en opschoning voor elke soort dossier. Berust op: de tests en sectie 1 van de gereedheidsbeoordeling. Resteert: de eigen beoordeling en rechtmatige grondslag van de inzetter (hieronder).
- **75%** **Versleutelde verbindingen en back-ups.** In productie weigert de service een niet-versleutelde databaseverbinding en breekt een verzoek af dat blijft hangen (WPM-T191), en back-ups zijn versleuteld (WPM-T192). Berust op: de tests van beide. Resteert: het beheer van sleutels en certificaten is aan de inzetter.
- **10%** **Wettelijke berekeningen.** De wettelijke berekeningen zijn voorbeelden en de loonbelasting is een plaatsvervanger ([het regelgevende standpunt](../../../spec/regulatory.md)). Berust op: de specificatie, die dat zegt. Resteert: de regels van de plaats van elk werk, geschreven en gecontroleerd door een bevoegd persoon. Betaal geen loon vanuit WPM.
- **50%** **Schaalbaarheid.** Op een laptop verwerken de cataloguspagina's en het capaciteitsoverzicht 50.000 medewerkers, en de planningsoverzichten weigeren er meer dan 10.000 (WPM-T196, WPM-T230). Berust op: een gemeten run ([BENCHMARKS.md](../../../BENCHMARKS.md)). Resteert: het capaciteitsoverzicht groeit nog steeds evenredig met het personeelsbestand en op de hardware van de inzetter is niets gemeten.
- **65%** **Toeleveringsketen.** Elke CI-actie is vastgezet op een commit en de geschiedenis wordt gescand op geheimen. Het containerimage is ondertekend en draagt zijn herkomst, en dit is uitgevoerd voor release v0.1.2 (WPM-T194, WPM-T195). Berust op: die releaserun. Resteert: het imagepakket is privé, dus niemand buiten het project kan het nog verifiëren.
- **60%** **Monitoring.** Elk antwoord draagt een verzoek-id en logregels bevatten geen queryreeks en geen persoonlijke id (WPM-T197). Berust op: de tests van het verzoeklogboek. Resteert: dashboards en alarmen zijn aan de inzetter, en aanvraaglimieten gelden maar voor één instantie (WPM-T198).
- **60%** **Back-up en herstel.** Versleutelde back-ups, een hersteloefening die in CI draait en een draaiboek. Berust op: de oefening. Resteert: een hersteloefening op het eigen platform van de inzetter binnen de laatste 90 kalenderdagen (poort WPM-G4).
- **40%** **Toegankelijkheidscontroles.** Geautomatiseerde controles tegen WCAG 2.2 AA draaien op de applicatie en op deze website, in licht en donker thema. Berust op: de tests met Playwright en axe. Resteert: de onafhankelijke beoordeling (hieronder); er is geen schermlezer gebruikt.
- **20%** **Vertalingen.** De applicatie heeft 25 taalvarianten en de rolbeschrijvingen 21. Berust op: de tekst bestaat. Resteert: een AI schreef de vertalingen en niemand heeft ze professioneel beoordeeld (WPM-T155).

## Controles door anderen dan de auteur

- **0%** **Onafhankelijke penetratietest.** Er is geen penetratietest van een ingezet exemplaar uitgevoerd (WPM-T201). Berust op: de taak staat open. Resteert: de test en het verhelpen van wat wordt gevonden.
- **0%** **Onafhankelijke toegankelijkheidsbeoordeling.** Er is geen beoordeling tegen WCAG 2.2 AA gedaan samen met mensen die hulpmiddelen gebruiken (WPM-T202). Berust op: de taak staat open. Resteert: de beoordeling en het verhelpen van wat wordt gevonden.
- **25%** **Dreigingsmodel.** Een eerste dreigingsmodel bestaat ([threat-model.md](../../../spec/governance/threat-model.md)), geschreven door de auteur. Berust op: het document. Resteert: een onafhankelijk team maakt het opnieuw voor een echte inzet.

## Het deel van de inzetter

- **20%** **Gegevensbeschermingseffectbeoordeling.** Er is een pakket om aan te passen: een effectbeoordeling, een gegevensstroom, een register van verwerkingen en een bewaarschema ([spec/governance](../../../spec/governance/dpia.md)). Berust op: het pakket. Resteert: er is geen inzetter, dus geen functionaris voor gegevensbescherming heeft een beoordeling, een rechtmatige grondslag, een bewaarschema of een verwerkersovereenkomst goedgekeurd (poort WPM-G5).
- **15%** **Beoordelingen van gelijkheidseffecten.** Er zijn een sjabloon en een register ([equality-impact](../../../spec/equality-impact/template.md)). Berust op: het sjabloon. Resteert: de inzetter schrijft en keurt een beoordeling goed voor eigen gebruik.
- **40%** **Hosting en beheer.** Er zijn een productieconfiguratie, een containerimage en documenten over inzet en upgrades. Berust op: die documenten en de hersteloefening. Resteert: niets is uitgevoerd op een geharde host en een inzetter moet de dienst beheren.
- **10%** **Ondersteuning.** Er zijn een issuetracker en een beveiligingsbeleid ([SECURITY.md](../../../SECURITY.md)). Berust op: die twee. Resteert: er is geen ondersteuningsovereenkomst, geen serviceniveau, geen bereikbaarheidsrooster en geen organisatie achter WPM.

## Wat deze pagina niet beweert

De pagina legt geen besluit vast om in productie te gaan. Dat besluit is aan een inzetter en wordt buiten WPM genomen. De schattingen betreffen alleen de genoemde punten, en de eigen lijst van de gereedheidsbeoordeling met wat zij niet controleerde geldt ook hier. Waar een klinisch systeem is gekoppeld, is een klinische veiligheidsbeoordeling een besluit van de inzetter.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/path-to-production/`), dan geldt de Engelse versie.
