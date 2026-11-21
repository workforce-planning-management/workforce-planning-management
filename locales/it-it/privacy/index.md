# Informativa sulla privacy

Questa informativa indica che cosa il sito web di Pianificazione della forza lavoro (Workforce Planning Management, WPM) e l'applicazione WPM conservano nel
tuo browser, perché, per quanto tempo e come rimuoverlo. Il sito web è
<https://workforce-planning-management.github.io/>; l'applicazione è il software dimostrativo WPM, ovunque
venga eseguito. (Aggiornata il 2026-11-17, WPM-R253.)

Né l'uno né l'altra hanno strumenti di analisi, pubblicità, né script, font o immagini di terze parti. Nulla di ciò che l'uno o l'altra conservano nel
tuo browser viene usato per seguirti da un sito all'altro.

## Il sito web

Il sito web è un insieme di pagine statiche. Non ha accesso né moduli. Ciò che conserva si trova nel tuo browser:

| Che cosa | Dove è conservato | Nome | Che cosa è conservato | Perché |
| --- | --- | --- | --- | --- |
| Il tuo tema | Archiviazione locale del browser | `mxi.wpm.theme` | Il nome del tema, per esempio `dark` | Perché la pagina successiva e la tua prossima visita abbiano l'aspetto che hai scelto |
| Il tuo tema | Un cookie solo per questo sito web | `wpm-theme` | Il nome del tema, per esempio `dark` | Perché il tema ritorni anche se il browser conserva i cookie ma cancella l'archiviazione locale, o viceversa |
| La dimensione del testo | Archiviazione locale del browser | `mxi.wpm.text-size` | Il nome della dimensione, per esempio `large` | Perché la pagina successiva e la tua prossima visita usino la dimensione del testo che hai scelto |
| La tua lingua | Non viene conservato nulla | | | La lingua fa parte dell'indirizzo di ogni pagina (per esempio `/cy-001/`), quindi non c'è nulla da ricordare |
| Dove ti trovavi in una pagina | Archiviazione di sessione del browser, solo per questa scheda | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Le posizioni di scorrimento e l'ordine delle pagine aperte in questa scheda | Perché Indietro e Avanti ti riportino dove eri. Li conserva il framework con cui è costruito il sito web (SvelteKit) |

Nessuna scelta viene conservata finché non la fai. Fino a quando non scegli, il sito web segue l'impostazione chiara o scura del browser
e la sua dimensione normale del testo, e non conserva né tema né dimensione del testo. Una scelta viene conservata quando la fai nella barra dei selettori,
e una nuova scelta la sostituisce. L'archiviazione di sessione viene scritta mentre passi da una pagina all'altra.

Altre parti del sito web non conservano nulla:

- **La ricerca** viene eseguita nel tuo browser, su un indice che il sito web pubblica, e non conserva nulla. Le tue parole sono inserite
  nell'indirizzo della pagina (`?q=`) in modo che una pagina di risultati possa essere condivisa; quando quell'indirizzo viene caricato dal server web (a un
  ricaricamento o da un collegamento condiviso), il server lo riceve con la richiesta, come per qualsiasi indirizzo.
- **I cursori di prontezza** nel percorso verso la produzione non conservano nulla e un ricaricamento ripristina le
  stime dell'autore.
- **Condividi** apre il sito che scegli (e-mail, LinkedIn, Reddit, Bluesky o Mastodon) solo quando lo scegli. Si applica
  allora l'informativa sulla privacy di quel sito.

## L'applicazione WPM

L'applicazione conserva gli stessi tipi di scelte, più il cookie che ti mantiene connesso:

| Che cosa | Dove è conservato | Nome | Che cosa è conservato | Perché |
| --- | --- | --- | --- | --- |
| Il tuo tema | Archiviazione locale del browser | `mxi.wpm.theme` | Il nome del tema | Perché ogni pagina abbia l'aspetto che hai scelto |
| La dimensione del testo | Archiviazione locale del browser | `mxi.wpm.text-size` | Il nome della dimensione | Perché ogni pagina usi la dimensione del testo che hai scelto |
| La tua lingua | Archiviazione locale del browser | `mxi.wpm.locale` | Il codice della lingua dell'ultima pagina aperta, per esempio `cy-001` | Perché un indirizzo senza lingua si apra nella tua |
| La tua lingua | Un cookie solo per l'applicazione | `wpm-locale` | Lo stesso codice della lingua | Perché il server possa indirizzare alla tua lingua un indirizzo che non ne ha una |
| Il tuo accesso | Un cookie solo per l'applicazione, che gli script della pagina non possono leggere | `__Host-mxi_session` (`mxi_session` sul computer di uno sviluppatore) | Un numero casuale che rappresenta la tua sessione di accesso | Perché tu resti connesso da una pagina all'altra; senza di esso dovresti accedere a ogni pagina |
| La lingua dell'editor dei contenuti | Archiviazione locale del browser, solo se apri l'editor dei contenuti (`/admin`) | `sveltia-cms.prefs` | Le impostazioni dell'editor, tra cui la sua lingua | Perché l'editor si apra nella lingua dell'applicazione |
| Dove ti trovavi in una pagina | Archiviazione di sessione del browser, solo per questa scheda | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Le posizioni di scorrimento e l'ordine delle pagine aperte in questa scheda | Perché Indietro e Avanti ti riportino dove eri |

