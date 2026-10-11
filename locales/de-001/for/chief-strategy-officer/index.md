# Für die Strategieleitung

**Die Frage, die Sie wahrscheinlich stellen:** Ist unsere Belegschaft so aufgestellt, dass sie der Strategie dient, was werden wir brauchen, und was sollten wir gegen die Lücke tun?

## Was Sie bekommen

| Sie wollen wissen | Wo WPM antwortet | Lesen |
| --- | --- | --- |
| Was eine Rolle verlangt | Rollenprofile mit erforderlichen Fähigkeiten und einer Mindeststufe, damit eine Lücke an einer Rolle statt an einer Person gemessen wird | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Was wir brauchen werden | Personalpläne als Entwurfswelten, mehrere nebeneinander, mit genannten Annahmen und Bedarf nach Abteilung, Rolle und Zeitraum | WPM-R36, dieselbe Datei |
| Wo die Lücke liegt | Prognostiziertes Angebot gegenüber geplantem Bedarf: eine Lücke bei Köpfen und bei Kompetenzen, Stellhebel als Vorschläge mit ihrer Evidenz aufgeführt | WPM-R37 |
| Ob es der Strategie dient | Mit dem Bedarf verknüpfte Ziele: Ziele ohne Ressourcen, Bedarf ohne Zuordnung, kritische Rollen ohne Nachfolgebank | WPM-R38 |
| Ob wir ein Programm liefern können | Fähigkeitspools, Programmbedarf, Zusagen von Partnern, der Engpasspool, eine Startprüfung, die ein Vorschlag ist | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Wie sehr wir uns auf nicht unbefristete Kräfte stützen | Der Anteil der FTE, der freiberuflich ist, und der, der befristet ist, jeweils getrennt, und wie viel Finanzierung in jedem der nächsten zwölf Monate endet | `GET /api/workforce-intelligence/contingent` (WPM-T109, teilweise) |
| Was wir tun wollen, wenn Verträge enden | Umwandlungspläne: die Entscheidung eines Menschen, umzuwandeln, zu verlängern oder zu beenden, mit der Finanzierung neben dem Vertragsende | [engagements.md](../../../../spec/engagements.md) |
| Was sonst geplant ist | Die Roadmap und die Liste offener Aufgaben | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Wie eine Prognose zu lesen ist

- Sie ist eine genannte Regel, angewandt auf genannte Eingaben, kein Modell, und jedes Ergebnis nennt seine Annahmen.
- Bei zu wenig Personalbestandsverlauf sagt sie `insufficient_history`, statt zu extrapolieren.
- Ein Plan hält nur hypothetische Köpfe. Er liest oder ändert nie den Datensatz einer Person.
- Stellhebel sind Vorschläge. Die Planerin oder der Planer wählt, und die Software entscheidet nichts.

## Grenzen

- Der Stellhebel „Flexible Kräfte unbefristet machen“ in Plänen ist spezifiziert (WPM-R63) und noch nicht im Code.
- Der flexible Anteil hat noch keinen Trend: Er braucht Momentaufnahmen nach Grundlage (WPM-T139, nicht gebaut).
- Planung nach kritischer Kette und kritischem Pfad gehört zu project-portfolio-management, nicht hierher.
- Für Finanzierung und Umwandlungspläne gibt es noch keine Benutzeroberfläche.

Siehe auch die Seiten der [Geschäftsführung](../chief-executive-officer/index.md) und der [Betriebsleitung](../chief-operating-officer/index.md).

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
