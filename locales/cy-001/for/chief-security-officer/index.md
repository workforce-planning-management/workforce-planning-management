# Ar gyfer y Prif Swyddog Diogelwch

**Y cwestiwn rydych chi'n debygol o'i ofyn:** sut mae hwn wedi'i ddiogelu, sut fyddem yn gwybod pe bai'n cael ei ymosod arno neu ei gamddefnyddio, a beth nad oes neb wedi'i brofi?

## Beth gewch chi

| Pryder | Beth sy'n bodoli | Darllenwch |
| --- | --- | --- |
| Bygythiadau | Model bygythiadau: asedau, actorion, ffiniau ymddiriedaeth, y rheolaeth fel y'i hadeiladwyd, a'r hyn **na** chafodd ei asesu | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Dilysu | Mewngofnodi ymlaen yn ddiofyn; mae cynhyrchu yn gwrthod cychwyn gydag ef i ffwrdd neu heb ffynhonnell allwedd tocyn; mewngofnodi cyfeiriadur menter (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Awdurdodi | Mae'r polisi'n penderfynu yn ôl gweithred yn unig; mae tabl wedi'i ddosbarthu yn penderfynu'r angen i wybod ar bob llwybr darllen, ac mae prawf sgan ffynhonnell yn methu ar lwybr heb ei ddosbarthu | [auth.md](../../../../spec/auth.md) |
| Archwilio | Llwybr atodi yn unig, wedi'i gadwyno â hash, gyda phwynt terfyn dilysu; nid yw cofnodion yn cario testun rheswm ac nid gwerth | [audit.md](../../../../spec/audit.md) |
| Trafnidiaeth a phorwr | Penawdau diogelwch ar bob ymateb API, rhestr ganiatáu CORS eglur sy'n atal cychwyn pan nad yw wedi'i gosod, `no-store` ar `/api` | WPM-R73 yn [production-readiness.md](../../../../spec/production-readiness.md) |
| Camddefnydd | Terfynau cyfradd fesul galwr, yn llymach ar gyfer dileu, mewnforio, allforio a llwybrau tocynnau | WPM-R74, yr un ffeil |
| Ffurfweddiad anniogel | Mae cychwyn yn gwrthod cronfa ddata testun plaen neu Redis testun plaen i westeiwr arall, a tharddiad CORS sydd ar goll | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Data wrth orffwys | Copïau wrth gefn wedi'u hamgryptio; mae'r storfa dros dro a rennir yn dal chwiliadau nad ydynt yn bersonol yn unig ac yn methu'n agored | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Cadwyn gyflenwi | Gwiriadau trwyddedau a gwaharddiadau gyda `cargo deny`, crates a rennir wedi'u cynnwys, bil defnyddiau meddalwedd wrth ryddhau | [production-readiness.md](../../../../spec/production-readiness.md) |
| Digwyddiadau | Llawlyfr gweithredol o'r camau cyntaf; rydych chi'n ychwanegu cysylltiadau a dyletswyddau hysbysu | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Hunanasesiad | Rhestr wirio rheolaethau: wedi'i bodloni, wedi'i bodloni'n rhannol, heb ei bodloni, i'r defnyddiwr ei hateb | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Bylchau hysbys, fel y'u cofnodwyd

- Ni wnaed prawf treiddio annibynnol. Ysgrifennwyd y profion gan yr awdur.
- Mae Actions wedi'u pinio gan dag, nid gan grynodeb, ac nid yw delweddau wedi'u llofnodi.
- Mae terfynau cyfradd fesul enghraifft.
- Mae'r rhaglen yn defnyddio pa bynnag rôl cronfa ddata a roddir iddi; nid oes set rolau braint leiaf wedi'i darparu.
- Cyfrifoldeb eich darparwr hunaniaeth chi yw dirymu tocynnau ac uchafswm oedran tocyn.
- Mae llwybr y rheolwr cynnwys yn llwytho sgript o CDN cyhoeddus heb fersiwn wedi'i phinio na hash cywirdeb.
- Ni wiriwyd y logiau am ddata personol, ac nid yw'r llwybr archwilio yn cael ei allforio i system fonitro.
- Nid yw Redis a reolir (clwstwr, sentinel, defnyddwyr rheoli mynediad) wedi'i roi ar brawf.

## Yr hyn y gall pobl yn unig ei wneud

Comisiynu'r prawf treiddio annibynnol, adolygu'r model bygythiadau ar gyfer eich amgylchedd, rhedeg ymarfer adfer ar eich platfform, a gosod diogelu canghennau ar y platfform cynnal. Gweler yr [asesiad parodrwydd](../../../../spec/readiness-assessment.md), a'i ddyfarniad yw **ddim yn barod** ar gyfer defnydd sydd angen sicrwydd uchel.

Gweler hefyd dudalennau'r [Prif Swyddog Risg](../chief-risk-officer/index.md) a'r [Prif Swyddog Gwybodaeth](../chief-information-officer/index.md).

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
