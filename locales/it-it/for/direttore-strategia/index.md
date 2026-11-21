# Per il direttore della strategia

**Prospettiva:** la nostra forza lavoro è plasmata per servire la strategia, di che cosa avremo bisogno e che cosa
dobbiamo fare riguardo al divario?

## Che cosa ottieni

| Vuoi sapere | Dove risponde WPM | Leggi |
| --- | --- | --- |
| Che cosa richiede un ruolo | Profili di ruolo con le competenze richieste e un livello minimo di padronanza, così un divario si misura rispetto a un ruolo e non a una persona | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Di che cosa avremo bisogno | Piani della forza lavoro come scenari in bozza, più di uno affiancati, con ipotesi dichiarate e domanda per reparto, ruolo e periodo | WPM-R36, stesso file |
| Dove sta il divario | Offerta prevista rispetto alla domanda pianificata: un divario di organico e un divario di competenze, con le leve elencate come suggerimenti e le relative prove | WPM-R37 |
| Se serve la strategia | Obiettivi collegati alla domanda: obiettivi senza risorse, domanda non allineata, ruoli critici senza sostituti | WPM-R38 |
| Se possiamo realizzare un programma | Bacini di competenze, domanda dei programmi, impegni dei partner, il bacino vincolante, una verifica di avvio che è un suggerimento | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Quanto facciamo affidamento sul personale non a tempo indeterminato | La quota di FTE costituita da collaboratori esterni e quella a tempo determinato, ciascuna a parte, e quanti finanziamenti terminano in ciascuno dei prossimi dodici mesi | `GET /api/workforce-intelligence/contingent` (WPM-T109, in parte) |
| Che cosa pianifichiamo riguardo ai contratti in scadenza | Piani di conversione: la decisione di una persona di convertire, prorogare o terminare, con i finanziamenti accanto alla fine del contratto | [engagements.md](../../../../spec/engagements.md) |
| Che altro è pianificato | La tabella di marcia e l'elenco delle attività aperte | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Come leggere una previsione

- È una regola dichiarata applicata a dati dichiarati, non un modello, e ogni risultato indica le proprie ipotesi.
- Con uno storico dell'organico troppo breve dice `insufficient_history` invece di estrapolare.
- Un piano contiene solo un organico ipotetico. Non legge né modifica mai l'anagrafica di una persona.
- Le leve sono suggerimenti. Sceglie chi pianifica e il software non decide nulla.

## Limiti

- La leva «convertire il personale non a tempo indeterminato in personale a tempo indeterminato» nei piani è specificata (WPM-R63) e non ancora nel codice.
- La quota di personale non a tempo indeterminato non ha ancora un andamento: servono istantanee suddivise per fondamento (WPM-T139, non realizzato).
- La pianificazione con catena critica e percorso critico appartiene a project-portfolio-management, non a questo sistema.
- Non esiste ancora un'interfaccia per i finanziamenti o i piani di conversione.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
