# Il percorso verso la produzione

WPM è un software dimostrativo. Questa pagina dice che cosa significherebbe per esso l'uso in produzione ed elenca ciò che separa WPM da quell'uso. Ogni voce riporta una stima di quanto sia pronta, da 0 a 100 per cento.

## Che cosa significa produzione

Uso in produzione significa che un'organizzazione si affida a WPM per un lavoro reale:

- gli archivi sono quelli di persone reali: dipendenti, candidati e i loro contatti di emergenza;
- le paghe calcolate da WPM vengono pagate e a esse si applica la legge del luogo di lavoro;
- le persone accedono ogni giorno e una parte di ciò che leggono è sensibile, come le assenze per malattia;
- l'organizzazione risponde a un'autorità di vigilanza, a un revisore e alle persone presenti negli archivi di come viene gestita.

Questa pagina misura WPM rispetto al caso più impegnativo per cui è stato valutato: una grande organizzazione sanitaria o governativa ([la valutazione di prontezza](../../../spec/readiness-assessment.md)). Un'organizzazione più piccola può aver bisogno di meno per alcune voci, ma le servono gli stessi tipi di prove.

La produzione richiede tre tipi di prove e il solo software ne fornisce soltanto il primo:

1. **Software** che svolge il proprio compito in modo sicuro, con test che lo dimostrano.
2. **Un'organizzazione che lo installa** e che lo ospita e lo gestisce: backup, monitoraggio, assistenza e risposta agli incidenti.
3. **Persone diverse dall'autore** che lo hanno verificato: un esperto di test di intrusione, un revisore dell'accessibilità, i traduttori e il responsabile della protezione dei dati di chi lo installa.

## Come leggere le stime

- Una stima è il giudizio dell'autore al 2026-11-13, formulato sulla base della specifica e dei test. Non è una misurazione né una certificazione.
- 0 per cento significa che non esiste ancora nulla. 100 per cento significa realizzato, testato e verificato da una persona indipendente su un'installazione reale. Nessuna voce è al 100 per cento.
- Ogni voce dice su che cosa si fonda la sua stima e che cosa resta da fare.
- Le stime non vengono mediate. Una sola voce con una stima bassa basta a tenere WPM fuori dalla produzione.

## Il software

- **75%** **Autenticazione e accesso in base alla necessità di sapere.** L'autenticazione è attiva per impostazione predefinita e il servizio rifiuta di avviarsi in produzione senza di esso (WPM-R72). Le letture sono limitate alle persone che hanno necessità di sapere (WPM-T188) e un token ha un'età massima (WPM-T200). Si fonda su: i test di applicazione che girano nell'integrazione continua. Resta da fare: non esiste un elenco di revoca dei token e non è stato eseguito alcun test di intrusione indipendente.
- **70%** **Una traccia di audit completa e non alterata.** La traccia di audit è di sola aggiunta e collegata da hash, e può essere esportata verso un sistema di monitoraggio (WPM-T189, WPM-T197). Si fonda su: test che sono stati rotti di proposito per dimostrare che falliscono. Resta da fare: la coda troncata della catena non viene rilevata e il proprietario di un database può ancora disattivare i trigger.
- **70%** **Protezione dei dati fin dalla progettazione.** Le persone sono identificate tramite riferimenti, non ci sono colonne demografiche e sono previsti accesso ai propri dati, cancellazione e conservazione per ogni tipo di archivio. Si fonda su: i test e la sezione 1 della valutazione di prontezza. Resta da fare: la valutazione e la base giuridica proprie di chi installa (più avanti).
- **75%** **Connessioni e backup cifrati.** In produzione il servizio rifiuta una connessione al database in chiaro e interrompe per timeout una richiesta bloccata (WPM-T191), e i backup sono cifrati (WPM-T192). Si fonda su: i test di entrambi. Resta da fare: la gestione di chiavi e certificati spetta a chi installa.
- **10%** **Calcoli previsti dalla legge.** I calcoli previsti dalla legge hanno valore illustrativo e l'imposta sulle paghe è un segnaposto ([la posizione normativa](../../../spec/regulatory.md)). Si fonda su: la specifica, che lo dice. Resta da fare: le regole di ogni luogo di lavoro, scritte e verificate da una persona qualificata. Non pagare nessuno con WPM.
- **50%** **Scala.** Su un solo portatile, le pagine della rubrica e la vista della capacità non falliscono con 50.000 lavoratori e le viste di pianificazione ne rifiutano più di 10.000 (WPM-T196, WPM-T230). Si fonda su: una prova misurata ([BENCHMARKS.md](../../../BENCHMARKS.md)). Resta da fare: la vista della capacità cresce ancora in proporzione alla forza lavoro e non è stato misurato nulla sull'hardware di chi installa.
- **65%** **Catena di fornitura.** Ogni azione dell'integrazione continua è fissata a un commit e la cronologia viene analizzata alla ricerca di segreti. L'immagine del container è firmata e porta con sé la propria provenienza, e ciò è stato eseguito per la versione v0.1.2 (WPM-T194, WPM-T195). Si fonda su: quell'esecuzione della versione. Resta da fare: il pacchetto dell'immagine è privato, quindi nessuno al di fuori del progetto può ancora verificarlo.
- **60%** **Monitoraggio.** Ogni risposta porta un identificativo della richiesta e le righe di log non contengono stringhe di query né identificativi personali (WPM-T197). Si fonda su: i test del log delle richieste. Resta da fare: dashboard e avvisi spettano a chi installa e i limiti di frequenza si applicano a una sola istanza (WPM-T198).
- **60%** **Backup e ripristino.** Backup cifrati, una prova di ripristino che gira nell'integrazione continua e un manuale operativo. Si fonda su: la prova. Resta da fare: una prova di ripristino sulla piattaforma di chi installa negli ultimi 90 giorni di calendario (porta WPM-G4).
- **40%** **Verifiche di accessibilità.** Verifiche automatiche rispetto a WCAG 2.2 AA girano sull'applicazione e su questo sito, con i temi chiaro e scuro. Si fonda su: i test Playwright e axe. Resta da fare: l'audit indipendente (più avanti); non è stato usato alcun lettore di schermo.
- **20%** **Traduzioni.** L'applicazione ha 25 lingue e varianti regionali e le schede per ruolo ne hanno 21. Si fonda su: il testo esiste. Resta da fare: le traduzioni sono state scritte da un'IA e nessuno le ha riviste in modo professionale (WPM-T155).

