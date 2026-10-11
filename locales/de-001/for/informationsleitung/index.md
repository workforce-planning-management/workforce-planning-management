# Für die Informationsleitung (CIO)

**Die Frage, die Sie wahrscheinlich stellen:** Wie fügt sich dies in unsere Landschaft ein, wer kann sich anmelden, wo liegen die Daten, und wie betreiben und stellen wir es wieder her?

## Was Sie bekommen

| Anliegen | Was es gibt | Lesen |
| --- | --- | --- |
| Aufbau | Eine JSON-API und ein separater Browser-Client; PostgreSQL hält die Daten | [architecture.md](../../../../spec/architecture.md) |
| Identität | WPM hält keine Identitäten. Person, Beschäftigte und Organisation sind Verweise auf andere Dienste, nur dem Namen nach aufgelöst und zwischengespeichert | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Anmeldung | Standardmäßig an. Anmeldung über ein Unternehmensverzeichnis (OIDC, Entra-kompatibel, direkt oder über Keycloak); Rollen und Attribute entscheiden über den Zugriff | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Bereitstellung | Ein Container-Build und eine Compose-Datei für eine Demonstration; für die Produktion ein TLS-terminierender Proxy, erforderliche Einstellungen und Startprüfungen, die ein unsicheres Setup ablehnen | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Gemeinsamer Cache | Optionales Redis nur für nicht personenbezogene Abfragen; jeder Aufruf scheitert offen | `REDIS_URL` im Bereitstellungsleitfaden (WPM-D78) |
| Sicherung und Wiederherstellung | Verschlüsselte Sicherungen, ein Wiederherstellungsskript und eine Wiederherstellungsübung, die CI ausführt | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Aktualisierung | Migrationen und ein Aktualisierungsleitfaden | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Wenn etwas schiefgeht | Ein Betriebshandbuch mit ersten Maßnahmen | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Datenbewegung | Ein Datenflussdokument | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Was Sie vor der Planung wissen sollten

- Die Oberfläche ist **nicht containerisiert**; Sie wählen einen Adapter für Ihre Plattform.
- Ratenbegrenzungen gelten je Instanz. Einen gemeinsamen Speicher müssen Sie bereitstellen.
- Die Anwendung nutzt die Datenbankrolle, die `DATABASE_URL` nennt; ein Satz von Rollen mit geringsten Rechten wird noch nicht mitgeliefert.
- Das Widerrufen von Token und deren maximales Alter liegen bei Ihrem Identitätsanbieter.
- Verwaltetes Redis (Cluster, Sentinel, Zugriffskontroll-Benutzer) wurde nicht ausprobiert.
- Es gibt keine Lastmessung, und einige Ansichten lesen eine Organisation in den Speicher.
- Der Image-Build und die Compose-Datei wurden geschrieben, sollten aber zuerst in Ihrer Umgebung erprobt werden.

Die vollständige Liste der Lücken und das Ergebnis stehen in [readiness-assessment.md](../../../../spec/readiness-assessment.md); die technische Sicht bietet die Seite der [Technikleitung](../technikleitung/index.md).

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
