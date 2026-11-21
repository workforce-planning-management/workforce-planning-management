# Voor de IT-directeur (CIO)

**Perspectief:** hoe past dit in ons landschap, wie kan zich aanmelden, waar staan de gegevens en hoe beheren en herstellen we het?

## Wat u krijgt

| Aandachtspunt | Wat er is | Lees |
| --- | --- | --- |
| Opzet | Een JSON-API en een aparte browserclient; PostgreSQL bewaart de gegevens | [architecture.md](../../../../spec/architecture.md) |
| Identiteit | WPM bewaart geen identiteiten. Een persoon, een medewerker en een organisatie zijn verwijzingen naar andere diensten, alleen op naam opgelost en in een cache bewaard | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Aanmelden | Standaard aan. Aanmelden via de bedrijfsmap (OIDC, compatibel met Entra, rechtstreeks of via Keycloak); rollen en kenmerken bepalen de toegang | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Inzet | Een containerbouw en een compose-bestand voor een demonstratie; voor productie een proxy die TLS afhandelt, vereiste instellingen en opstartcontroles die een onveilige configuratie weigeren | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Gedeelde cache | Optionele Redis, uitsluitend voor niet-persoonlijke opzoekingen; elke aanroep valt open bij een storing | `REDIS_URL` in de inzetgids (WPM-D78) |
| Back-up en herstel | Versleutelde back-ups, een herstelscript en een hersteloefening die CI uitvoert | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Upgrade | Migraties en een upgradegids | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Wanneer het misgaat | Een draaiboek met eerste handelingen | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Gegevensverkeer | Een document over de gegevensstroom | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Wat u moet weten voordat u plant

- De UI is **niet in een container verpakt**; u kiest een adapter voor uw platform.
- Aanvraaglimieten gelden per instantie. Een gedeelde opslag moet u zelf leveren.
- De applicatie gebruikt de databaserol die `DATABASE_URL` noemt; een rolset met minimale rechten is nog niet meegeleverd.
- Intrekking van tokens en de maximale leeftijd van een token zijn aan uw identiteitsprovider.
- Beheerde Redis (cluster, sentinel, gebruikers met toegangsbeheer) is niet geprobeerd.
- Er is geen belastingmeting, en sommige weergaven lezen een hele organisatie in het geheugen.
- De imagebouw en het compose-bestand zijn geschreven, maar moeten eerst in uw omgeving worden uitgeprobeerd.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/for/`), dan geldt de Engelse versie.
