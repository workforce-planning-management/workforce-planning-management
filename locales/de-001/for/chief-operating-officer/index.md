# Für die Betriebsleitung (COO)

**Die Frage, die Sie wahrscheinlich stellen:** Haben wir die richtigen Leute am richtigen Ort zur richtigen Zeit, und was bricht weg, wenn jemand fehlt?

## Was Sie bekommen

| Sie wollen wissen | Wo WPM antwortet | Lesen |
| --- | --- | --- |
| Ob ein Team Abwesenheit auffangen kann | Vertretungen je Person, der Bereitschaftsplan mit Tausch und Erinnerungen und eine Vertretungsansicht, die klar sagt, wenn niemand verfügbar ist | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Zeit, Anwesenheit und Urlaub | Zeiteinträge, Urlaubsanträge mit Genehmigungsweg, Ansprüche und Schichten | [workforce-management.md](../../../../spec/workforce-management.md) |
| Ob Arbeitszeitgrenzen eingehalten werden | Beratende Arbeitszeit-Leitplanken; sie warnen und blockieren nicht | WPM-R27, dieselbe Datei |
| Kapazität nach Fähigkeit, Monat für Monat | Fähigkeitspools, Angebot und Programmbedarf, Zusagen von Partnern, der Engpasspool und eine Startprüfung, die ein Vorschlag ist, mit erfassten Startentscheidungen | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Was wir brauchen werden und wo die Lücke liegt | Personalpläne, Prognose und Lückenanalyse nach Abteilung und Rolle | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Welche Verträge bald enden | Ein Enddatum je Einsatzgrundlage, Verlängerungen mit Verlauf, Erinnerungen vor dem Ende, eine Liste von Einsätzen ohne Enddatum und Umwandlungspläne | [engagements.md](../../../../spec/engagements.md) |
| Wo flexible Kräfte einen großen Anteil haben | Der Anteil der FTE, der freiberuflich ist, und der, der befristet ist, jeweils getrennt, und die Finanzierung, die Monat für Monat endet | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Eintritte und Austritte | Datierte Checklisten und eine Übergabe am letzten Tag | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Kompetenzlücken und Schulungszeit | Lückenanalyse mit der Evidenz, auf der eine Empfehlung beruht | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Wie die Kapazitätsansicht zu lesen ist

Kapazität wird nach Fähigkeitspool geplant, nicht nach Person. Jeder Pool-Monat ergibt eines von vier Ergebnissen, und eine nicht angegebene Fähigkeit ist *unbekannt*, kein Fehlbestand. Die Startprüfung ist ein Vorschlag, und eine Person erfasst die Entscheidung zu starten. Kritische Kette und Planung des kritischen Pfads sind nicht hier: Sie gehören zu project-portfolio-management.

## Grenzen

- Der flexible Anteil hat noch keinen Trend: Er braucht Personalbestands-Momentaufnahmen nach Grundlage (WPM-T139, nicht gebaut).
- Ein Pool ist in der Ansicht zu flexiblen Kräften die Abteilung, bis Pläne Pools definieren.
- Schichtplanung und Lohnsteuerberechnungen ersetzen nicht Ihre Dienstplan- oder Lohnsysteme.
- Mehrere Ansichten laden die Datensätze einer Organisation in den Speicher; es wurde keine Lastzahl gemessen ([readiness-assessment.md](../../../../spec/readiness-assessment.md), Lücke 8).

Siehe auch die [Personalleitung](../chief-people-officer/index.md) und die [Geschäftsführung](../chief-executive-officer/index.md).

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
