# Der Weg in den Produktivbetrieb

WPM ist Demonstrationssoftware. Diese Seite sagt, was der Produktivbetrieb für WPM bedeuten würde, und führt auf, was zwischen WPM und diesem Betrieb steht. Jeder Punkt trägt eine Schätzung, wie weit er bereit ist, von 0 bis 100 Prozent.

## Was Produktivbetrieb bedeutet

Produktivbetrieb bedeutet, dass sich eine Organisation für echte Arbeit auf WPM verlässt:

- die Datensätze sind die echter Menschen: Beschäftigte, Bewerberinnen und Bewerber sowie deren Notfallkontakte;
- von WPM berechnete Vergütung wird ausgezahlt, und für sie gilt das Recht des Arbeitsorts;
- Menschen melden sich täglich an, und einiges von dem, was sie lesen, ist sensibel, etwa krankheitsbedingte Abwesenheit;
- die Organisation ist einer Aufsichtsbehörde, einer Prüfstelle und den Menschen in den Datensätzen gegenüber dafür verantwortlich, wie sie geführt wird.

Diese Seite misst WPM am schwierigsten Fall, für den es bewertet wurde: einer großen medizinischen oder staatlichen Organisation ([die Reifegradbewertung](../../../spec/readiness-assessment.md), Englisch). Eine kleinere Organisation braucht von manchen Punkten vielleicht weniger, aber sie braucht dieselben Arten von Nachweisen.

Der Produktivbetrieb braucht drei Arten von Nachweisen, und Software allein liefert nur die erste:

1. **Software**, die die Aufgabe sicher erfüllt, mit Tests, die das zeigen.
2. **Ein Betreiber**, der sie hostet und betreibt: Sicherungen, Überwachung, Support und Reaktion auf Vorfälle.
3. **Andere Personen als der Autor**, die sie geprüft haben: ein Penetrationstester, eine Prüfstelle für Barrierefreiheit, Übersetzerinnen und Übersetzer sowie die oder der Datenschutzbeauftragte des Betreibers.

## Wie die Schätzungen zu lesen sind

- Eine Schätzung ist das Urteil des Autors am 2026-11-13, gebildet aus der Spezifikation und den Tests. Sie ist keine Messung und keine Zertifizierung.
- 0 Prozent bedeutet, dass noch nichts existiert. 100 Prozent bedeutet gebaut, getestet und von einer unabhängigen Stelle in einem echten Einsatz geprüft. Kein Punkt steht bei 100 Prozent.
- Jeder Punkt sagt, worauf seine Schätzung beruht und was offen bleibt.
- Die Schätzungen werden nicht gemittelt. Ein einziger Punkt mit niedriger Schätzung genügt, um WPM aus dem Produktivbetrieb herauszuhalten.

## Die Software

- **75%** **Anmeldung und Zugriff nur bei Kenntnisbedarf.** Die Anmeldung ist standardmäßig eingeschaltet, und der Dienst verweigert im Produktivbetrieb ohne sie den Start (WPM-R72). Lesezugriffe sind auf Personen mit Kenntnisbedarf beschränkt (WPM-T188), und ein Token hat ein Höchstalter (WPM-T200). Beruht auf: den Durchsetzungstests, die in CI laufen. Offen: Es gibt keine Sperrliste für Tokens, und es wurde kein unabhängiger Penetrationstest durchgeführt.
- **70%** **Ein vollständiges und unverändertes Prüfprotokoll.** Das Prüfprotokoll lässt nur Anfügen zu, ist per Hash verkettet und kann in ein Überwachungssystem exportiert werden (WPM-T189, WPM-T197). Beruht auf: Tests, die absichtlich gebrochen wurden, um zu beweisen, dass sie fehlschlagen. Offen: Ein abgeschnittenes Ende der Kette wird nicht erkannt, und eine Eigentümerin oder ein Eigentümer der Datenbank kann die Trigger weiterhin abschalten.
- **70%** **Datenschutz durch Technikgestaltung.** Personen werden über Referenzen identifiziert, es gibt keine demografischen Spalten, und es gibt Auskunft, Löschung und Aufbewahrung für jede Art von Datensatz. Beruht auf: den Tests und Abschnitt 1 der Reifegradbewertung. Offen: die eigene Bewertung und Rechtsgrundlage des Betreibers (siehe unten).
- **75%** **Verschlüsselte Verbindungen und Sicherungen.** Im Produktivbetrieb verweigert der Dienst eine unverschlüsselte Datenbankverbindung und bricht eine hängende Anfrage nach einer Zeitüberschreitung ab (WPM-T191), und Sicherungen sind verschlüsselt (WPM-T192). Beruht auf: den Tests beider. Offen: Die Verwaltung von Schlüsseln und Zertifikaten liegt beim Betreiber.
- **10%** **Gesetzliche Berechnungen.** Die gesetzlichen Berechnungen dienen der Veranschaulichung, und die Lohnsteuer ist ein Platzhalter ([die regulatorische Einordnung](../../../spec/regulatory.md), Englisch). Beruht auf: der Spezifikation, die das sagt. Offen: die Regeln jedes Arbeitsorts, geschrieben und geprüft von einer fachkundigen Person. Zahlen Sie niemandem etwas auf Grundlage von WPM aus.
- **50%** **Skalierung.** Auf einem Laptop versagen die Verzeichnisseiten und die Kapazitätsansicht bei 50.000 Beschäftigten nicht, und die Planungsansichten lehnen mehr als 10.000 ab (WPM-T196, WPM-T230). Beruht auf: einem gemessenen Lauf ([BENCHMARKS.md](../../../BENCHMARKS.md), Englisch). Offen: Die Kapazitätsansicht wächst weiterhin proportional zur Belegschaft, und auf der Hardware eines Betreibers wurde nichts gemessen.
- **65%** **Lieferkette.** Jede CI-Action ist auf einen Commit fixiert, und die Historie wird nach Geheimnissen durchsucht. Das Container-Image ist signiert und trägt seine Herkunftsangaben, und dies wurde für das Release v0.1.2 ausgeführt (WPM-T194, WPM-T195). Beruht auf: diesem Release-Lauf. Offen: Das Image-Paket ist privat, sodass es außerhalb des Projekts noch niemand überprüfen kann.
- **60%** **Überwachung.** Jede Antwort trägt eine Anfrage-ID, und Protokollzeilen enthalten weder Abfragezeichenfolge noch personenbezogene Kennung (WPM-T197). Beruht auf: den Tests des Anfrageprotokolls. Offen: Dashboards und Alarme liegen beim Betreiber, und Ratenbegrenzungen gelten nur für eine einzelne Instanz (WPM-T198).
- **60%** **Sicherung und Wiederherstellung.** Verschlüsselte Sicherungen, eine Wiederherstellungsübung, die in CI läuft, und ein Betriebshandbuch. Beruht auf: der Übung. Offen: eine Wiederherstellungsübung auf der eigenen Plattform des Betreibers innerhalb der letzten 90 Kalendertage (Schwelle WPM-G4).
- **40%** **Prüfungen der Barrierefreiheit.** Automatisierte Prüfungen gegen WCAG 2.2 AA laufen auf der Anwendung und auf dieser Website, im hellen und im dunklen Design. Beruht auf: den Tests mit Playwright und axe. Offen: die unabhängige Prüfung (siehe unten); es wurde kein Screenreader verwendet.
- **20%** **Übersetzungen.** Die Anwendung hat 17 Locales und die Rollenseiten haben 21. Beruht auf: Der Text existiert. Offen: Eine KI hat die Übersetzungen verfasst, und niemand hat sie professionell geprüft (WPM-T155).

