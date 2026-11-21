# Test di sicurezza indipendente

**Prospettiva:** come facciamo attaccare questo software da persone che non lo hanno scritto, su che cosa
dobbiamo indirizzarle e che cosa facciamo di ciò che trovano?

Non è stato svolto alcun test indipendente. Il [modello delle minacce](../../../../spec/governance/threat-model.md) è stato scritto dallo
stesso assistente che ha scritto il software e non è stato rivisto da nessun altro, e ogni controllo che indica è stato messo alla prova da
quell'autore. La [valutazione di prontezza](../../../../spec/readiness-assessment.md) elenca un test di intrusione indipendente
come porta che solo le persone possono chiudere (WPM-T201, **non fatto**). Questa pagina è una scheda per chi ne commissiona uno. Non è un
piano di test di un tester e non ne prende il posto.

## Che cosa dare ai tester

| Elemento | Dove si trova | Note |
| --- | --- | --- |
| Un'installazione da attaccare | La tua: [guida all'installazione](../../../../spec/operations/deployment.md) | Impostazioni di produzione con l'accesso **attivo**, un provider di identità di prova e **solo dati sintetici**. Non testare su archivi di persone reali |
| Il sorgente | Questo repository | Un test con il sorgente è più approfondito di uno a scatola chiusa; di' quale vuoi |
| Il modello delle minacce | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Ventiquattro minacce, il controllo come realizzato e le lacune. Consegnalo loro **dopo** una prima passata a scatola chiusa se vuoi vedere che cosa trovano senza suggerimenti |
| La descrizione dell'API | `GET /api-docs/openapi.json` | Ogni rotta, con l'`Idempotency-Key` su ogni operazione che modifica qualcosa |
| Il progetto dell'accesso | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Personas, attributi e ciò che la politica decide solo in base all'azione |
| Identità di prova | Le crei tu | Almeno: un lavoratore senza attributi, un responsabile diretto, un responsabile fuori dalla catena, le HR, le paghe, un amministratore e un servizio pari |

## Che cosa chiedere loro di esaminare

L'autore non poteva verificare queste cose. Ogni riga indica la voce corrispondente del modello delle minacce, così un riscontro può essere ricondotto.

| Area | Che cosa provare | Minaccia |
| --- | --- | --- |
| Token | Confusione sull'algoritmo, un insieme di chiavi che ruota mentre le richieste sono in corso, un token scaduto o non ancora valido, un token per un altro destinatario | T1, T2 |
| Necessità di sapere | Ogni rotta di lettura come chiamante senza attributi e come vicino; una rotta che la tabella di classificazione non conosce; una scrittura che la politica consente ma che il gestore dovrebbe rifiutare | T3, T4, T6 |
| Attributi privilegiati | Se un attributo in un token può essere aggiunto, riprodotto o ereditato; chi può emetterne uno; un amministratore condiviso o di lunga durata | T24 |
| Iniezione | Fuzzing di ogni parametro, segmento di percorso e campo del corpo; le istruzioni di cancellazione e di conservazione sono costruite da identificatori tipizzati e sono state lette, non sottoposte a fuzzing | T12 |
| Il client per browser | Gestione della sessione da parte del server dell'interfaccia, come scambia una sessione con un token, cross-site scripting tramite testo memorizzato, un aggiramento della content security policy, la pagina del gestore dei contenuti che carica uno script da una rete pubblica senza versione fissata | T13, T14 |
| Richieste in uscita | Falsificazione di richieste lato server tramite qualsiasi URL configurato (insiemi di chiavi, servizi fratelli, cache condivisa) | T15 |
| Limiti | Il limite del corpo, il timeout della richiesta, i limiti di frequenza per istanza e le viste che caricano in memoria gli archivi di un'intera organizzazione | T16, T17 |
| Log | Se dati personali, token o stringhe di query arrivano in un log | T19 |
| Chiavi di idempotenza | Una chiave usata da due chiamanti; la stessa chiave con un corpo, un percorso o una query modificati; una chiave quasi valida; un corpo molto grande; due richieste in corsa sotto una chiave (l'affermazione è argomentata, non verificata con una corsa in un test); se una risposta memorizzata può essere letta da qualcuno diverso dal suo chiamante | nuova |
| La traccia di audit | Qualsiasi modo di modificare, rimuovere o saltare una voce; l'endpoint di verifica della catena | T7 |
| Backup | Ripristino da un backup manomesso o con chiave sbagliata; se una persona cancellata ritorna | T10, T11 |
| Catena di fornitura | Fissazione delle azioni di build, scansione dei segreti, firma delle immagini | T20, T21 |

## Regole da stabilire prima che inizino

Compilale insieme ai tuoi team di sicurezza e legale. Qui nessuna è decisa.

- **Autorizzazione:** chi firma che il test è autorizzato e per quali sistemi e indirizzi.
- **Finestra:** le date e gli orari, contati in giorni di calendario, e chi chiamare per fermarlo.
- **Ambito:** che cosa è incluso (l'API, l'interfaccia, il flusso di accesso, lo script di backup) e che cosa è escluso (il servizio
  del provider di identità, la tua rete, qualsiasi cosa condivisa con altri sistemi).
- **Dati:** solo sintetici. Se un tester trova dati personali reali, chi viene informato e come vengono gestiti.
- **Test distruttivi:** se sono consentiti negazione del servizio, scritture di massa e cancellazione, e dove.
- **Rapporto:** il formato, la scala di gravità, chi lo riceve, per quanto tempo può conservarlo e come vengono gestiti i segreti
  che trova.
- **Nuovo test:** che il contratto includa la verifica delle correzioni.

## Che cosa fare dei riscontri

1. Registra ciascuno come attività in [tasks.md](../../../../spec/tasks.md) con il riferimento del tester, le prove e la
   gravità che ha indicato. Un riscontro contestato resta registrato come contestato, con il motivo.
2. Correggi con un test che prima fallisce. Se la correzione è una regola, aggiungila alla colonna delle lacune del modello delle minacce e chiudila
   lì solo quando il nuovo test riesce.
3. Non segnare WPM-T201 come fatto finché non hai il rapporto in mano, ogni riscontro ha un responsabile e un nuovo test è completato. Aggiorna
   la [valutazione di prontezza](../../../../spec/readiness-assessment.md) e l'elenco delle porte (WPM-T203) nella stessa modifica.
4. Tieni il rapporto fuori dal repository se contiene qualcosa di sfruttabile; rimanda al luogo in cui è conservato.

## Che cosa significa e che cosa non significa un rapporto pulito

Un test senza riscontri mostra che questi tester, in questa finestra, con questo ambito e queste
identità, non hanno trovato nulla.
Non mostra l'assenza di difetti, non copre una configurazione che hai modificato in seguito
e non sostituisce le altre porte: un audit di accessibilità, la tua valutazione d'impatto sulla protezione
dei dati approvata e una prova di ripristino sulla tua piattaforma.

## Che cosa non fa questa pagina

Non nomina un fornitore, non fissa un prezzo e non promette uno standard. Quale schema o accreditamento il tuo settore si aspetti da un
tester spetta a te deciderlo.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
