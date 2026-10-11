# Für die Risikoleitung (CRO)

**Die Frage, die Sie wahrscheinlich stellen:** Was könnte mit diesem System schiefgehen, was hält es auf, und was hat niemand geprüft?

## Der ehrliche Ausgangspunkt

Die [Reifegradbewertung](../../../../spec/readiness-assessment.md) vom 2026-10-11 kommt zu dem Ergebnis **nicht bereit** für einen Einsatz mit hohem Sicherheitsanspruch. Sie fand durch Prüfung des laufenden Dienstes ein Hindernis: Jede angemeldete Person konnte die Krankmeldung einer Kollegin oder eines Kollegen samt Freitextgrund lesen. Das ist behoben und getestet. Die Schranken, die nur Menschen schließen können, sind weiterhin offen.

## Was Sie bekommen

| Anliegen | Was es gibt | Lesen |
| --- | --- | --- |
| Bedrohungen an jeder Grenze | Ein Bedrohungsmodell: Werte, Akteure, Grenzen, die Kontrolle wie gebaut und das, was **nicht bewertet** wurde | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Unbefugtes Lesen | Ein Schutz für Kenntnis nur bei Bedarf, der jede Leseroute klassifiziert; ein Test schlägt bei einer nicht klassifizierten fehl | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Manipulation und Rechenschaft | Ein nur anfügbares, hashverkettetes Prüfprotokoll mit Prüfendpunkt; Einträge enthalten weder Begründungstext noch Wert | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Sensible Daten | Keine Diagnose gespeichert; Funktionen für besondere Kategorien standardmäßig aus; eine Untergrenze für kleine Gruppen | [regulatory.md](../../../../spec/regulatory.md) |
| Unsichere Pfade | Die Produktion startet nicht bei ausgeschalteter Anmeldung, ohne Quelle für Token-Schlüssel, bei unverschlüsselter Datenbankverbindung zu einem anderen Host oder bei unverschlüsseltem entferntem Redis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Datenverlust | Verschlüsselte Sicherungen, eine in CI laufende Wiederherstellungsübung, ein Löschverzeichnis | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Verzerrung bei Bewertungen | Eine Vorlage für die Gleichstellungs-Folgenabschätzung und ein Verzeichnis jeder Bewertung mit einem Entwurf der Abschätzung | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Eine Selbstbewertung der Kontrollen | Eine Checkliste, bei der jede Antwort erfüllt, teilweise erfüllt, nicht erfüllt oder vom Betreiber zu beantworten ist | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Offene Risiken, wie erfasst

Fixieren und Signieren der Lieferkette, Lastnachweise, Protokollprüfungen auf personenbezogene Daten, instanzübergreifend geteilte Ratenbegrenzungen, ein Satz von Datenbankrollen mit geringsten Rechten und Hinweise zum Widerruf von Token sind offene Punkte der Reifegradbewertung, jeweils mit einer Aufgabe. Die Tests schrieb die Autorin oder der Autor; keiner ist eine unabhängige Prüfung.

## Was nur Menschen tun können

Ein unabhängiger Penetrationstest, ein unabhängiges Barrierefreiheits-Audit, die unterzeichnete Folgenabschätzung des Betreibers, eine Wiederherstellungsübung auf Ihrer Plattform und die Genehmigung der Gleichstellungs-Folgenabschätzungen. Das Repository führt diese als Schranken und markiert sie nicht als erledigt.

Siehe auch die [Informationsleitung](../informationsleitung/index.md) und die [Rechtsleitung](../rechtsleitung/index.md).

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
