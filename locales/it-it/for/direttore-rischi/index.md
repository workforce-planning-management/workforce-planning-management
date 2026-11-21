# Per il direttore dei rischi

**Prospettiva:** che cosa potrebbe andare storto con questo sistema, che cosa lo impedisce e che cosa nessuno
ha verificato?

## Il punto di partenza onesto

La [valutazione di prontezza](../../../../spec/readiness-assessment.md) del 2026-10-11 dà il verdetto **non pronto** per
un'installazione ad alta garanzia. Ha individuato un ostacolo sondando il servizio in esecuzione: qualsiasi persona che avesse effettuato l'accesso poteva leggere
l'assenza per malattia di un collega e il suo motivo in testo libero. Il problema è stato corretto e testato. Le porte che solo le persone possono chiudere sono
ancora aperte.

## Che cosa ottieni

| Esigenza | Che cosa esiste | Leggi |
| --- | --- | --- |
| Minacce a ogni confine | Un modello delle minacce: risorse, attori, confini, il controllo come realizzato e ciò che **non è valutato** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Letture non autorizzate | Una protezione basata sulla necessità di sapere che classifica ogni rotta di lettura; un test fallisce su una rotta non classificata | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Manomissione e responsabilità | Una traccia di audit di sola aggiunta, collegata da hash, con un endpoint di verifica; le voci non contengono testo di motivazione né valori | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Dati sensibili | Nessuna diagnosi conservata; funzioni per dati di categoria particolare disattivate per impostazione predefinita; una soglia per i gruppi piccoli | [regulatory.md](../../../../spec/regulatory.md) |
| Percorsi insicuri | La produzione rifiuta di avviarsi con l'accesso disattivato, senza una fonte della chiave dei token, con una connessione al database in chiaro verso un altro host o con un Redis remoto in chiaro | [production-readiness.md](../../../../spec/production-readiness.md) |
| Perdita di dati | Backup cifrati, una prova di ripristino eseguita nell'integrazione continua, un registro delle cancellazioni | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Distorsioni nei punteggi | Un modello di valutazione d'impatto sulla parità e un registro di ogni punteggio con una valutazione in bozza | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Un'autovalutazione dei controlli | Una lista di controllo in cui ogni risposta è soddisfatto, soddisfatto in parte, non soddisfatto o a carico di chi installa | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Rischi aperti, come registrati

La fissazione e la firma della catena di fornitura, le prove di carico, i controlli dei log per cercarvi dati personali, i limiti di frequenza condivisi tra
istanze, un insieme di ruoli di database a privilegi minimi e le indicazioni sulla revoca dei token sono voci aperte nella valutazione
di prontezza, ciascuna con un'attività. I test sono stati scritti dall'autore; nessuno è una revisione indipendente.

## Che cosa possono fare solo le persone

Un test di intrusione indipendente, un audit di accessibilità indipendente, la valutazione d'impatto di chi installa approvata,
una prova di ripristino sulla tua piattaforma e l'approvazione delle valutazioni d'impatto sulla parità. Il repository le indica
come porte e non le segna come completate.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