## Verifiche svolte da persone diverse dall'autore

- **0%** **Test di intrusione indipendente.** Non è stato eseguito alcun test di intrusione su un'istanza installata (WPM-T201). Si fonda su: l'attività è aperta. Resta da fare: il test e la correzione di ciò che emerge.
- **0%** **Audit di accessibilità indipendente.** Non è stato svolto alcun audit rispetto a WCAG 2.2 AA con persone che usano tecnologie assistive (WPM-T202). Si fonda su: l'attività è aperta. Resta da fare: l'audit e la correzione di ciò che emerge.
- **25%** **Modello delle minacce.** Esiste un modello delle minacce iniziale ([threat-model.md](../../../spec/governance/threat-model.md)), scritto dall'autore. Si fonda su: il documento. Resta da fare: un team indipendente lo rifà per un'installazione reale.

## La parte di chi installa

- **20%** **Valutazione d'impatto sulla protezione dei dati.** Esiste un pacchetto da adattare: una valutazione d'impatto, un flusso di dati, un registro dei trattamenti e un calendario di conservazione ([spec/governance](../../../spec/governance/dpia.md)). Si fonda su: il pacchetto. Resta da fare: non c'è chi installa, quindi nessun responsabile della protezione dei dati ha approvato una valutazione, una base giuridica, un calendario di conservazione o un accordo con il responsabile del trattamento (porta WPM-G5).
- **15%** **Valutazioni d'impatto sulla parità.** Esistono un modello e un registro ([equality-impact](../../../spec/equality-impact/template.md)). Si fonda su: il modello. Resta da fare: chi installa scrive e approva una valutazione per il proprio uso.
- **40%** **Hosting e gestione.** Esistono una configurazione di produzione, un'immagine del container e documenti per l'installazione e gli aggiornamenti. Si fonda su: quei documenti e la prova di ripristino. Resta da fare: nulla è stato eseguito su un host rinforzato e chi installa deve gestire il servizio.
- **10%** **Assistenza.** Esistono un sistema di tracciamento delle segnalazioni e una politica di sicurezza ([SECURITY.md](../../../SECURITY.md)). Si fonda su: queste due cose. Resta da fare: non c'è un contratto di assistenza, un livello di servizio, un turno di reperibilità né un'organizzazione dietro WPM.

## Che cosa questa pagina non afferma

La pagina non registra alcuna decisione di entrare in produzione. Quella decisione spetta a chi installa ed è presa al di fuori di WPM. Le stime riguardano solo le voci elencate e vale anche qui l'elenco, contenuto nella valutazione di prontezza, di ciò che essa non ha verificato. Dove è collegato un sistema clinico, una revisione della sicurezza clinica è decisione di chi installa.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/path-to-production/`), prevale la versione inglese.