## Prüfungen durch andere Personen als den Autor

- **0%** **Unabhängiger Penetrationstest.** Es wurde kein Penetrationstest einer bereitgestellten Instanz durchgeführt (WPM-T201). Beruht auf: Die Aufgabe ist offen. Offen: der Test und die Behebung dessen, was er findet.
- **0%** **Unabhängige Prüfung der Barrierefreiheit.** Es wurde keine Prüfung gegen WCAG 2.2 AA mit Menschen durchgeführt, die assistive Technologien nutzen (WPM-T202). Beruht auf: Die Aufgabe ist offen. Offen: die Prüfung und die Behebung dessen, was sie findet.
- **25%** **Bedrohungsmodell.** Ein erstes Bedrohungsmodell existiert ([threat-model.md](../../../spec/governance/threat-model.md), Englisch), verfasst vom Autor. Beruht auf: dem Dokument. Offen: Ein unabhängiges Team erstellt es für einen echten Einsatz neu.

## Der Anteil des Betreibers

- **20%** **Datenschutz-Folgenabschätzung.** Ein anzupassendes Paket existiert: eine Folgenabschätzung, ein Datenfluss, ein Verzeichnis von Verarbeitungstätigkeiten und ein Aufbewahrungsplan ([spec/governance](../../../spec/governance/dpia.md), Englisch). Beruht auf: dem Paket. Offen: Es gibt keinen Betreiber, daher hat keine Datenschutzbeauftragte und kein Datenschutzbeauftragter eine Bewertung, eine Rechtsgrundlage, einen Aufbewahrungsplan oder einen Auftragsverarbeitungsvertrag freigegeben (Schwelle WPM-G5).
- **15%** **Gleichstellungsfolgenabschätzungen.** Eine Vorlage und ein Register existieren ([equality-impact](../../../spec/equality-impact/template.md), Englisch). Beruht auf: der Vorlage. Offen: Der Betreiber verfasst und genehmigt eine Abschätzung für den eigenen Einsatz.
- **40%** **Hosting und Betrieb.** Eine Produktivkonfiguration, ein Container-Image sowie Dokumente für Bereitstellung und Upgrades existieren. Beruht auf: diesen Dokumenten und der Wiederherstellungsübung. Offen: Auf einem gehärteten Host wurde nichts ausgeführt, und ein Betreiber muss den Dienst betreiben.
- **10%** **Support.** Es gibt einen Issue-Tracker und eine Sicherheitsrichtlinie ([SECURITY.md](../../../SECURITY.md), Englisch). Beruht auf: diesen beiden. Offen: Es gibt keinen Supportvertrag, kein Service-Level, keine Rufbereitschaft und keine Organisation hinter WPM.

## Was diese Seite nicht behauptet

Die Seite hält keine Entscheidung fest, den Betrieb aufzunehmen. Diese Entscheidung liegt bei einem Betreiber und wird außerhalb von WPM getroffen. Die Schätzungen decken nur die aufgeführten Punkte ab, und die eigene Liste der Reifegradbewertung dessen, was sie nicht geprüft hat, gilt auch hier. Wo ein klinisches System angebunden ist, entscheidet der Betreiber über eine Prüfung der klinischen Sicherheit.
