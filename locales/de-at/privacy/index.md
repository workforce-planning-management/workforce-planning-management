# Datenschutzhinweis

Dieser Hinweis sagt, was die Website von Workforce Planning Management (WPM) und die WPM-Anwendung in Ihrem Browser speichern, warum, wie lange und wie Sie es entfernen. Die Website ist <https://workforce-planning-management.github.io/>; die Anwendung ist die WPM-Demonstrationssoftware, wo immer sie betrieben wird. (Aktualisiert 2026-11-17, WPM-R253.)

Keine von beiden hat Analysewerkzeuge, Werbung oder Skripte, Schriften oder Bilder von Dritten. Nichts, was eine von beiden in Ihrem Browser speichert, wird verwendet, um Sie von Website zu Website zu verfolgen.

## Die Website

Die Website ist eine Sammlung statischer Seiten. Sie hat keine Anmeldung und keine Formulare. Was sie speichert, liegt in Ihrem eigenen Browser:

| Was | Wo es gespeichert wird | Name | Was gespeichert wird | Warum |
| --- | --- | --- | --- | --- |
| Ihr Farbschema | Im lokalen Speicher Ihres Browsers | `mxi.wpm.theme` | Der Name des Farbschemas, zum Beispiel `dark` | Damit die nächste Seite und Ihr nächster Besuch so aussehen, wie Sie es gewählt haben |
| Ihr Farbschema | In einem Cookie nur für diese Website | `wpm-theme` | Der Name des Farbschemas, zum Beispiel `dark` | Damit das Farbschema zurückkommt, auch wenn Ihr Browser Cookies behält, aber den lokalen Speicher löscht, oder umgekehrt |
| Ihre Textgröße | Im lokalen Speicher Ihres Browsers | `mxi.wpm.text-size` | Der Name der Größe, zum Beispiel `large` | Damit die nächste Seite und Ihr nächster Besuch die gewählte Textgröße verwenden |
| Ihre Sprache | Nichts wird gespeichert | | | Die Sprache ist Teil der Adresse jeder Seite (zum Beispiel `/cy-001/`), daher gibt es nichts zu merken |
| Wo Sie auf einer Seite waren | Im Sitzungsspeicher Ihres Browsers, nur für diesen Tab | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Scrollpositionen und die Reihenfolge der Seiten, die Sie in diesem Tab geöffnet haben | Damit Zurück und Vorwärts Sie dorthin bringen, wo Sie waren. Das Framework, mit dem die Website gebaut ist (SvelteKit), speichert diese Einträge |

Keine Wahl wird gespeichert, bevor Sie sie treffen. Bis Sie wählen, folgt die Website der hellen oder dunklen Einstellung Ihres Browsers und seiner normalen Textgröße und speichert weder Farbschema noch Textgröße. Eine Wahl wird gespeichert, wenn Sie sie in der Auswahlleiste treffen, und eine neue Wahl ersetzt sie. Der Sitzungsspeicher wird geschrieben, während Sie zwischen Seiten wechseln.

Andere Teile der Website speichern nichts:

- **Die Suche** läuft in Ihrem Browser, auf einem Index, den die Website veröffentlicht, und speichert nichts. Ihre Suchwörter werden in die Adresse der Seite gesetzt (`?q=`), damit eine Ergebnisseite geteilt werden kann; wenn diese Adresse vom Webhost geladen wird (beim Neuladen oder über einen geteilten Link), erhält der Host sie mit der Anfrage, wie jede Adresse.
- **Die Schieberegler für den Reifegrad** auf dem Weg in den Produktivbetrieb speichern nichts, und ein Neuladen stellt die Schätzungen des Autors wieder her.
- **Teilen** öffnet die Website, die Sie auswählen (E-Mail, LinkedIn, Reddit, Bluesky oder Mastodon), nur wenn Sie sie auswählen. Dann gilt der Datenschutzhinweis dieser Website.

## Die WPM-Anwendung

Die Anwendung speichert dieselben Arten von Einstellungen und das Cookie, das Sie angemeldet hält:

| Was | Wo es gespeichert wird | Name | Was gespeichert wird | Warum |
| --- | --- | --- | --- | --- |
| Ihr Farbschema | Im lokalen Speicher Ihres Browsers | `mxi.wpm.theme` | Der Name des Farbschemas | Damit jede Seite so aussieht, wie Sie es gewählt haben |
| Ihre Textgröße | Im lokalen Speicher Ihres Browsers | `mxi.wpm.text-size` | Der Name der Größe | Damit jede Seite die gewählte Textgröße verwendet |
| Ihre Sprache | Im lokalen Speicher Ihres Browsers | `mxi.wpm.locale` | Der Sprachcode der zuletzt geöffneten Seite, zum Beispiel `cy-001` | Damit eine Adresse ohne Sprache in Ihrer Sprache öffnet |
| Ihre Sprache | In einem Cookie nur für die Anwendung | `wpm-locale` | Derselbe Sprachcode | Damit der Server eine Adresse ohne Sprache an Ihre Sprache weiterleiten kann |
| Ihre Anmeldung | In einem Cookie nur für die Anwendung, das die Skripte der Seite nicht lesen können | `__Host-mxi_session` (`mxi_session` auf dem eigenen Rechner einer Entwicklerin oder eines Entwicklers) | Eine Zufallszahl, die für Ihre angemeldete Sitzung steht | Damit Sie von Seite zu Seite angemeldet bleiben; ohne sie müssten Sie sich auf jeder Seite anmelden |
| Die Sprache des Inhaltseditors | Im lokalen Speicher Ihres Browsers, nur wenn Sie den Inhaltseditor öffnen (`/admin`) | `sveltia-cms.prefs` | Die Einstellungen des Editors, darunter seine Sprache | Damit der Editor in der Sprache der Anwendung öffnet |
| Wo Sie auf einer Seite waren | Im Sitzungsspeicher Ihres Browsers, nur für diesen Tab | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Scrollpositionen und die Reihenfolge der Seiten, die Sie in diesem Tab geöffnet haben | Damit Zurück und Vorwärts Sie dorthin bringen, wo Sie waren |

