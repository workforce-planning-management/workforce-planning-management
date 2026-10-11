# Ar gyfer y Prif Swyddog Gwybodaeth

**Y cwestiwn rydych chi'n debygol o'i ofyn:** sut mae hyn yn ffitio i'n hystad, pwy all fewngofnodi, ble mae'r data'n byw, a sut ydyn ni'n ei redeg a'i adfer?

## Beth gewch chi

| Pryder | Beth sy'n bodoli | Darllenwch |
| --- | --- | --- |
| Siâp | API JSON a chleient porwr ar wahân; PostgreSQL sy'n dal y data | [architecture.md](../../../../spec/architecture.md) |
| Hunaniaeth | Nid yw WPM yn dal hunaniaethau. Cyfeiriadau at wasanaethau eraill yw person, gweithiwr a sefydliad, wedi'u datrys yn ôl enw yn unig a'u storio dros dro | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Mewngofnodi | Ymlaen yn ddiofyn. Mewngofnodi cyfeiriadur menter (OIDC, cydnaws ag Entra, yn uniongyrchol neu drwy Keycloak); rolau a phriodoleddau sy'n penderfynu mynediad | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Defnyddio | Adeiladu cynhwysydd a ffeil compose ar gyfer arddangosiad; ar gyfer cynhyrchu, dirprwy sy'n terfynu TLS, gosodiadau gofynnol, a gwiriadau cychwyn sy'n gwrthod gosodiad anniogel | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Storfa dros dro a rennir | Redis dewisol ar gyfer chwiliadau nad ydynt yn bersonol yn unig; mae pob galwad yn methu'n agored | `REDIS_URL` yn y canllaw defnyddio (WPM-D78) |
| Copi wrth gefn ac adfer | Copïau wrth gefn wedi'u hamgryptio, sgript adfer, ac ymarfer adfer y mae CI yn ei redeg | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Uwchraddio | Mudo a chanllaw uwchraddio | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Pan fydd rhywbeth yn mynd o'i le | Llawlyfr gweithredol o'r camau cyntaf | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Symud data | Dogfen llif data | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Pethau i'w gwybod cyn cynllunio

- Nid yw'r rhyngwyneb defnyddiwr **wedi'i roi mewn cynhwysydd**; rydych chi'n dewis addasydd ar gyfer eich platfform.
- Mae terfynau cyfradd fesul enghraifft. Chi sydd i ddarparu storfa a rennir.
- Mae'r rhaglen yn defnyddio pa bynnag rôl cronfa ddata y mae `DATABASE_URL` yn ei henwi; nid oes set rolau braint leiaf wedi'i darparu eto.
- Cyfrifoldeb eich darparwr hunaniaeth chi yw dirymu tocynnau ac uchafswm oedran tocyn.
- Nid yw Redis a reolir (clwstwr, sentinel, defnyddwyr rheoli mynediad) wedi'i roi ar brawf.
- Nid oes mesur llwyth, ac mae rhai golygon yn darllen sefydliad i'r cof.
- Ysgrifennwyd yr adeiladu delwedd a'r ffeil compose ond dylid eu rhoi ar brawf yn gyntaf yn eich amgylchedd chi.

Gweler [readiness-assessment.md](../../../../spec/readiness-assessment.md) am y rhestr lawn o fylchau a'r dyfarniad, a thudalen y [Prif Swyddog Technoleg](../prif-swyddog-technoleg/index.md) am y golwg peirianneg.

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
