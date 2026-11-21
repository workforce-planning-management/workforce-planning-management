# Styring, risiko, compliance

**Perspektiv:** hvem er ansvarlig for hvad, hvad kan gå galt, og hvad forhindrer det, og
hvilke beviser kan jeg vise en vurderingsperson? Denne side samler materialet om styring, risiko og compliance ét sted.
Rollesiderne for risikodirektøren og den juridiske direktør dækker de
samme emner efter rolle.

## Styring: hvem beslutter, og hvordan det registreres

| Emne | Hvad der findes | Læs |
| --- | --- | --- |
| Hvordan arbejdet styres | Specifikation først: et krav, en beslutning, en opgave, kode og tests leveres sammen; hver opgave siger, hvad der **ikke** er verificeret | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Styring af projektet | Styring, vedligeholdere, en RFC-proces og en fil med kodeejere | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Beslutninger om mennesker | Et menneske beslutter; score rådgiver. Omlægninger, opsigelser og anmodninger om fleksibel arbejdstid registreres som et menneskes beslutning | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Adgang og roller | Roller og attributter afgør adgangen; en klassificeret tabel afgør behov for at vide på hver læseroute | [auth.md](../../../../spec/auth.md) |
| Forfatterskab med hjælp fra AI | En erklæring om, hvordan AI er brugt | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Risiko: hvad kan gå galt

| Emne | Hvad der findes | Læs |
| --- | --- | --- |
| Konklusion | **Ikke klar** til en driftssætning med høje krav til sikkerhed; én hindring (læsninger fulgte ikke behovet for at vide) fundet ved undersøgelse og rettet | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Trusler | En trusselsmodel: aktiver, aktører, grænser, kontrollerne som bygget og det, der **ikke er vurderet** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Tegn på manipulation | Et revisionsspor, der kun kan føjes til, kædet med hashværdier, med et slutpunkt til verificering; poster rummer ingen begrundelsestekst eller værdi | [audit.md](../../../../spec/audit.md) |
| Sikker opstart | Produktion nægter at starte med login slået fra, uden tokennøgler, med en databaseforbindelse uden kryptering til en anden vært eller med en ekstern Redis uden kryptering | [production-readiness.md](../../../../spec/production-readiness.md) |
| Genopretning | Krypterede sikkerhedskopier, en genopretningsøvelse, som CI kører, en sletteprotokol, så en genopretning ikke bringer en slettet person tilbage | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Hændelser | En driftsvejledning med de første handlinger; den driftsansvarlige tilføjer kontaktpersoner og eskaleringsveje | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Skævhed i scoring | En skabelon til konsekvensanalyse for ligestilling og et register over hver score, hver med en kladde til vurdering | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Åbne risici | Fastlåsning og signering af forsyningskæden, dokumentation for belastning, kontrol af logge for personoplysninger, fælles hastighedsgrænser, et sæt databaseroller med mindst mulige rettigheder, vejledning om tilbagekaldelse af tokens | [readiness-assessment.md](../../../../spec/readiness-assessment.md), afsnit 3 |

## Compliance: hvad du kan vise en vurderingsperson

| Emne | Hvad der findes | Læs |
| --- | --- | --- |
| Selvvurdering af kontroller | En tjekliste, hvor hvert svar er opfyldt, delvist opfyldt, ikke opfyldt eller skal besvares af den driftsansvarlige | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Konsekvensanalyse for databeskyttelse | En skabelon, udfyldt for demoen, med de dele, du selv skal udfylde, markeret | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Fortegnelse over behandling | En fortegnelse, der kan tilpasses | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Dataflow | Hvor personoplysninger bevæger sig | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Opbevaring | En plan for hver slags post | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Den registreredes rettigheder | Eksport ved indsigtsret og sletning | [regulatory.md](../../../../spec/regulatory.md) |
| Særlige kategorier af oplysninger | Overvågning af ligestilling og krav til sundhed på arbejdspladsen er slået fra, indtil et lovligt grundlag er registreret, og er derefter afgrænset | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Tilgængelighed og sprog | En tilgængelighedserklæring og en erklæring om officielle sprog | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Licenser og afhængigheder | Licensfiler, en politik for afhængigheder kontrolleret af `cargo deny` og en stykliste over software ved udgivelse | [production-readiness.md](../../../../spec/production-readiness.md) |

## Hvad kun mennesker kan gøre

Det er porte, ikke kode, og repositoriet markerer dem ikke som udført: en uafhængig penetrationstest, en
uafhængig tilgængelighedsrevision, din konsekvensanalyse for databeskyttelse godkendt af din ansvarlige, en genopretningsøvelse
på din platform, en professionel gennemgang af oversættelserne og din godkendelse af konsekvensanalyserne for ligestilling.

## Denne sides begrænsninger

Tabellerne peger på dokumenter. Om hvert dokument passer på din driftssætning, er op til dig og din vurderingsperson.
Forfatterens tests er ikke en uafhængig gennemgang, og intet her er juridisk rådgivning eller en certificering.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