A differenza del sito web, l'applicazione conserva il tema e la dimensione del testo non appena li mostra, prima che tu
scelga, e conserva la lingua di ogni pagina che apri. Le chiavi precedenti al cambio di nome del progetto (`mxi.hcm.theme`, `mxi.hcm.locale`) vengono spostate nei
nomi indicati sopra ed eliminate.

Il cookie di accesso è l'unico elemento che rappresenta te. Il numero casuale al suo interno non significa nulla da solo: la
sessione di accesso a cui rimanda è conservata dal server dell'applicazione e la disconnessione la termina. Ciò che l'applicazione
conserva sulle persone presenti nei suoi archivi è trattato nella penultima sezione, più avanti.

## Chi li riceve

L'archiviazione locale e l'archiviazione di sessione non lasciano mai il tuo browser.

Un cookie viene inviato con ogni richiesta che il tuo browser fa al sito che lo ha impostato, e a nessun altro sito
(`SameSite=Lax`, e `Secure` su HTTPS):

- Il cookie `wpm-theme` del sito web va a GitHub Pages, che ospita il sito web. Nessun programma del progetto
  lo legge.
- I cookie `wpm-locale` e di accesso dell'applicazione vanno al server su cui l'applicazione è in esecuzione, che li legge
  per scegliere la tua lingua e sapere che sei connesso.

GitHub, che ospita il sito web, riceve le richieste che il tuo browser fa, come qualsiasi host web.
[L'informativa sulla privacy di GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
indica che cosa registra. L'organizzazione che gestisce l'applicazione decide che cosa registra il suo server.

## Per quanto tempo sono conservati

- Il cookie `wpm-theme` del sito web: 400 giorni di calendario dopo l'ultima volta che hai scelto un tema, il periodo più lungo per cui un browser conserva un
  cookie.
- Il cookie `wpm-locale` dell'applicazione: 365 giorni di calendario dopo l'ultima volta che hai aperto una pagina.
- Il cookie di accesso: fino a quando non ti disconnetti o chiudi il browser.
- L'archiviazione locale: fino a quando non la cancelli.
- L'archiviazione di sessione: fino a quando non chiudi la scheda.

## Come rimuoverli

Cancella i dati del sito nelle impostazioni del browser (cookie e dati dei siti), per il sito web, per l'applicazione o
per entrambi. Il sito web si comporta allora come alla prima visita; l'applicazione ti chiede di accedere di nuovo. La disconnessione
dall'applicazione rimuove il cookie di accesso.

## Perché non c'è un banner per il consenso

Ogni elemento è conservato per offrirti il servizio che hai richiesto: il sito web o l'applicazione come li hai impostati, nella
tua lingua e, nell'applicazione, con l'accesso effettuato. È il tipo di archiviazione che le norme del Regno Unito e dell'UE sui cookie (le
Privacy and Electronic Communications Regulations e la direttiva ePrivacy) considerano strettamente necessaria, e che
non richiede il consenso. È l'interpretazione di queste norme data dal progetto, non una consulenza legale.

## Gli archivi che l'applicazione conserva

Questa informativa riguarda ciò che è conservato nel tuo browser. L'applicazione conserva anche archivi di persone (dipendenti,
candidati e i loro contatti) sul proprio server. Chi gestisce l'applicazione è responsabile di quegli archivi e
redige l'informativa sulla privacy per le persone che vi compaiono
([la lista di controllo della valutazione](../../../spec/governance/assessment-checklist.md), domanda 3.10). Il software dimostrativo
conserva solo dati sintetici.

## Domande

Chiedi nel repository del progetto, <https://github.com/workforce-planning-management/workforce-planning-management>,
oppure scrivi al responsabile della manutenzione indicato lì.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/privacy/`), prevale la versione inglese.
