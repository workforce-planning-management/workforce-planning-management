# Für die Personalleitung (CPO)

**Die Frage, die Sie wahrscheinlich stellen:** Unterstützt dies den gesamten Lebenszyklus der Beschäftigten, und kann ich es nutzen, ohne mehr über Menschen zu halten, als ich sollte?

## Was Sie bekommen

| Phase | Was WPM hält | Lesen |
| --- | --- | --- |
| Einstellung | Stellenanforderungen, eine Bewerbungspipeline, ein Kandidatenpool, Einarbeitungs-Checklisten | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| Der Datensatz | Ein Beschäftigungsdatensatz je Person, das Organigramm, Berichtslinien, Gruppen, Wechsel zwischen Organisationen | [hr-core.md](../../../../spec/hr-core.md) |
| Selbstbedienung | Eine Person sieht und ändert ihre eigenen Angaben, beantragt flexibles Arbeiten, kündigt an und sieht ihre Abwesenheiten | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Arbeitsleben | Urlaub, Zeit, Schichten, Leistungen, Wohlbefindens-Hinweise, angemessene Anpassungen, ergonomische Beurteilungen, der anonyme Puls | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Entwicklung | Beurteilungen, 360-Grad-Beurteilungen, Kurse über das Kursregister, Assessments, Mentoring, Entwicklungspläne, Nachfolge, Berufseinstieg | [talent-development.md](../../../../spec/talent-development.md) |
| Vergütung | Abrechnungsläufe, Gehaltsabrechnungen, Gehaltsvergleiche, Nachschlagen einer nationalen Gehaltstabelle, Stellenstufen | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Nicht unbefristete Kräfte | Enddaten je Grundlage, Verlängerungen, Statusbeurteilungen, von einem Menschen entschiedene Umwandlungspläne, Finanzierung neben dem Vertragsende | [engagements.md](../../../../spec/engagements.md) |
| Austritt | Kündigung mit Kündigungsfrist, eine Übergabe am letzten Tag, Aufbewahrung je Datensatzart, Löschung | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Wie es Menschen schützt

- **Kenntnis nur bei Bedarf beim Lesen.** Eine Führungskraft sieht ihre eigenen Mitarbeitenden, eine Person sich selbst, die Personalabteilung das, was sie braucht. Eine angemeldete fremde Person sieht nichts über eine Kollegin oder einen Kollegen ([auth.md](../../../../spec/auth.md)).
- **Keine Diagnose wird gespeichert.** Bei Krankheit wird eine Art erfasst, kein Grund.
- **Funktionen für besondere Kategorien sind standardmäßig aus.** Gleichstellungsmonitoring und Gesundheitsanforderungen am Arbeitsplatz brauchen eine erfasste Rechtsgrundlage, sind freiwillig oder liegen bei der betriebsärztlichen Rolle und zeigen nur Gruppen oberhalb einer Untergrenze. Sie fließen nie in eine Bewertung oder Entscheidung über eine Person ein.
- **Bewertungen beraten.** Nachfolgebereitschaft, Kompetenzlücken und Prognosen schlagen vor; ein Mensch entscheidet, und Entscheidungen wie die Umwandlung eines freiberuflichen Einsatzes werden als Entscheidung eines Menschen erfasst, nie als automatisches Ergebnis.
- **Die betroffene Person kann einsehen und löschen lassen.** Der Auskunftsexport und die Löschung decken die obigen Datensätze ab, mit einem Verzeichnis, damit eine Wiederherstellung keine gelöschte Person zurückbringt.

## Grenzen

- Noch nicht gebaut: eine Benutzeroberfläche für Umwandlungspläne und Finanzierung, der Trend bei flexiblen Kräften und die Seite „Meine Daten“ der Beschäftigten. Den aktuellen Stand finden Sie in [tasks.md](../../../../spec/tasks.md).
- Übersetzungen sind maschinell unterstützt; eine professionelle Prüfung ist eine offene Schranke.
- Tarifverträge und anerkannte Gremien sind nur Recherche und Vorschlag ([worker-unions.md](../../../../spec/worker-unions.md)).
- Die Lohnsteuer ist ein Platzhalter.

Siehe auch die [Rechtsleitung](../rechtsleitung/index.md) und die [Geschäftsführung](../geschäftsführung/index.md).

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
