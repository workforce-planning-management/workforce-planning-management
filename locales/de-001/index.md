# Personalplanung

**Eine quelloffene HR-Plattform für den gesamten Lebenszyklus von Beschäftigten, von der Einstellung bis zum Ruhestand.**

Workforce Planning Management (WPM) vereint Personalgewinnung, Zeit und Abwesenheit, Mitarbeiterentwicklung, Personalplanung und Entgeltabrechnung in einer Anwendung. Es ist Demonstrationssoftware, gedacht zur Evaluierung und als Referenzentwurf.

Sprache: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Demonstrationssoftware, noch nicht für den Produktivbetrieb bereit.** WPM verwendet ausschließlich synthetische Daten, und seine gesetzlichen Berechnungen sind beispielhaft. Geben Sie keine echten personenbezogenen Daten ein. [Der Weg in den Produktivbetrieb](path-to-production/index.md) sagt, was ein Produktivbetrieb bräuchte und wie weit jeder Teil ist. Siehe auch [die rechtliche Einordnung](../../spec/regulatory.md).

[Demo starten](start/index.md) [Rollenbeschreibungen](for/index.md) [Quellcode auf GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Vorteile

HR-Daten liegen oft in getrennten Werkzeugen, etwa in Bewerbungspostfächern, Tabellen für Urlaub und Dienstplänen. Jede Übergabe zwischen ihnen bedeutet Handarbeit und das Risiko von Fehlern. WPM soll diese Übergaben verringern.

- **Ein einziges System.** Onboarding, Urlaub, Schulungen und Entgeltdaten liegen in einer Anwendung, was das erneute Eintippen verringert.
- **Datenminimierung.** Daten, die nicht gespeichert werden dürfen, haben kein Feld, und Daten, die nicht offengelegt werden dürfen, haben keine API-Route. Es gibt zum Beispiel kein Diagnosefeld, und Antworten auf Kurzumfragen haben keine Verfasserin und keinen Verfasser.
- **Regeln für Berichte.** Unbekannte Werte werden nicht als Null ausgewiesen, Aggregate identifizieren keine Einzelpersonen, und Schätzungen nennen ihre Grundlage.
- **Lokalisierung.** Die Anwendung gibt es in 17 Locales und die Rollenbeschreibungen in 21, darunter Sprachen mit Schreibrichtung von rechts nach links. Länderspezifische Regeln liegen als Daten vor. Die Übersetzungen stammen von einer KI und wurden nicht von Muttersprachlerinnen und Muttersprachlern geprüft.
- **Nachvollziehbarkeit.** Der Quellcode ist offen, und jedes Verhalten lässt sich auf eine nummerierte Anforderung, Entwurfsentscheidung und Aufgabe zurückführen.
- **Unabhängiger Betrieb.** Der Stack besteht aus einer JSON-API in Rust, einem SvelteKit-Client und PostgreSQL. Er lässt sich aus einem frischen Klon bauen, und Integrationen mit Schwesteranwendungen sind optional.

## Funktionen

- **Personalgewinnung und Akten.** Stellenanforderungen, Bewerbungsverfahren, Onboarding, Personalakten und Organigramm.
- **Zeit und Einsatzplanung.** Zeit und Abwesenheit, Schichten mit Arbeitszeitprüfungen, der Bereitschaftsplan mit Tausch und Erinnerungen sowie ein Mitarbeiterverzeichnis.
- **Entwicklung.** Beurteilungen, 360°-Feedback, Lernen, Kompetenzlückenanalyse, Empfehlungen für Schulungszeit und verpflichtende Schulungen.
- **Planung und Berichte.** Strategische Personalplanung, Kennzahlen und Erkenntnisse sowie ein CEO-Dashboard.
- **Eintritte und Austritte.** Datierte Checklisten und eine Übergabe am letzten Tag mit Prüfpfad.
- **Kommunikation.** Ankündigungen mit Zielgruppen, Links und Lesezahlen.
- **Rechte und Wohlbefinden der Beschäftigten.** Impulse zum Wohlbefinden, eine anonyme Kurzumfrage, Anpassungen, Auskunft, Löschung und Aufbewahrung.
- **Entgelt.** Entgeltabrechnung und Vergütung mit beispielhaften gesetzlichen Berechnungen.

## Vergleich

Die Tabelle vergleicht WPM mit drei Kategorien von HR-Systemen. Sie ist eine allgemeine Orientierung, kein Benchmark. Siehe auch den [Vergleich nach Produkt](../../docs/compare/index.md).

| | WPM | Komplettsuiten (Workday, SAP SuccessFactors, BambooHR) | Quelloffene HR-Software (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Einzelwerkzeuge (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Umfang** | Voller Lebenszyklus, von der Einstellung bis zum Ruhestand | Ähnlicher Umfang auf einer Plattform | Dieselben Kernfunktionen | Eine Funktion |
| **Lizenz** | Quelloffen | Kommerziell | Quelloffen | Kommerziell |
| **Hauptstärke** | Datenminimierung, nachvollziehbarer Entwurf, Analyse von Kompetenzlücken und Schulungszeit | Eine integrierte, betreute Plattform | Mehr Integrationen, Lokalisierungen und Installationen | Tiefe in einer Funktion |
| **Typischer Einsatz** | Evaluierung, Referenzentwurf oder Weiterentwicklung | Organisationen, die ein betreutes System brauchen | Organisationen, die quelloffene Software im Produktivbetrieb wollen | Organisationen, die Tiefe in einer Funktion brauchen und den Rest integrieren können |

## Zielgruppe

- **Führungskräfte.** Die [Rollenbeschreibungen](for/index.md) legen dar, was WPM für jede Führungsrolle bietet und wie seine Ausgaben zu lesen sind.
- **HR-Teams.** Das [Demo-Tutorial](start/index.md) behandelt jede Seite anhand fiktiver Daten.
- **Entwicklerinnen, Entwickler und Architekten.** Siehe die [Spezifikation](../../spec/index.md) und die [Installationsanleitung](../../INSTALL.md).
- **Prüfende.** Die [Reifegradbewertung](../../spec/readiness-assessment.md) listet die Lücken zwischen WPM und dem Produktivbetrieb auf.

## Hilfe erhalten

- **Einen Fehler melden oder eine Frage stellen** im [Issue-Tracker](https://github.com/workforce-planning-management/workforce-planning-management/issues). Nennen Sie die Seite, was Sie getan haben, was Sie erwartet haben und was geschah. Fügen Sie keine echten personenbezogenen Daten bei.
- **Ein Sicherheitsproblem melden** per E-Mail, wie in der [Sicherheitsrichtlinie](../../SECURITY.md) beschrieben, nicht in einem öffentlichen Issue.
- **Eine Aufgabe in der Anwendung erlernen.** Der Link **Hilfe** in der Kopfzeile der Anwendung öffnet die Hilfeseite und die Anleitungen für Beschäftigte, Führungskräfte und HR. Jede Fehlermeldung verweist ebenfalls darauf.

## Nächste Schritte

1. **Lesen Sie die Beschreibung für Ihre Rolle.** Jede Beschreibung sagt, was WPM bietet und was offen bleibt.
2. **Starten Sie die Demo.** Die Tour läuft auf einem lokalen Rechner mit fiktiven Daten.
3. **Geben Sie Rückmeldung.** Siehe [Kommentieren](../../RFC.md) und [Mitwirken](../../CONTRIBUTING.md).

[Demo starten](start/index.md) [Dokumentation](../../index.md)

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-001/`).
