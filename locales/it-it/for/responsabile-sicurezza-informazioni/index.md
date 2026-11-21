# Per il responsabile della sicurezza delle informazioni

**Prospettiva:** come è protetto, come ci accorgeremmo se fosse attaccato o usato in modo improprio e
che cosa nessuno ha ancora testato?

## Che cosa ottieni

| Esigenza | Che cosa esiste | Leggi |
| --- | --- | --- |
| Minacce | Un modello delle minacce: risorse, attori, confini di fiducia, il controllo come realizzato e ciò che **non è valutato** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Autenticazione | Accesso attivo per impostazione predefinita; la produzione rifiuta di avviarsi se è disattivato o se manca una fonte della chiave dei token; accesso tramite la directory aziendale (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Autorizzazione | La politica decide solo in base all'azione; una tabella classificata decide la necessità di sapere su ogni rotta di lettura e un test di scansione del sorgente fallisce su una rotta non classificata | [auth.md](../../../../spec/auth.md) |
| Audit | Una traccia di sola aggiunta, collegata da hash, con un endpoint di verifica; le voci non contengono testo di motivazione né valori | [audit.md](../../../../spec/audit.md) |
| Trasporto e browser | Intestazioni di sicurezza su ogni risposta dell'API, un elenco esplicito di origini consentite CORS che impedisce l'avvio se non è impostato, `no-store` su `/api` | WPM-R73 in [production-readiness.md](../../../../spec/production-readiness.md) |
| Abuso | Limiti di frequenza per chiamante, più severi per le rotte di cancellazione, importazione, esportazione e token | WPM-R74, stesso file |
| Configurazione insicura | L'avvio rifiuta una connessione al database in chiaro o un Redis in chiaro verso un altro host, e un'origine CORS mancante | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Dati a riposo | Backup cifrati; la cache condivisa contiene solo ricerche non personali e fallisce in modo aperto | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Catena di fornitura | Controlli di licenza e di divieto con `cargo deny`, crate condivisi con versione fissata, una distinta dei componenti software a ogni versione | [production-readiness.md](../../../../spec/production-readiness.md) |
| Incidenti | Un manuale operativo con le prime azioni; tu aggiungi i contatti e gli obblighi di notifica | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Autovalutazione | Una lista di controllo dei controlli: soddisfatto, soddisfatto in parte, non soddisfatto, risposta a carico di chi installa | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Lacune note, come registrate

- Non è stato eseguito alcun test di intrusione indipendente. I test sono stati scritti dall'autore.
- Le azioni sono fissate per tag, non per digest, e le immagini non sono firmate.
- I limiti di frequenza sono per istanza.
- L'applicazione usa il ruolo di database che le viene dato; non è fornito alcun insieme di ruoli a privilegi minimi.
- La revoca dei token e la loro età massima dipendono dal tuo provider di identità.
- Il percorso del gestore dei contenuti carica uno script da una CDN pubblica senza versione fissata né hash di integrità.
- I log non sono stati controllati per cercarvi dati personali e la traccia di audit non è esportata verso un sistema di monitoraggio.
- Un Redis gestito (cluster, sentinel, utenti con controllo degli accessi) non è stato provato.

## Che cosa possono fare solo le persone

Commissionare il test di intrusione indipendente, rivedere il modello delle minacce per il tuo ambiente, eseguire una prova di ripristino sulla
tua piattaforma e impostare la protezione dei rami sulla piattaforma di hosting. Vedi la
[valutazione di prontezza](../../../../spec/readiness-assessment.md), il cui verdetto è **non pronto** per un'installazione ad alta garanzia.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
