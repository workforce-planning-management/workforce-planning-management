# Für die Rechtsleitung (CLO)

**Die Frage, die Sie wahrscheinlich stellen:** Welche rechtlichen Pflichten schafft diese Software oder unterstützt sie, und was müssen wir entscheiden, bevor echte Daten hineingehen?

> Nichts hier ist Rechtsberatung. Die Spezifikation beschreibt eine Haltung, keine Schlussfolgerung für eine bestimmte Rechtsordnung.

## Was Sie bekommen

| Thema | Was es gibt | Lesen |
| --- | --- | --- |
| Haltung | Haltung zu UK GDPR und Beschäftigtendaten; Demo-Status; was die Produktion zusätzlich bräuchte | [regulatory.md](../../../../spec/regulatory.md) |
| Folgenabschätzung | Eine Vorlage für die Datenschutz-Folgenabschätzung, für die Demo ausgefüllt, mit markierten Teilen, die Sie selbst ausfüllen müssen | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Verarbeitungsverzeichnis und Datenfluss | Anpassbare Dokumente | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md), [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Aufbewahrung | Ein Plan je Datensatzart, bei einigen mit Bereinigungslauf | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Betroffenenrechte | Auskunftsexport, Löschung und ein Löschverzeichnis | [regulatory.md](../../../../spec/regulatory.md) (WPM-R30) |
| Daten besonderer Kategorien | Gleichstellungsmonitoring und Gesundheitsanforderungen am Arbeitsplatz sind aus, bis Sie eine Rechtsgrundlage erfassen, und dann begrenzt: freiwillig, nur für die Person selbst oder rollenbeschränkt, Aggregate oberhalb einer Untergrenze, nie in einer Entscheidung verwendet | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Gesetzliche Rechte | Anträge auf flexibles Arbeiten mit einer Entscheidungsfrist und Gründen, die Sie konfigurieren; eine Kündigungsfrist, die Sie konfigurieren | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Gleichstellungspflichten | Eine Vorlage für die Gleichstellungs-Folgenabschätzung und ein Verzeichnis jeder Bewertung | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| Barrierefreiheit und Sprache | Eine Erklärung zur Barrierefreiheit und eine Erklärung zur Amtssprache | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Kollektive Gremien | Recherche und Vorschlag zu anerkannten Gremien und Konsultation, nie zur individuellen Mitgliedschaft | [worker-unions.md](../../../../spec/worker-unions.md) |
| Freiberufliche Kräfte | Einsatzgrundlage, Lieferant, Weg und maskierter Satz werden erfasst. Ob ein Einsatz rechtmäßig oder richtig eingestuft ist, entscheidet die Software nicht | [engagements.md](../../../../spec/engagements.md) |
| Lizenzen | Lizenzdateien, eine Abhängigkeitsrichtlinie und eine Software-Stückliste bei der Veröffentlichung | [production-readiness.md](../../../../spec/production-readiness.md) |

## Voreinstellungen, die kein Recht sind

Mehrere Einstellungen sind Voreinstellungen, keine Rechtsberatung: die Kündigungsfrist, die Entscheidungs- und Widerspruchsfrist beim flexiblen Arbeiten und die Ablehnungsgründe. Der Bereitstellungsleitfaden verlangt, sie für Ihre Rechtsordnung zu **bestätigen**. Gesetzliche Berechnungen wie die Lohnsteuer sind nicht umgesetzt.

## Entscheidungen, die bei Ihnen liegen

Die Rechtsgrundlage für jeden Verarbeitungszweck, die Datenschutz-Folgenabschätzung und ihre Unterzeichnung, Auftragsverarbeitungsverträge, der von Ihnen übernommene Aufbewahrungsplan und ob eine Funktion für besondere Kategorien eingeschaltet wird.

Siehe auch die [Personalleitung](../personalleitung/index.md) und die [Risikoleitung](../risikoleitung/index.md).

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
