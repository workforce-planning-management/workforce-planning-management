# Ar gyfer y Prif Swyddog Strategaeth

**Y cwestiwn rydych chi'n debygol o'i ofyn:** a yw ein gweithlu wedi'i lunio i wasanaethu'r strategaeth, beth fydd ei angen arnom, a beth ddylem ni ei wneud am y bwlch?

## Beth gewch chi

| Rydych chi eisiau gwybod | Ble mae WPM yn ateb | Darllenwch |
| --- | --- | --- |
| Beth mae rôl yn ei ofyn | Proffiliau rôl gyda'r sgiliau gofynnol a lefel hyfedredd isaf, fel bod bwlch yn cael ei fesur yn erbyn rôl ac nid person | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Beth fydd ei angen arnom | Cynlluniau gweithlu fel bydoedd drafft, sawl un ochr yn ochr, gyda rhagdybiaethau a nodir a galw yn ôl adran, rôl a chyfnod | WPM-R36, yr un ffeil |
| Ble mae'r bwlch | Cyflenwad a ragwelir yn erbyn galw a gynlluniwyd: bwlch niferoedd a bwlch cymhwysedd, gyda liferi wedi'u rhestru fel awgrymiadau a'u tystiolaeth | WPM-R37 |
| A yw'n gwasanaethu'r strategaeth | Amcanion wedi'u cysylltu â'r galw: amcanion heb adnoddau, galw heb ei gyfeirio, rolau critigol heb fainc | WPM-R38 |
| Ydyn ni'n gallu cyflawni rhaglen | Pyllau sgiliau, galw'r rhaglen, ymrwymiadau partneriaid, y pwll cyfyngu, gwiriad dechrau sy'n awgrym | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Faint rydyn ni'n dibynnu ar staff nad ydynt yn barhaol | Y gyfran o FTE sy'n gontractwr a'r gyfran sy'n dymor penodol, ar wahân, a faint o ariannu sy'n dod i ben ym mhob un o'r deuddeg mis nesaf | `GET /api/workforce-intelligence/contingent` (WPM-T109, rhannol) |
| Beth rydyn ni'n bwriadu ei wneud pan fydd contractau'n dod i ben | Cynlluniau trosi: penderfyniad person i drosi, ymestyn neu ddod â therfyn, gyda'r ariannu wrth ymyl diwedd y contract | [engagements.md](../../../../spec/engagements.md) |
| Beth arall sydd wedi'i gynllunio | Y map ffordd a'r rhestr tasgau agored | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Sut i ddarllen rhagolwg

- Rheol a nodir wedi'i chymhwyso at fewnbynnau a nodir ydyw, nid model, ac mae pob canlyniad yn enwi ei ragdybiaethau.
- Gyda rhy ychydig o hanes nifer staff, mae'n dweud `insufficient_history` yn hytrach nag allosod.
- Dim ond nifer damcaniaethol y mae cynllun yn ei ddal. Nid yw byth yn darllen nac yn newid cofnod person.
- Awgrymiadau yw liferi. Y cynlluniwr sy'n dewis, ac nid yw'r feddalwedd yn penderfynu dim.

## Cyfyngiadau

- Mae'r lifer "trosi staff wrth gefn yn barhaol" mewn cynlluniau wedi'i fanylu (WPM-R63) ond heb fod yn y cod eto.
- Nid oes tuedd eto i'r gyfran wrth gefn: mae angen cipluniau wedi'u rhannu yn ôl sail (WPM-T139, heb eu hadeiladu).
- Mae amserlennu cadwyn gritigol a llwybr critigol yn perthyn i project-portfolio-management, nid yma.
- Nid oes rhyngwyneb defnyddiwr eto ar gyfer ariannu na chynlluniau trosi.

Gweler hefyd dudalennau'r [Prif Swyddog Gweithredol](../prif-swyddog-gweithredol/index.md) a'r [Prif Swyddog Gweithrediadau](../prif-swyddog-gweithrediadau/index.md).

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
