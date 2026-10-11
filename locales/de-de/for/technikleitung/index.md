# Für die Technikleitung (CTO)

**Die Frage, die Sie wahrscheinlich stellen:** Ist es gut gebaut, kann mein Team es sicher ändern, und woher weiß ich das?

## Was Sie bekommen

| Anliegen | Was es gibt | Lesen |
| --- | --- | --- |
| Stack | Rust mit Loco (Axum, SeaORM) und PostgreSQL 18 für die API; SvelteKit 2 mit Svelte 5 und TypeScript für den Client | [architecture.md](../../../../spec/architecture.md) |
| Entwurfsregel | Ein reiner Kern: Regeln sind datenbank- und uhrenfrei, erschöpfend getestet und erst dann an Controller angeschlossen | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Spezifikation zuerst | Eine Änderung ist Anforderung, Entscheidung, Aufgabe, Code und Tests zusammen; jeder Aufgabeneintrag sagt, was **nicht** geprüft wurde | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Tests | Unit-Tests der Regeln, datenbankgestützte Anfragetests, Durchsetzungs-Binaries mit eingeschalteter Anmeldung, Frontend-Unit- und Browsertests. Mutationsprüfungen: Den Code absichtlich kaputtmachen und bestätigen, dass ein Test fehlschlägt | [testing.md](../../../../spec/testing.md) |
| Zahlen | Aus den Dateien erzeugt, nie von Hand kopiert | [implementation-status.md](../../../../spec/implementation-status.md) |
| Lieferkette | `cargo deny` für Lizenzen und Sperren, mitgelieferte gemeinsame Crates, eine Software-Stückliste bei der Veröffentlichung | [production-readiness.md](../../../../spec/production-readiness.md) |
| API-Vertrag | OpenAPI und ein Test, der fehlschlägt, wenn eine registrierte Route nicht dokumentiert ist | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Zugriffskontrolle | Die Richtlinie entscheidet nur nach Aktion; Handler und eine klassifizierte Routentabelle entscheiden die Kenntnis nur bei Bedarf, und ein Quellscan schlägt bei einer nicht klassifizierten Leseroute fehl | [auth.md](../../../../spec/auth.md) |
| Leistung | [BENCHMARKS.md](../../../../BENCHMARKS.md) hält fest, was gemessen wurde; es gibt keinen Lasttest | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Bekannte technische Schulden

- Mehrere Ansichten laden ganze Organisationen in den Speicher.
- Zeilensperren bei Umwandlungsplänen sind begründet, nicht durch einen Test bewiesen.
- CI, Veröffentlichung und der dynamische Scan sind geschrieben, aber noch nicht auf der Hosting-Plattform gelaufen.
- Actions sind per Tag fixiert, nicht per Digest, und Images sind nicht signiert.
- Einige Redis-Tests mit Passwort und TLS brauchen Variablen und eine Zertifizierungsstelle, sodass CI sie nicht ausführt.
- Der Baum `spec/` ist groß; eine erzeugte Statusdatei zeigt, was deklariert ist und was besteht.

## Wie man beiträgt

Lesen Sie zuerst [AGENTS.md](../../../../AGENTS.md): fünf Regeln, dann eine Karte nach Aufgabe.

Siehe auch die Seite der [Informationsleitung](../informationsleitung/index.md) für die Betriebssicht.

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
