# Per il direttore dei sistemi informativi

**Prospettiva:** come si inserisce nel nostro parco applicativo, chi può accedere, dove risiedono i dati
e come lo gestiamo e lo ripristiniamo?

## Che cosa ottieni

| Esigenza | Che cosa esiste | Leggi |
| --- | --- | --- |
| Struttura | Un'API JSON e un client per browser separato; PostgreSQL conserva i dati | [architecture.md](../../../../spec/architecture.md) |
| Identità | WPM non conserva identità. Una persona, un lavoratore e un'organizzazione sono riferimenti ad altri servizi, risolti solo per nome e conservati in cache | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Accesso | Attivo per impostazione predefinita. Accesso tramite la directory aziendale (OIDC, compatibile con Entra, direttamente o tramite Keycloak); ruoli e attributi decidono l'accesso | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Installazione | Una build del container e un file compose per una dimostrazione; per la produzione un proxy che termina il TLS, impostazioni obbligatorie e controlli all'avvio che rifiutano una configurazione insicura | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Cache condivisa | Redis facoltativo solo per ricerche non personali; ogni chiamata fallisce in modo aperto | `REDIS_URL` nella guida all'installazione (WPM-D78) |
| Backup e ripristino | Backup cifrati, uno script di ripristino e una prova di ripristino eseguita dall'integrazione continua | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Aggiornamento | Migrazioni e una guida all'aggiornamento | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Quando qualcosa va storto | Un manuale operativo con le prime azioni | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Movimento dei dati | Un documento sul flusso dei dati | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Da sapere prima di pianificare

- L'interfaccia **non è containerizzata**; scegli tu un adattatore per la tua piattaforma.
- I limiti di frequenza sono per istanza. Un archivio condiviso spetta a te fornirlo.
- L'applicazione usa il ruolo di database indicato da `DATABASE_URL`; un insieme di ruoli a privilegi minimi non è ancora fornito.
- La revoca dei token e la loro età massima dipendono dal tuo provider di identità.
- Un Redis gestito (cluster, sentinel, utenti con controllo degli accessi) non è stato provato.
- Non esiste alcuna misurazione del carico e alcune viste leggono un'intera organizzazione in memoria.
- La build dell'immagine e il file compose sono stati scritti ma vanno provati prima nel tuo ambiente.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
