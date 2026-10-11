# Ar gyfer y Prif Swyddog Risg

**Y cwestiwn rydych chi'n debygol o'i ofyn:** beth allai fynd o'i le gyda'r system hon, beth sy'n ei atal, a beth nad oes neb wedi'i wirio?

## Y man cychwyn gonest

Mae'r [asesiad parodrwydd](../../../../spec/readiness-assessment.md) o 2026-10-11 yn rhoi'r dyfarniad **ddim yn barod** ar gyfer defnydd sydd angen sicrwydd uchel. Daeth o hyd i un rhwystr drwy brofi'r gwasanaeth byw: gallai unrhyw un oedd wedi mewngofnodi ddarllen absenoldeb salwch cydweithiwr a'i reswm testun rhydd. Mae hynny wedi'i drwsio a'i brofi. Mae'r gatiau y gall pobl yn unig eu cau yn dal ar agor.

## Beth gewch chi

| Pryder | Beth sy'n bodoli | Darllenwch |
| --- | --- | --- |
| Bygythiadau ar bob ffin | Model bygythiadau: asedau, actorion, ffiniau, y rheolaeth fel y'i hadeiladwyd, a'r hyn **na** chafodd ei asesu | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Darllen heb awdurdod | Gwarchodwr angen gwybod sy'n dosbarthu pob llwybr darllen; mae prawf yn methu ar un heb ei ddosbarthu | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Ymyrryd ac atebolrwydd | Llwybr archwilio atodi yn unig, wedi'i gadwyno â hash, gyda phwynt terfyn dilysu; nid yw cofnodion yn dal testun rheswm na gwerth | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Data sensitif | Dim diagnosis wedi'i storio; nodweddion categori arbennig wedi'u diffodd yn ddiofyn; trothwy grwpiau bach | [regulatory.md](../../../../spec/regulatory.md) |
| Llwybrau anniogel | Mae cynhyrchu yn gwrthod cychwyn gyda mewngofnodi i ffwrdd, dim ffynhonnell allwedd tocyn, cysylltiad cronfa ddata testun plaen â gwesteiwr arall, neu Redis pell testun plaen | [production-readiness.md](../../../../spec/production-readiness.md) |
| Colli data | Copïau wrth gefn wedi'u hamgryptio, ymarfer adfer a redir yn CI, llyfr cyfrifon dileu | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Rhagfarn mewn sgorio | Templed effaith cydraddoldeb, a chofrestr o bob sgôr gydag asesiad drafft | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Hunanasesiad o reolaethau | Rhestr wirio lle mae pob ateb wedi'i fodloni, wedi'i fodloni'n rhannol, heb ei fodloni, neu i'r defnyddiwr ei ateb | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Risgiau agored, fel y'u cofnodwyd

Mae pinio a llofnodi'r gadwyn gyflenwi, tystiolaeth llwyth, gwiriadau logiau am ddata personol, terfynau cyfradd a rennir ar draws enghreifftiau, set rolau cronfa ddata braint leiaf, a chanllawiau dirymu tocynnau yn eitemau agored yn yr asesiad parodrwydd, pob un â thasg. Ysgrifennwyd y profion gan yr awdur; nid oes yr un yn adolygiad annibynnol.

## Yr hyn y gall pobl yn unig ei wneud

Prawf treiddio annibynnol, archwiliad hygyrchedd annibynnol, asesiad effaith y defnyddiwr wedi'i lofnodi, ymarfer adfer ar eich platfform, a chymeradwyo asesiadau effaith cydraddoldeb. Mae'r ystorfa yn nodi'r rhain fel gatiau ac nid yw'n eu marcio'n gyflawn.

Gweler hefyd y [Prif Swyddog Gwybodaeth](../prif-swyddog-gwybodaeth/index.md) a'r [Prif Swyddog Cyfreithiol](../prif-swyddog-cyfreithiol/index.md).

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
