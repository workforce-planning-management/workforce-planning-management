# Til risikodirektøren

**Perspektiv:** hvad kan gå galt med dette system, hvad forhindrer det, og hvad har ingen
kontrolleret?

## Det ærlige udgangspunkt

[Parathedsvurderingen](../../../../spec/readiness-assessment.md) fra 2026-10-11 giver konklusionen **ikke klar** til en
driftssætning med høje krav til sikkerhed. Den fandt én hindring ved at undersøge den kørende tjeneste: enhver, der var logget ind, kunne læse en
kollegas sygefravær og dets begrundelse i fri tekst. Det er rettet og testet. De porte, som kun mennesker kan lukke, er
stadig åbne.

## Hvad du får

| Emne | Hvad der findes | Læs |
| --- | --- | --- |
| Trusler ved hver grænse | En trusselsmodel: aktiver, aktører, grænser, kontrollerne som bygget og det, der **ikke er vurderet** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Uautoriseret læsning | En kontrol af behov for at vide, der klassificerer hver læseroute; en test fejler ved en uklassificeret | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Manipulation og ansvarlighed | Et revisionsspor, der kun kan føjes til, kædet med hashværdier, med et slutpunkt til verificering; poster rummer ingen begrundelsestekst eller værdi | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Følsomme data | Ingen diagnose gemmes; funktioner til særlige kategorier af oplysninger er slået fra som standard; en nedre grænse for små grupper | [regulatory.md](../../../../spec/regulatory.md) |
| Usikre veje | Produktion nægter at starte med login slået fra, uden kilde til tokennøgler, med en databaseforbindelse uden kryptering til en anden vært eller med en ekstern Redis uden kryptering | [production-readiness.md](../../../../spec/production-readiness.md) |
| Tab af data | Krypterede sikkerhedskopier, en genopretningsøvelse kørt i CI, en sletteprotokol | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Skævhed i scoring | En skabelon til konsekvensanalyse for ligestilling og et register over hver score med en kladde til vurdering | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| En selvvurdering af kontroller | En tjekliste, hvor hvert svar er opfyldt, delvist opfyldt, ikke opfyldt eller skal besvares af den driftsansvarlige | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Åbne risici, som registreret

Fastlåsning og signering af forsyningskæden, dokumentation for belastning, kontrol af logge for personoplysninger, fælles hastighedsgrænser på tværs af
instanser, et sæt databaseroller med mindst mulige rettigheder og vejledning om tilbagekaldelse af tokens er åbne punkter i parathedsvurderingen,
hver med en opgave. Testene er skrevet af forfatteren; ingen af dem er en uafhængig gennemgang.

## Hvad kun mennesker kan gøre

En uafhængig penetrationstest, en uafhængig tilgængelighedsrevision, den driftsansvarliges godkendte konsekvensanalyse,
en genopretningsøvelse på din platform og godkendelse af konsekvensanalyser for ligestilling. Repositoriet angiver dem
som porte og markerer dem ikke som udført.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/for/`), er det den engelske version, der gælder.
