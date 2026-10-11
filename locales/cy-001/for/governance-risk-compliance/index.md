# Llywodraethu, Risg, Cydymffurfio

**Y cwestiynau rydych chi'n debygol o'u gofyn:** pwy sy'n atebol am beth, beth allai fynd o'i le a beth sy'n ei atal, a pha dystiolaeth y gallaf ei dangos i aseswr? Mae'r dudalen hon yn casglu'r deunydd llywodraethu, risg a chydymffurfio mewn un lle. Mae tudalennau'r [Prif Swyddog Risg](../chief-risk-officer/index.md) a'r [Prif Swyddog Cyfreithiol](../chief-legal-officer/index.md) yn cwmpasu'r un tir yn ôl rôl.

## Llywodraethu: pwy sy'n penderfynu, a sut y caiff ei gofnodi

| Pwnc | Beth sy'n bodoli | Darllenwch |
| --- | --- | --- |
| Sut mae'r gwaith yn cael ei lywodraethu | Manyleb yn gyntaf: mae gofyniad, penderfyniad, tasg, cod a phrofion yn glanio gyda'i gilydd; mae pob tasg yn dweud beth **na** chafodd ei wirio | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Llywodraethu'r prosiect | Llywodraethu, cynhalwyr, proses RFC a ffeil perchnogion cod | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Penderfyniadau am bobl | Mae person yn penderfynu; mae sgorau'n cynghori. Cofnodir trosi, ymddiswyddo a cheisiadau gweithio hyblyg fel penderfyniad person | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Mynediad a rolau | Mae rolau a phriodoleddau'n penderfynu mynediad; mae tabl wedi'i ddosbarthu yn penderfynu'r angen i wybod ar bob llwybr darllen | [auth.md](../../../../spec/auth.md) |
| Awduraeth â chymorth AI | Datganiad ynghylch sut y defnyddiwyd AI | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Risg: beth allai fynd o'i le

| Pwnc | Beth sy'n bodoli | Darllenwch |
| --- | --- | --- |
| Dyfarniad | **Ddim yn barod** ar gyfer defnydd sydd angen sicrwydd uchel; daethpwyd o hyd i un rhwystr drwy brofi (nid oedd darllen yn seiliedig ar yr angen i wybod) a'i drwsio | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Bygythiadau | Model bygythiadau: asedau, actorion, ffiniau, y rheolaeth fel y'i hadeiladwyd, a'r hyn **na** chafodd ei asesu | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Tystiolaeth o ymyrryd | Llwybr archwilio atodi yn unig, wedi'i gadwyno â hash, gyda phwynt terfyn dilysu; nid yw cofnodion yn dal testun rheswm na gwerth | [audit.md](../../../../spec/audit.md) |
| Cychwyn sy'n methu'n ddiogel | Mae cynhyrchu yn gwrthod cychwyn gyda mewngofnodi i ffwrdd, dim allweddi tocyn, cysylltiad cronfa ddata testun plaen â gwesteiwr arall, neu Redis pell testun plaen | [production-readiness.md](../../../../spec/production-readiness.md) |
| Adfer | Copïau wrth gefn wedi'u hamgryptio, ymarfer adfer a redir gan CI, llyfr cyfrifon dileu fel nad yw adfer yn dod â pherson a ddilëwyd yn ôl | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Digwyddiadau | Llawlyfr gweithredol o'r camau cyntaf; mae'r defnyddiwr yn ychwanegu cysylltiadau a llwybrau uwchgyfeirio | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Rhagfarn mewn sgorio | Templed effaith cydraddoldeb a chofrestr o bob sgôr, pob un ag asesiad drafft | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Risgiau agored | Pinio a llofnodi'r gadwyn gyflenwi, tystiolaeth llwyth, gwiriadau logiau am ddata personol, terfynau cyfradd a rennir, set rolau cronfa ddata braint leiaf, canllawiau dirymu tocynnau | [readiness-assessment.md](../../../../spec/readiness-assessment.md), adran 3 |

## Cydymffurfio: yr hyn y gallwch ei ddangos i aseswr

| Pwnc | Beth sy'n bodoli | Darllenwch |
| --- | --- | --- |
| Hunanasesiad o reolaethau | Rhestr wirio lle mae pob ateb wedi'i fodloni, wedi'i fodloni'n rhannol, heb ei fodloni, neu i'r defnyddiwr ei ateb | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Asesiad effaith diogelu data | Templed, wedi'i gwblhau ar gyfer yr arddangosiad, gyda'r rhannau y mae'n rhaid i chi eu cwblhau wedi'u marcio | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Cofnod prosesu | Cofnod y gellir ei addasu | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Llif data | I ble mae data personol yn symud | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Cadw | Amserlen i bob math o gofnod | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Hawliau gwrthrych | Allforio mynediad gwrthrych a dileu | [regulatory.md](../../../../spec/regulatory.md) |
| Data categori arbennig | Mae monitro cydraddoldeb a gofynion iechyd yn y gweithle wedi'u diffodd nes cofnodi sail gyfreithlon, ac yna'n gyfyngedig | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Hygyrchedd ac iaith | Datganiad hygyrchedd a datganiad iaith swyddogol | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Trwyddedau a dibyniaethau | Ffeiliau trwydded, polisi dibyniaethau a wiriwyd gan `cargo deny`, a bil defnyddiau meddalwedd wrth ryddhau | [production-readiness.md](../../../../spec/production-readiness.md) |

## Yr hyn y gall pobl yn unig ei wneud

Gatiau yw'r rhain, nid cod, ac nid yw'r ystorfa yn eu marcio'n gyflawn: prawf treiddio annibynnol, archwiliad hygyrchedd annibynnol, eich asesiad effaith diogelu data wedi'i lofnodi gan eich swyddog, ymarfer adfer ar eich platfform, adolygiad proffesiynol o'r cyfieithiadau, a'ch cymeradwyaeth o'r asesiadau effaith cydraddoldeb.

## Cyfyngiadau'r dudalen hon

Mae'r tablau'n pwyntio at ddogfennau. Chi a'ch aseswr sydd i benderfynu a yw pob dogfen yn wir am eich defnydd chi. Nid yw profion yr awdur yn adolygiad annibynnol, ac nid yw dim yma yn gyngor cyfreithiol nac yn ardystiad.

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
