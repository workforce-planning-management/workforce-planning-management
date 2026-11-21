# Per il direttore delle risorse umane

**Prospettiva:** questo supporta l'intero ciclo di vita del dipendente e posso usarlo senza
conservare sulle persone più di quanto dovrei?

## Che cosa ottieni

| Fase | Che cosa conserva WPM | Leggi |
| --- | --- | --- |
| Selezione | Richieste di personale, un flusso dei candidati, un bacino di candidati, liste di controllo per l'inserimento | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| L'anagrafica | Un'anagrafica di impiego per ogni lavoratore, l'organigramma, le linee gerarchiche, i gruppi, i trasferimenti tra organizzazioni | [hr-core.md](../../../../spec/hr-core.md) |
| Self-service | Un lavoratore vede e aggiorna i propri dati, chiede il lavoro flessibile, dà il preavviso e vede le proprie assenze | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Vita lavorativa | Assenze, tempi, turni, benefit, spunti per il benessere, adeguamenti ragionevoli, valutazioni ergonomiche, il sondaggio di clima anonimo | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Sviluppo | Valutazioni, valutazioni a 360°, corsi tramite il registro dei corsi, test di valutazione, programmi di mentoring, piani di sviluppo, successione, inizio carriera | [talent-development.md](../../../../spec/talent-development.md) |
| Paghe | Elaborazioni delle paghe, cedolini, confronto retributivo di riferimento, una ricerca nella scala retributiva nazionale, livelli di inquadramento | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Personale non a tempo indeterminato | Date di fine per fondamento, proroghe, valutazioni dello stato, piani di conversione decisi da una persona, finanziamenti accanto alla fine del contratto | [engagements.md](../../../../spec/engagements.md) |
| Uscita | Dimissioni con un periodo di preavviso, una consegna dell'ultimo giorno, conservazione per tipo di archivio, cancellazione | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Come protegge le persone

- **Necessità di sapere sulle letture.** Un responsabile vede i propri collaboratori diretti, un lavoratore vede se stesso, le HR vedono ciò che serve alle HR.
  Un estraneo che ha effettuato l'accesso non vede nulla su un collega ([auth.md](../../../../spec/auth.md)).
- **Non viene conservata alcuna diagnosi.** L'assenza per malattia registra un tipo, non un motivo.
- **Le funzioni per dati di categoria particolare sono disattivate per impostazione predefinita.** Il monitoraggio della parità e i requisiti di salute sul lavoro richiedono una
  base giuridica registrata, sono volontari o tenuti dal ruolo di medicina del lavoro e mostrano solo gruppi sopra una soglia.
  Non sono mai un dato di ingresso di alcun punteggio o decisione su una persona.
- **I punteggi consigliano.** La prontezza alla successione, i divari di competenze e le previsioni suggeriscono; decide una persona, e decisioni come
  la conversione di un collaboratore esterno sono registrate come decisione di una persona, mai come esito automatico.
- **L'interessato può vedere e cancellare.** L'esportazione per l'accesso ai propri dati e la cancellazione coprono gli archivi sopra elencati, con un registro affinché
  un ripristino non riporti indietro una persona cancellata.

## Limiti

- Non ancora realizzati: un'interfaccia per i piani di conversione e i finanziamenti, l'andamento del personale non a tempo indeterminato e la
  pagina «i miei dati» del lavoratore. Controlla [tasks.md](../../../../spec/tasks.md) per lo stato attuale.
- Le traduzioni sono assistite da macchina; una revisione professionale è una porta ancora aperta.
- I contratti collettivi e gli organismi riconosciuti sono solo ricerca e proposta ([worker-unions.md](../../../../spec/worker-unions.md)).
- L'imposta sulle paghe è un segnaposto.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
