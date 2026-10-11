# Für die Sicherheitsleitung

**Die Frage, die Sie wahrscheinlich stellen:** Wie ist dies geschützt, woran würden wir einen Angriff oder Missbrauch erkennen, und was hat niemand getestet?

## Was Sie bekommen

| Anliegen | Was es gibt | Lesen |
| --- | --- | --- |
| Bedrohungen | Ein Bedrohungsmodell: Werte, Akteure, Vertrauensgrenzen, die Kontrolle wie gebaut und das, was **nicht bewertet** wurde | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Authentifizierung | Anmeldung standardmäßig an; die Produktion startet nicht, wenn sie aus ist oder keine Quelle für Token-Schlüssel besteht; Anmeldung über ein Unternehmensverzeichnis (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Autorisierung | Die Richtlinie entscheidet nur nach Aktion; eine klassifizierte Tabelle entscheidet die Kenntnis nur bei Bedarf auf jeder Leseroute, und ein Quellscan-Test schlägt bei einer nicht klassifizierten Route fehl | [auth.md](../../../../spec/auth.md) |
| Prüfprotokoll | Nur anfügbar, hashverkettet, mit Prüfendpunkt; Einträge tragen keinen Begründungstext und keinen Wert | [audit.md](../../../../spec/audit.md) |
| Transport und Browser | Sicherheitsheader bei jeder API-Antwort, eine ausdrückliche CORS-Zulassungsliste, die den Start abbricht, wenn sie fehlt, `no-store` auf `/api` | WPM-R73 in [production-readiness.md](../../../../spec/production-readiness.md) |
| Missbrauch | Ratenbegrenzungen je Aufrufer, strenger bei Löschung, Import, Export und Token-Routen | WPM-R74, dieselbe Datei |
| Unsichere Konfiguration | Der Start lehnt eine unverschlüsselte Datenbank- oder Redis-Verbindung zu einem anderen Host und einen fehlenden CORS-Ursprung ab | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Daten im Ruhezustand | Verschlüsselte Sicherungen; der gemeinsame Cache hält nur nicht personenbezogene Abfragen und scheitert offen | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Lieferkette | Lizenz- und Sperrprüfungen mit `cargo deny`, mitgelieferte gemeinsame Crates, eine Software-Stückliste bei der Veröffentlichung | [production-readiness.md](../../../../spec/production-readiness.md) |
| Vorfälle | Ein Betriebshandbuch mit ersten Maßnahmen; Kontakte und Meldepflichten ergänzen Sie | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Selbstbewertung | Eine Kontroll-Checkliste: erfüllt, teilweise erfüllt, nicht erfüllt, vom Betreiber zu beantworten | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Bekannte Lücken, wie erfasst

- Es wurde kein unabhängiger Penetrationstest durchgeführt. Die Tests schrieb die Autorin oder der Autor.
- Actions sind per Tag fixiert, nicht per Digest, und Images sind nicht signiert.
- Ratenbegrenzungen gelten je Instanz.
- Die Anwendung nutzt die Datenbankrolle, die ihr gegeben wird; ein Satz von Rollen mit geringsten Rechten wird nicht mitgeliefert.
- Das Widerrufen von Token und deren maximales Alter liegen bei Ihrem Identitätsanbieter.
- Der Pfad des Inhaltsmanagers lädt ein Skript von einem öffentlichen CDN ohne fixierte Version und ohne Integritäts-Hash.
- Protokolle wurden nicht auf personenbezogene Daten geprüft, und das Prüfprotokoll wird nicht in ein Überwachungssystem exportiert.
- Verwaltetes Redis (Cluster, Sentinel, Zugriffskontroll-Benutzer) wurde nicht ausprobiert.

## Was nur Menschen tun können

Den unabhängigen Penetrationstest beauftragen, das Bedrohungsmodell für Ihre Umgebung prüfen, eine Wiederherstellungsübung auf Ihrer Plattform durchführen und den Branch-Schutz auf der Hosting-Plattform einrichten. Siehe die [Reifegradbewertung](../../../../spec/readiness-assessment.md), deren Ergebnis für einen Einsatz mit hohem Sicherheitsanspruch **nicht bereit** lautet.

Siehe auch die Seiten der [Risikoleitung](../risikoleitung/index.md) und der [Informationsleitung](../informationsleitung/index.md).

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
