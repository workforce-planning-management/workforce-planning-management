# Unabhängige Sicherheitsprüfung

**Die Frage, die Sie wahrscheinlich stellen:** Wie bringen wir Menschen, die diese Software nicht geschrieben haben, dazu, sie anzugreifen, worauf sollen wir sie ansetzen, und was tun wir mit dem, was sie finden?

Es wurde keine unabhängige Prüfung durchgeführt. Das [Bedrohungsmodell](../../../../spec/governance/threat-model.md) schrieb derselbe Assistent, der die Software schrieb, und niemand sonst hat es geprüft; jede darin genannte Kontrolle wurde von diesem Autor erprobt. Die [Reifegradbewertung](../../../../spec/readiness-assessment.md) führt einen unabhängigen Penetrationstest als Schranke auf, die nur Menschen schließen können (WPM-T201, **nicht erledigt**). Diese Seite ist ein Briefing für diejenigen, die einen beauftragen. Sie ist kein Prüfplan einer Prüferin oder eines Prüfers und ersetzt keinen.

## Was den Prüfern zu geben ist

| Posten | Wo | Hinweise |
| --- | --- | --- |
| Eine Bereitstellung zum Angreifen | Ihre eigene: [Bereitstellungsleitfaden](../../../../spec/operations/deployment.md) | Produktionseinstellungen mit **eingeschalteter** Anmeldung, ein Test-Identitätsanbieter und **nur synthetische Daten**. Nicht gegen Datensätze echter Personen testen |
| Der Quelltext | Dieses Repository | Ein Test mit Quelltext ist gründlicher als ein Black-Box-Test; sagen Sie, welchen Sie möchten |
| Das Bedrohungsmodell | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Vierundzwanzig Bedrohungen, die Kontrolle wie gebaut und die Lücken. Geben Sie es ihnen **nach** einem ersten Black-Box-Durchgang, wenn Sie sehen wollen, was sie ohne Anstoß finden |
| Die API-Beschreibung | `GET /api-docs/openapi.json` | Jede Route, mit dem `Idempotency-Key` bei jeder Operation, die etwas ändert |
| Der Anmeldeentwurf | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Personas, Attribute und dass die Richtlinie nur nach Aktion entscheidet |
| Testidentitäten | Sie legen sie an | Mindestens: eine Person ohne Attribute, eine Führungskraft, eine Führungskraft außerhalb der Linie, Personalabteilung, Lohnbuchhaltung, eine Administration und ein Dienst-Peer |

## Worauf sie besonders achten sollen

Der Autor konnte dies nicht prüfen. Jede Zeile nennt den Eintrag des Bedrohungsmodells, damit ein Befund zurückverfolgt werden kann.

| Bereich | Was zu versuchen ist | Bedrohung |
| --- | --- | --- |
| Token | Algorithmenverwechslung; ein Schlüsselsatz, der sich während laufender Anfragen dreht; ein abgelaufenes oder noch nicht gültiges Token; ein Token für eine andere Zielgruppe | T1, T2 |
| Kenntnis nur bei Bedarf | Jede Leseroute als Aufrufer ohne Attribute und als Nachbar; eine Route, die die Klassifizierungstabelle nicht kennt; ein Schreibzugriff, den die Richtlinie erlaubt, den der Handler aber verweigern sollte | T3, T4, T6 |
| Privilegierte Attribute | Ob ein Attribut in einem Token hinzugefügt, wiederverwendet oder vererbt werden kann; wer es ausstellen kann; ein geteilter oder langlebiger Administrator | T24 |
| Injektion | Jeden Parameter, jedes Pfadsegment und jedes Body-Feld fuzzen; die Lösch- und Aufbewahrungsanweisungen werden aus typisierten Kennungen gebaut und wurden gelesen, nicht gefuzzt | T12 |
| Der Browser-Client | Sitzungsbehandlung durch den Oberflächenserver, wie er eine Sitzung gegen ein Token tauscht, Cross-Site-Scripting über gespeicherten Text, Umgehung der Content-Security-Policy und die Seite des Inhaltsmanagers, die ein Skript ohne festgelegte Version aus einem öffentlichen Netz lädt | T13, T14 |
| Ausgehende Anfragen | Server-Side-Request-Forgery über jede konfigurierte URL (Schlüsselsätze, die Schwesterdienste, der gemeinsame Cache) | T15 |
| Grenzen | Die Body-Grenze, das Anfrage-Timeout, die Ratenbegrenzungen je Instanz und die Ansichten, die die Datensätze einer ganzen Organisation in den Speicher laden | T16, T17 |
| Protokolle | Ob personenbezogene Daten, ein Token oder ein Query-String in ein Protokoll gelangen | T19 |
| Idempotenzschlüssel | Ein Schlüssel von zwei Aufrufern; derselbe Schlüssel mit geändertem Body, Pfad oder Query; ein fast gültiger Schlüssel; ein sehr großer Body; zwei Anfragen im Wettlauf unter einem Schlüssel (die Inbesitznahme ist begründet, nicht in einem Wettlauf getestet); ob außer ihrem Aufrufer jemand eine gespeicherte Antwort lesen kann | neu |
| Das Prüfprotokoll | Jede Möglichkeit, einen Eintrag zu ändern, zu entfernen oder zu überspringen; der Endpunkt zur Verkettungsprüfung | T7 |
| Sicherungen | Wiederherstellung aus einer manipulierten oder mit falschem Schlüssel versehenen Sicherung; ob eine gelöschte Person zurückkehrt | T10, T11 |
| Lieferkette | Festlegen der Build-Actions, Secret-Scanning, Signieren von Images | T20, T21 |

