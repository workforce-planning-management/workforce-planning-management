# Per il direttore degli affari legali

**Prospettiva:** quali obblighi legali crea o aiuta a gestire questo software e che cosa dobbiamo
decidere prima che vi entrino dati reali?

> Nulla di quanto segue è consulenza legale. La specifica enuncia un orientamento, non una conclusione su alcuna giurisdizione.

## Che cosa ottieni

| Tema | Che cosa esiste | Leggi |
| --- | --- | --- |
| Orientamento | L'orientamento sul GDPR del Regno Unito e sugli archivi del personale; lo stato di demo; che cosa servirebbe in più per la produzione | [regulatory.md](../../../../spec/regulatory.md) |
| Valutazione d'impatto | Un modello di valutazione d'impatto sulla protezione dei dati, compilato per la demo, con le parti che devi completare contrassegnate | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Registro dei trattamenti e flusso dei dati | Documenti adattabili | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md), [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Conservazione | Un calendario per tipo di archivio, con una pulizia per alcuni | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Diritti dell'interessato | Esportazione per l'accesso ai propri dati, cancellazione e un registro delle cancellazioni | [regulatory.md](../../../../spec/regulatory.md) (WPM-R30) |
| Dati di categoria particolare | Il monitoraggio della parità e i requisiti di salute sul lavoro sono disattivati finché non registri una base giuridica, poi circoscritti: volontari, solo per sé o limitati a un ruolo, aggregati sopra una soglia, mai usati in una decisione | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Diritti previsti dalla legge | Richieste di lavoro flessibile con un termine di decisione e motivazioni che configuri tu; preavviso di dimissioni che configuri tu | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Obblighi di parità | Un modello di valutazione d'impatto sulla parità e un registro di ogni punteggio | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| Accessibilità e lingua | Una dichiarazione di accessibilità e una dichiarazione sulla lingua ufficiale | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Organismi collettivi | Ricerca e proposta per organismi riconosciuti e consultazione, mai l'iscrizione individuale | [worker-unions.md](../../../../spec/worker-unions.md) |
| Collaboratori esterni | Sono registrati il fondamento dell'incarico, il fornitore, il canale e la tariffa oscurata. Se un incarico sia lecito o correttamente inquadrato non è qualcosa che il software decide | [engagements.md](../../../../spec/engagements.md) |
| Licenze | File di licenza, una politica sulle dipendenze e una distinta dei componenti software a ogni versione | [production-readiness.md](../../../../spec/production-readiness.md) |

## Valori predefiniti che non sono legge

Diverse impostazioni sono valori predefiniti, non consulenza legale: il preavviso di dimissioni, il termine di decisione sul lavoro flessibile e
il termine di ricorso, e i motivi di rifiuto. La guida all'installazione dice di **confermarli** per la tua giurisdizione.
I calcoli previsti dalla legge, come l'imposta sulle paghe, non sono implementati.

## Decisioni che spettano a te

La base giuridica per ogni finalità di trattamento, la valutazione d'impatto sulla protezione dei dati e la sua approvazione, gli accordi con i responsabili
del trattamento, il calendario di conservazione che adotti e se attivare una qualsiasi funzione per dati di categoria particolare.

> Questa traduzione è stata prodotta da un'IA e non è stata rivista da un madrelingua italiano. In caso di differenze con la versione inglese (`locales/en-001/for/`), prevale la versione inglese.