Anders als die Website speichert die Anwendung Farbschema und Textgröße, sobald sie sie anzeigt, also bevor Sie wählen, und sie speichert die Sprache jeder Seite, die Sie öffnen. Schlüssel aus der Zeit vor der Umbenennung des Projekts (`mxi.hcm.theme`, `mxi.hcm.locale`) werden auf die oben genannten Namen übertragen und gelöscht.

Das Anmelde-Cookie ist der einzige Eintrag, der für Sie steht. Die Zufallszahl darin bedeutet für sich allein nichts: Die angemeldete Sitzung, auf die sie verweist, wird vom Server der Anwendung gehalten, und das Abmelden beendet sie. Was die Anwendung über die Menschen in ihren Datensätzen speichert, behandelt der vorletzte Abschnitt weiter unten.

## Wer es erhält

Lokaler Speicher und Sitzungsspeicher verlassen Ihren Browser nie.

Ein Cookie wird mit jeder Anfrage gesendet, die Ihr Browser an die Website stellt, die es gesetzt hat, und an keine andere Website (`SameSite=Lax`, und `Secure` über HTTPS):

- Das Cookie `wpm-theme` der Website geht an GitHub Pages, das die Website hostet. Kein Programm des Projekts liest es.
- Das Cookie `wpm-locale` und das Anmelde-Cookie der Anwendung gehen an den Server, auf dem die Anwendung läuft. Er liest sie, um Ihre Sprache zu wählen und zu wissen, dass Sie angemeldet sind.

GitHub, das die Website hostet, erhält die Anfragen Ihres Browsers, wie jeder Webhost. [Die Datenschutzerklärung von GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) sagt, was GitHub aufzeichnet. Die Organisation, die die Anwendung betreibt, entscheidet, was ihr Server aufzeichnet.

## Wie lange es gespeichert wird

- Das Cookie `wpm-theme` der Website: 400 Kalendertage, nachdem Sie zuletzt ein Farbschema gewählt haben; länger hält ein Browser kein Cookie.
- Das Cookie `wpm-locale` der Anwendung: 365 Kalendertage, nachdem Sie zuletzt eine Seite geöffnet haben.
- Das Anmelde-Cookie: bis Sie sich abmelden oder den Browser schließen.
- Lokaler Speicher: bis Sie ihn löschen.
- Sitzungsspeicher: bis Sie den Tab schließen.

## Wie Sie es entfernen

Löschen Sie die Websitedaten in den Einstellungen Ihres Browsers (Cookies und Websitedaten), für die Website, die Anwendung oder beide. Die Website verhält sich dann wie beim ersten Besuch; die Anwendung bittet Sie, sich erneut anzumelden. Das Abmelden von der Anwendung entfernt das Anmelde-Cookie.

## Warum es kein Einwilligungsbanner gibt

Jeder Eintrag wird gespeichert, um Ihnen den Dienst zu bieten, den Sie angefordert haben: die Website oder die Anwendung so, wie Sie sie eingestellt haben, in Ihrer Sprache und, in der Anwendung, angemeldet. Diese Art der Speicherung gilt nach den Cookie-Regeln des Vereinigten Königreichs und der EU (den britischen Privacy and Electronic Communications Regulations und der Datenschutzrichtlinie für elektronische Kommunikation) als unbedingt erforderlich, und sie braucht keine Einwilligung. Das ist die Lesart des Projekts zu diesen Regeln, keine Rechtsberatung.

## Die Datensätze, die die Anwendung speichert

Dieser Hinweis behandelt, was in Ihrem Browser gespeichert wird. Die Anwendung speichert auf ihrem Server außerdem Datensätze über Menschen (Beschäftigte, Bewerberinnen und Bewerber sowie deren Kontakte). Wer die Anwendung betreibt, ist für diese Datensätze verantwortlich und schreibt den Datenschutzhinweis für die Menschen darin ([die Prüfliste zur Bewertung](../../../spec/governance/assessment-checklist.md), Englisch, Frage 3.10). Die Demonstrationssoftware enthält nur synthetische Daten.

## Fragen

Fragen Sie im Repository des Projekts, <https://github.com/workforce-planning-management/workforce-planning-management>, oder schreiben Sie an die dort genannte Person, die das Projekt pflegt.