## Regeln, die vor dem Start zu klären sind

Füllen Sie diese mit Ihren eigenen Sicherheits- und Rechtsteams aus. Keine davon wird hier entschieden.

- **Befugnis:** Wer unterzeichnet, dass die Prüfung autorisiert ist, und für welche Systeme und Adressen.
- **Zeitfenster:** Die Tage und Stunden, in Kalendertagen gezählt, und wen man anruft, um sie zu stoppen.
- **Umfang:** Was dazugehört (die API, die Oberfläche, der Anmeldeablauf, das Sicherungsskript) und was nicht (der Dienst des Identitätsanbieters selbst, Ihr Netzwerk, alles, was mit anderen Systemen geteilt wird).
- **Daten:** Nur synthetische. Wenn eine Prüferin oder ein Prüfer echte personenbezogene Daten findet: Wer wird informiert und wie wird damit umgegangen.
- **Zerstörerische Tests:** Ob Dienstverweigerung, Massenschreibvorgänge und Löschung erlaubt sind, und wo.
- **Berichterstattung:** Das Format, die Schweregradskala, wer ihn erhält, wie lange er aufbewahrt werden darf und wie gefundene Geheimnisse behandelt werden.
- **Nachprüfung:** Dass der Vertrag die Prüfung der Korrekturen einschließt.

## Was mit den Befunden zu tun ist

1. Erfassen Sie jeden als Aufgabe in [tasks.md](../../../../spec/tasks.md) mit der Referenz der Prüfer, dem Nachweis und dem von ihnen vergebenen Schweregrad. Ein bestrittener Befund bleibt als bestritten erfasst, mit der Begründung.
2. Beheben Sie mit einem Test, der zuerst fehlschlägt. Ist die Korrektur eine Regel, tragen Sie sie in die Lückenspalte des Bedrohungsmodells ein und schließen Sie sie dort erst, wenn die Nachprüfung besteht.
3. Markieren Sie WPM-T201 erst als erledigt, wenn der Bericht vorliegt, jeder Befund einen Verantwortlichen hat und eine Nachprüfung abgeschlossen ist. Aktualisieren Sie die [Reifegradbewertung](../../../../spec/readiness-assessment.md) und die Schrankenliste (WPM-T203) in derselben Änderung.
4. Halten Sie den Bericht aus dem Repository heraus, wenn er etwas Ausnutzbares enthält; verlinken Sie stattdessen dorthin, wo er aufbewahrt wird.

## Was ein sauberer Bericht bedeutet und was nicht

Eine Prüfung ohne Befunde zeigt, dass diese Prüfer in diesem Zeitfenster mit diesem Umfang und diesen Identitäten nichts gefunden haben. Sie zeigt nicht die Abwesenheit von Fehlern, sie deckt keine Konfiguration ab, die Sie danach geändert haben, und sie ersetzt nicht die anderen Schranken: ein Barrierefreiheits-Audit, Ihre von Ihrer Beauftragten oder Ihrem Beauftragten unterzeichnete Datenschutz-Folgenabschätzung und eine Wiederherstellungsübung auf Ihrer Plattform.

## Was diese Seite nicht leistet

Sie nennt keinen Anbieter, setzt keinen Preis fest und verspricht keinen Standard. Welches Schema oder welche Akkreditierung Ihre Branche von Prüfern erwartet, entscheiden Sie.

## Wie aktuell dies ist

Geschrieben am 2026-10-11 anhand der Spezifikation, nicht durch einen Lauf der Software für diese Seite. Zahlen, die sich ändern (Tests, Migrationen, erledigte Aufgaben), sind hier nicht kopiert; sie stehen in [implementation-status.md](../../../../spec/implementation-status.md), das ein Skript erzeugt. WPM ist Demonstrationssoftware mit synthetischen Daten: Diese Seite sagt, was sie tut und was nicht, und behauptet nicht, dass sie für Ihre Organisation geeignet ist. Nichts hier ist Rechtsberatung.

> Diese Übersetzung wurde von einer KI verfasst und von keiner Muttersprachlerin und keinem Muttersprachler geprüft. Bei Abweichungen gilt die englische Fassung (`locales/en-gb-oxendict/for/`).
