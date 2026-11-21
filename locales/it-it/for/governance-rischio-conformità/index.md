# Governance, rischio, conformità

**Prospettiva:** chi risponde di che cosa, che cosa potrebbe andare storto e che cosa lo impedisce, e
quali prove posso mostrare a un valutatore? Questa pagina riunisce in un unico luogo il materiale su governance, rischio e conformità.
Le pagine dei ruoli di direttore dei rischi e di direttore degli affari legali trattano
lo stesso terreno per ruolo.

## Governance: chi decide e come viene registrato

| Tema | Che cosa esiste | Leggi |
| --- | --- | --- |
| Come è governato il lavoro | Prima la specifica: un requisito, una decisione, un'attività, codice e test arrivano insieme; ogni attività dice che cosa **non** è stato verificato | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Governance del progetto | Governance, responsabili della manutenzione, un processo RFC e un file dei responsabili del codice | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Decisioni sulle persone | Decide una persona; i punteggi consigliano. Conversioni, dimissioni e richieste di lavoro flessibile sono registrate come decisione di una persona | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Accesso e ruoli | Ruoli e attributi decidono l'accesso; una tabella classificata decide la necessità di sapere su ogni rotta di lettura | [auth.md](../../../../spec/auth.md) |
| Paternità assistita dall'IA | Una dichiarazione su come è stata usata l'IA | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Rischio: che cosa potrebbe andare storto

| Tema | Che cosa esiste | Leggi |
| --- | --- | --- |
| Verdetto | **Non pronto** per un'installazione ad alta garanzia; un ostacolo (le letture non erano basate sulla necessità di sapere) individuato con un sondaggio e corretto | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Minacce | Un modello delle minacce: risorse, attori, confini, il controllo come realizzato e ciò che **non è valutato** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Evidenza di manomissione | Una traccia di audit di sola aggiunta, collegata da hash, con un endpoint di verifica; le voci non contengono testo di motivazione né valori | [audit.md](../../../../spec/audit.md) |
| Avvio a prova di guasto | La produzione rifiuta di avviarsi con l'accesso disattivato, senza chiavi dei token, con una connessione al database in chiaro verso un altro host o con un Redis remoto in chiaro | [production-readiness.md](../../../../spec/production-readiness.md) |
| Ripristino | Backup cifrati, una prova di ripristino eseguita dall'integrazione continua, un registro delle cancellazioni affinché un ripristino non riporti indietro una persona cancellata | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Incidenti | Un manuale operativo con le prime azioni; chi installa aggiunge contatti e percorsi di escalation | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Distorsioni nei punteggi | Un modello di valutazione d'impatto sulla parità e un registro di ogni punteggio, ciascuno con una valutazione in bozza | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Rischi aperti | Fissazione e firma della catena di fornitura, prove di carico, controlli dei log per cercarvi dati personali, limiti di frequenza condivisi, un insieme di ruoli di database a privilegi minimi, indicazioni sulla revoca dei token | [readiness-assessment.md](../../../../spec/readiness-assessment.md), sezione 3 |

## Conformità: che cosa puoi mostrare a un valutatore

| Tema | Che cosa esiste | Leggi |
| --- | --- | --- |
| Autovalutazione dei controlli | Una lista di controllo in cui ogni risposta è soddisfatto, soddisfatto in parte, non soddisfatto o a carico di chi installa | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Valutazione d'impatto sulla protezione dei dati | Un modello, compilato per la demo, con le parti che devi completare contrassegnate | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Registro dei trattamenti | Un registro adattabile | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Flusso dei dati | Dove si muovono i dati personali | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Conservazione | Un calendario per tipo di archivio | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Diritti dell'interessato | Esportazione per l'accesso ai propri dati e cancellazione | [regulatory.md](../../../../spec/regulatory.md) |
| Dati di categoria particolare | Il monitoraggio della parità e i requisiti di salute sul lavoro sono disattivati finché non è registrata una base giuridica, poi circoscritti | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Accessibilità e lingua | Una dichiarazione di accessibilità e una dichiarazione sulla lingua ufficiale | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Licenze e dipendenze | File di licenza, una politica sulle dipendenze verificata da `cargo deny` e una distinta dei componenti software a ogni versione | [production-readiness.md](../../../../spec/production-readiness.md) |

## Che cosa possono fare solo le persone

Queste sono porte, non codice, e il repository non le segna come completate: un test di intrusione indipendente, un
audit di accessibilità indipendente, la tua valutazione d'impatto sulla protezione dei dati approvata dal tuo responsabile, una prova di ripristino
sulla tua piattaforma, una revisione professionale delle traduzioni e la tua approvazione delle valutazioni d'impatto sulla parità.

## Limiti di questa pagina

Le tabelle rimandano a documenti. Se ciascun documento corrisponda al vero per la tua installazione è affare tuo e del tuo valutatore.
I test dell'autore non sono una revisione indipendente e nulla qui è consulenza legale o certificazione.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
