# Per il direttore operativo

**Prospettiva:** abbiamo le persone giuste, nel posto giusto, al momento giusto e
che cosa si rompe se qualcuno è assente?

## Che cosa ottieni

| Vuoi sapere | Dove risponde WPM | Leggi |
| --- | --- | --- |
| Se un team può coprire un'assenza | Sostituti per ogni persona, i turni di reperibilità con scambi e promemoria e una vista della copertura che dice chiaramente quando nessuno è disponibile | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Tempi, presenze e assenze | Registrazioni orarie, richieste di assenza con un percorso di approvazione, ratei e turni | [workforce-management.md](../../../../spec/workforce-management.md) |
| Se i limiti dell'orario di lavoro sono rispettati | Controlli consultivi sull'orario di lavoro; avvisano e non bloccano | WPM-R27, stesso file |
| La capacità per competenza, mese per mese | Bacini di competenze, offerta e domanda dei programmi, impegni dei partner, il bacino vincolante e una verifica di avvio che è un suggerimento, con le decisioni di avvio registrate | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Di che cosa avremo bisogno e dove sta il divario | Piani della forza lavoro, previsione e analisi dei divari per reparto e per ruolo | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Quali contratti scadono a breve | Una data di fine per fondamento dell'incarico, proroghe con storico, promemoria prima di una scadenza, un elenco degli incarichi senza data di fine e piani di conversione | [engagements.md](../../../../spec/engagements.md) |
| Dove il personale non a tempo indeterminato ha una quota elevata | La quota di FTE costituita da collaboratori esterni e quella a tempo determinato, ciascuna a parte, e i finanziamenti che terminano mese per mese | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Ingressi e uscite | Liste di controllo datate e una consegna dell'ultimo giorno | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Divari di competenze e tempo di formazione | Analisi dei divari con le prove su cui si fonda una raccomandazione | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Come leggere la vista della capacità

La capacità è pianificata per bacino di competenze, non per persona. Un bacino-mese dà uno di quattro risultati e una
competenza non dichiarata è *sconosciuta*, non una carenza. La verifica di avvio è un suggerimento e una persona registra la decisione di avviare.
La pianificazione con catena critica e percorso critico non è qui: appartiene a project-portfolio-management.

## Limiti

- La quota di personale non a tempo indeterminato non ha ancora un andamento: servono istantanee dell'organico suddivise per fondamento (WPM-T139, non realizzato).
- Un bacino per la vista del personale non a tempo indeterminato è il reparto, finché i piani non definiscono i bacini.
- La pianificazione dei turni e i calcoli dell'imposta sulle paghe non sostituiscono i tuoi sistemi di turnazione o di paghe.
- Diverse viste caricano in memoria gli archivi di un'organizzazione; non è stato misurato alcun dato di carico
  ([readiness-assessment.md](../../../../spec/readiness-assessment.md), lacuna 8).

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
