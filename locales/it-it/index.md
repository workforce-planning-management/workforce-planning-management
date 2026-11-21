# Pianificazione della forza lavoro

**Una piattaforma HR open source che copre l'intero ciclo di vita del dipendente, dall'assunzione al pensionamento.**

Pianificazione della forza lavoro (Workforce Planning Management, WPM) riunisce in un'unica applicazione selezione del personale, presenze e assenze, sviluppo dei dipendenti, pianificazione della forza lavoro e paghe. È un software dimostrativo, pensato per la valutazione e come progetto di riferimento.

Lingua: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Software dimostrativo, non ancora pronto per l'uso in produzione.** WPM usa solo dati sintetici e i suoi calcoli previsti dalla legge hanno valore illustrativo. Non inserire dati personali reali. [Il percorso verso la produzione](path-to-production/index.md) indica che cosa richiederebbe l'uso in produzione e a che punto è ciascuna parte. Vedi anche [la posizione normativa](../../spec/regulatory.md).

[Avvia la demo](start/index.md) [Schede per ruolo](for/index.md) [Codice sorgente su GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Vantaggi

I dati HR sono spesso sparsi in strumenti diversi, come le caselle di posta dei candidati, i fogli di calcolo delle ferie e le bacheche dei turni. Ogni passaggio dall'uno all'altro comporta lavoro manuale e un rischio di errore. WPM è progettato per ridurre questi passaggi.

- **Sistema unico.** Inserimento, ferie, formazione e dati retributivi sono conservati in un'unica applicazione, il che riduce le reimmissioni.
- **Minimizzazione dei dati.** I dati che non devono essere conservati non hanno un campo, e quelli che non devono essere divulgati non hanno una rotta API. Per esempio, non esiste un campo per la diagnosi e le risposte ai sondaggi di clima non hanno un autore.
- **Regole di rendicontazione.** I valori sconosciuti non sono riportati come zero, gli aggregati non identificano le persone e le stime dichiarano su che cosa si fondano.
- **Localizzazione.** L'applicazione è disponibile in 17 lingue e varianti regionali e le schede per ruolo in 21, comprese le lingue scritte da destra a sinistra. Le regole specifiche di ogni paese sono conservate come dati. Le traduzioni sono state prodotte da un'IA e non sono state riviste da madrelingua.
- **Tracciabilità.** Il codice sorgente è aperto e ogni comportamento è riconducibile a un requisito numerato, a una decisione di progetto e a un'attività.
- **Distribuzione indipendente.** Lo stack è composto da un'API JSON in Rust, un client SvelteKit e PostgreSQL. Si compila da un clone appena scaricato e le integrazioni con le applicazioni sorelle sono facoltative.

## Funzionalità

- **Selezione e anagrafiche.** Richieste di personale, flusso dei candidati, inserimento dei nuovi assunti, anagrafiche dei lavoratori e organigramma.
- **Tempi e turni.** Presenze e assenze, turni con controlli sull'orario di lavoro, turni di reperibilità con scambi e promemoria, e una rubrica del personale.
- **Sviluppo.** Valutazioni, feedback a 360°, formazione, analisi del divario di competenze, raccomandazioni sul tempo di formazione e formazione obbligatoria.
- **Pianificazione e reportistica.** Pianificazione strategica della forza lavoro, metriche e approfondimenti, e una dashboard per l'amministratore delegato.
- **Ingressi e uscite.** Liste di controllo datate e una consegna dell'ultimo giorno con traccia di audit.
- **Comunicazione.** Annunci con destinatari, collegamenti e conteggi di lettura.
- **Diritti e benessere dei dipendenti.** Spunti per il benessere, un sondaggio di clima anonimo, adeguamenti, accesso ai propri dati, cancellazione e conservazione.
- **Paghe.** Paghe e compensi, con calcoli previsti dalla legge a titolo illustrativo.

## Confronto

La tabella confronta WPM con tre categorie di sistemi HR. È un orientamento generale, non un benchmark. Vedi anche il [confronto per prodotto](../../docs/compare/index.md).

| | WPM | Suite tutto in uno (Workday, SAP SuccessFactors, BambooHR) | HR open source (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Strumenti specialistici (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Ambito** | Ciclo di vita completo, dall'assunzione al pensionamento | Ambito simile su un'unica piattaforma | Stesse funzioni di base | Una sola funzione |
| **Licenza** | Open source | Commerciale | Open source | Commerciale |
| **Punto di forza** | Minimizzazione dei dati, progetto tracciabile, analisi del divario di competenze e del tempo di formazione | Una piattaforma integrata e supportata | Più integrazioni, localizzazioni e installazioni | Profondità in una sola funzione |
| **Uso tipico** | Valutazione, progetto di riferimento o ulteriore sviluppo | Organizzazioni che hanno bisogno di un sistema supportato | Organizzazioni che vogliono l'open source in produzione | Organizzazioni che hanno bisogno di profondità in una funzione e sanno integrare il resto |

## A chi è rivolto

- **Dirigenti.** Le [schede per ruolo](for/index.md) illustrano che cosa WPM offre a ciascun ruolo dirigenziale e come leggerne i risultati.
- **Team HR.** Il [tutorial della demo](start/index.md) copre ogni pagina con dati fittizi.
- **Sviluppatori e architetti.** Vedi la [specifica](../../spec/index.md) e la [guida all'installazione](../../INSTALL.md).
- **Valutatori.** La [valutazione di prontezza](../../spec/readiness-assessment.md) elenca le lacune tra WPM e l'uso in produzione.

## Come ottenere aiuto

- **Segnala un errore o fai una domanda** nel [sistema di tracciamento delle segnalazioni](https://github.com/workforce-planning-management/workforce-planning-management/issues). Indica la pagina, che cosa hai fatto, che cosa ti aspettavi e che cosa è successo. Non includere dati personali reali.
- **Segnala un problema di sicurezza** per e-mail, come descritto nella [politica di sicurezza](../../SECURITY.md), non in una segnalazione pubblica.
- **Impara un'attività nell'applicazione.** Il collegamento **Aiuto** nell'intestazione dell'applicazione apre la pagina di aiuto e le guide passo passo per dipendenti, responsabili e HR. Anche ogni messaggio di errore rimanda a essa.

## Passi successivi

1. **Leggi la scheda del tuo ruolo.** Ogni scheda descrive che cosa offre WPM e che cosa resta aperto.
2. **Avvia la demo.** Il percorso guidato gira su un computer locale con dati fittizi.
3. **Invia un riscontro.** Vedi [come commentare](../../RFC.md) e [come contribuire](../../CONTRIBUTING.md).

[Avvia la demo](start/index.md) [Documentazione](../../index.md)

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/`), prevale la versione inglese.
