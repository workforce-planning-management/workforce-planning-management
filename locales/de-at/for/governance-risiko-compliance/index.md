# Governance, Risiko, Compliance

**Die Fragen, die Sie wahrscheinlich stellen:** Wer ist wofür verantwortlich, was könnte schiefgehen und was hält es auf, und welche Nachweise kann ich einer Prüferin oder einem Prüfer zeigen? Diese Seite bündelt das Material zu Governance, Risiko und Compliance an einem Ort. Die Seiten der [Risikoleitung](../risikoleitung/index.md) und der [Rechtsleitung](../rechtsleitung/index.md) decken dasselbe Feld nach Rolle ab.

## Governance: wer entscheidet und wie es festgehalten wird

| Thema | Was es gibt | Lesen |
| --- | --- | --- |
| Wie die Arbeit gesteuert wird | Spezifikation zuerst: Anforderung, Entscheidung, Aufgabe, Code und Tests landen zusammen; jede Aufgabe sagt, was **nicht** geprüft wurde | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Projekt-Governance | Governance, Maintainer, ein RFC-Verfahren und eine Datei für Code-Verantwortliche | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Entscheidungen über Menschen | Ein Mensch entscheidet; Bewertungen beraten. Umwandlungen, Kündigungen und Anträge auf flexibles Arbeiten werden als Entscheidung eines Menschen erfasst | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Zugriff und Rollen | Rollen und Attribute entscheiden über den Zugriff; eine klassifizierte Tabelle entscheidet die Kenntnis nur bei Bedarf auf jeder Leseroute | [auth.md](../../../../spec/auth.md) |
| KI-gestützte Autorschaft | Eine Erklärung dazu, wie KI eingesetzt wurde | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Risiko: was schiefgehen könnte

| Thema | Was es gibt | Lesen |
| --- | --- | --- |
| Ergebnis | **Nicht bereit** für einen Einsatz mit hohem Sicherheitsanspruch; ein Hindernis wurde durch Prüfung gefunden (Lesezugriffe folgten nicht der Kenntnis nur bei Bedarf) und behoben | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Bedrohungen | Ein Bedrohungsmodell: Werte, Akteure, Grenzen, die Kontrolle wie gebaut und das, was **nicht bewertet** wurde | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Manipulationsnachweis | Ein nur anfügbares, hashverkettetes Prüfprotokoll mit Prüfendpunkt; Einträge enthalten weder Begründungstext noch Wert | [audit.md](../../../../spec/audit.md) |
| Sicherer Start | Die Produktion startet nicht bei ausgeschalteter Anmeldung, ohne Token-Schlüssel, bei unverschlüsselter Datenbankverbindung zu einem anderen Host oder bei unverschlüsseltem entferntem Redis | [production-readiness.md](../../../../spec/production-readiness.md) |
| Wiederherstellung | Verschlüsselte Sicherungen, eine in CI laufende Wiederherstellungsübung, ein Löschverzeichnis, damit eine Wiederherstellung keine gelöschte Person zurückbringt | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Vorfälle | Ein Betriebshandbuch mit ersten Maßnahmen; Kontakte und Eskalationswege ergänzt der Betreiber | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Verzerrung bei Bewertungen | Eine Vorlage für die Gleichstellungs-Folgenabschätzung und ein Verzeichnis jeder Bewertung, jeweils mit einem Entwurf der Abschätzung | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Offene Risiken | Fixieren und Signieren der Lieferkette, Lastnachweise, Protokollprüfungen auf personenbezogene Daten, geteilte Ratenbegrenzungen, ein Satz von Datenbankrollen mit geringsten Rechten, Hinweise zum Widerruf von Token | [readiness-assessment.md](../../../../spec/readiness-assessment.md), Abschnitt 3 |

## Compliance: was Sie einer Prüferin oder einem Prüfer zeigen können

| Thema | Was es gibt | Lesen |
| --- | --- | --- |
| Selbstbewertung der Kontrollen | Eine Checkliste, bei der jede Antwort erfüllt, teilweise erfüllt, nicht erfüllt oder vom Betreiber zu beantworten ist | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Datenschutz-Folgenabschätzung | Eine Vorlage, für die Demo ausgefüllt, mit markierten Teilen, die Sie selbst ausfüllen müssen | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Verarbeitungsverzeichnis | Ein anpassbares Verzeichnis | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Datenfluss | Wohin sich personenbezogene Daten bewegen | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Aufbewahrung | Ein Plan je Datensatzart | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Betroffenenrechte | Auskunftsexport und Löschung | [regulatory.md](../../../../spec/regulatory.md) |
| Daten besonderer Kategorien | Gleichstellungsmonitoring und Gesundheitsanforderungen am Arbeitsplatz sind aus, bis eine Rechtsgrundlage erfasst ist, und dann begrenzt | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Barrierefreiheit und Sprache | Eine Erklärung zur Barrierefreiheit und eine Erklärung zur Amtssprache | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Lizenzen und Abhängigkeiten | Lizenzdateien, eine mit `cargo deny` geprüfte Abhängigkeitsrichtlinie und eine Software-Stückliste bei der Veröffentlichung | [production-readiness.md](../../../../spec/production-readiness.md) |

## Was nur Menschen tun können

Dies sind Schranken, kein Code, und das Repository markiert sie nicht als erledigt: ein unabhängiger Penetrationstest, ein unabhängiges Barrierefreiheits-Audit, Ihre von Ihrer Beauftragten oder Ihrem Beauftragten unterzeichnete Datenschutz-Folgenabschätzung, eine Wiederherstellungsübung auf Ihrer Plattform, eine professionelle Prüfung der Übersetzungen und Ihre Genehmigung der Gleichstellungs-Folgenabschätzungen.

## Grenzen dieser Seite

Die Tabellen verweisen auf Dokumente. Ob jedes Dokument für Ihren Einsatz zutrifft, beurteilen Sie und Ihre Prüferin oder Ihr Prüfer. Die Tests der Autorin oder des Autors sind keine unabhängige Prüfung, und nichts hier ist Rechtsberatung oder Zertifizierung.

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
