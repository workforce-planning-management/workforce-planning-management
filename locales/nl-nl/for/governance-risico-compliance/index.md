# Governance, risico en compliance

**Perspectief:** wie is waarvoor verantwoordelijk, wat kan er misgaan en wat houdt dat tegen, en welk bewijs kan ik aan een beoordelaar tonen? Deze pagina brengt het materiaal over governance, risico en compliance op één plaats samen. De rolpagina's voor de risicodirecteur en de juridisch directeur behandelen dezelfde onderwerpen vanuit hun eigen rol.

## Governance: wie beslist, en hoe het wordt vastgelegd

| Onderwerp | Wat er is | Lees |
| --- | --- | --- |
| Hoe het werk wordt aangestuurd | Eerst de specificatie: een eis, een beslissing, een taak, code en tests worden samen opgeleverd; elke taak zegt wat **niet** is geverifieerd | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Bestuur van het project | Bestuur, beheerders, een RFC-proces en een bestand met codeverantwoordelijken | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Beslissingen over mensen | Een mens beslist; scores geven onderbouwing. Omzettingen, ontslagen en verzoeken om flexibel werken worden vastgelegd als de beslissing van een mens | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Toegang en rollen | Rollen en kenmerken bepalen de toegang; een geclassificeerde tabel bepaalt need-to-know op elke leesroute | [auth.md](../../../../spec/auth.md) |
| Auteurschap met hulp van AI | Een verklaring over hoe AI is gebruikt | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Risico: wat kan er misgaan

| Onderwerp | Wat er is | Lees |
| --- | --- | --- |
| Oordeel | **Niet gereed** voor een inzet met hoge zekerheidseisen; een blokkade (lezen was niet gebaseerd op need-to-know) is door onderzoek gevonden en verholpen | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Bedreigingen | Een dreigingsmodel: bedrijfsmiddelen, actoren, grenzen, de beheersmaatregel zoals gebouwd en wat **niet is beoordeeld** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Sporen van manipulatie | Een spoor dat alleen kan worden aangevuld en met hashes is geketend, met een verificatie-eindpunt; items bevatten geen redentekst en geen waarde | [audit.md](../../../../spec/audit.md) |
| Veilig opstarten | Productie weigert te starten met aanmelden uitgeschakeld, zonder tokensleutels, met een niet-versleutelde databaseverbinding naar een andere host of met een niet-versleutelde externe Redis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Herstel | Versleutelde back-ups, een hersteloefening die CI uitvoert, een wissingslogboek zodat een herstel een gewiste persoon niet terugbrengt | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Incidenten | Een draaiboek met eerste handelingen; wie inzet voegt contactpersonen en escalatieroutes toe | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Vooringenomenheid bij scoring | Een sjabloon voor gelijkheidseffecten en een register van elke score, elk met een voorlopige beoordeling | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Openstaande risico's | Vastzetten en ondertekenen van de toeleveringsketen, bewijs van belasting, logcontroles op persoonsgegevens, gedeelde aanvraaglimieten, een rolset voor de database met minimale rechten, richtlijnen voor het intrekken van tokens | [readiness-assessment.md](../../../../spec/readiness-assessment.md), sectie 3 |

## Compliance: wat u een beoordelaar kunt tonen

| Onderwerp | Wat er is | Lees |
| --- | --- | --- |
| Zelfbeoordeling van beheersmaatregelen | Een checklist waarin elk antwoord voldaan, gedeeltelijk voldaan, niet voldaan of door de inzetter te beantwoorden is | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Gegevensbeschermingseffectbeoordeling | Een sjabloon, voor de demo ingevuld, met de door uzelf in te vullen delen gemarkeerd | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Register van verwerkingen | Een aanpasbaar register | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Gegevensstroom | Waar persoonsgegevens naartoe gaan | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Bewaartermijn | Een schema per soort dossier | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Rechten van de betrokkene | Inzage en wissing | [regulatory.md](../../../../spec/regulatory.md) |
| Bijzondere categorieën persoonsgegevens | Gelijkheidsmonitoring en vereisten voor gezondheid op het werk staan uit totdat een rechtmatige grondslag is vastgelegd, en zijn daarna begrensd | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Toegankelijkheid en taal | Een toegankelijkheidsverklaring en een verklaring over officiële talen | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Licenties en afhankelijkheden | Licentiebestanden, een afhankelijkhedenbeleid dat met `cargo deny` wordt gecontroleerd en een software bill of materials bij elke release | [production-readiness.md](../../../../spec/production-readiness.md) |

## Wat alleen mensen kunnen doen

Dit zijn poorten, geen code, en de repository markeert ze niet als gereed: een onafhankelijke penetratietest, een onafhankelijke toegankelijkheidsaudit, uw gegevensbeschermingseffectbeoordeling goedgekeurd door uw functionaris voor gegevensbescherming, een hersteloefening op uw platform, een professionele beoordeling van de vertalingen en uw goedkeuring van de beoordelingen van gelijkheidseffecten.

## De beperkingen van deze pagina

De tabellen wijzen naar documenten. Of elk document klopt voor uw inzet, is een kwestie voor u en uw beoordelaar. De tests van de auteur zijn geen onafhankelijke beoordeling, en niets hier is juridisch advies of een certificering.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/for/`), dan geldt de Engelse versie.
