# Ar gyfer y Prif Swyddog Gweithrediadau

**Y cwestiwn rydych chi'n debygol o'i ofyn:** oes gennym ni'r bobl iawn, yn y lle iawn, ar yr adeg iawn, a beth sy'n torri os yw rhywun yn absennol?

## Beth gewch chi

| Rydych chi eisiau gwybod | Ble mae WPM yn ateb | Darllenwch |
| --- | --- | --- |
| A all tîm gwmpasu absenoldeb | Copïau wrth gefn i bob person, y rota ar alwad gyda chyfnewid a nodiadau atgoffa, a golwg cwmpasu sy'n dweud yn blaen pan nad oes neb ar gael | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Amser, presenoldeb a chyfnodau gwyliau | Cofnodion amser, ceisiadau am wyliau gyda llwybr cymeradwyo, cronni, a shifftiau | [workforce-management.md](../../../../spec/workforce-management.md) |
| A yw terfynau oriau gwaith yn cael eu parchu | Gwarchodlenni oriau gwaith cynghorol; maen nhw'n rhybuddio ac nid ydynt yn rhwystro | WPM-R27, yr un ffeil |
| Capasiti yn ôl sgil, fis wrth fis | Pyllau sgiliau, cyflenwad a galw'r rhaglen, ymrwymiadau partneriaid, y pwll cyfyngu, a gwiriad dechrau sy'n awgrym, gyda phenderfyniadau dechrau wedi'u cofnodi | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Beth fydd ei angen arnom a ble mae'r bwlch | Cynlluniau gweithlu, rhagolwg a dadansoddi bylchau yn ôl adran ac yn ôl rôl | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Pa gontractau sy'n dod i ben yn fuan | Dyddiad gorffen yn ôl sail yr ymrwymiad, estyniadau gyda hanes, nodiadau atgoffa cyn diwedd, rhestr o ymrwymiadau heb ddyddiad gorffen, a chynlluniau trosi | [engagements.md](../../../../spec/engagements.md) |
| Ble mae staff wrth gefn yn gyfran fawr | Y gyfran o FTE sy'n gontractwr a'r gyfran sy'n dymor penodol, ar wahân, a'r ariannu sy'n dod i ben fis wrth fis | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Newydd-ddyfodiaid a'r rhai sy'n gadael | Rhestrau gwirio â dyddiadau, a throsglwyddo ar y diwrnod olaf | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Bylchau sgiliau ac amser hyfforddi | Dadansoddi bylchau gyda'r dystiolaeth y mae argymhelliad yn seiliedig arni | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Sut i ddarllen y golwg capasiti

Cynllunnir capasiti yn ôl pwll sgiliau, nid yn ôl person. Mae pob mis-pwll yn arwain at un o bedwar canlyniad, ac mae sgil heb ei ddatgan yn *anhysbys*, nid yn brinder. Awgrym yw'r gwiriad dechrau, a pherson sy'n cofnodi'r penderfyniad i ddechrau. Nid yw amserlennu cadwyn gritigol a llwybr critigol yma: maen nhw'n perthyn i project-portfolio-management.

## Cyfyngiadau

- Nid oes tuedd eto i'r gyfran wrth gefn: mae angen ciplun nifer staff wedi'u rhannu yn ôl sail (WPM-T139, heb ei adeiladu).
- Yr adran yw "pwll" yn y golwg wrth gefn nes bod cynlluniau'n diffinio pyllau.
- Nid yw amserlennu shifftiau a chyfrifiadau treth cyflogres yn cymryd lle eich systemau rotâu neu gyflogres.
- Mae sawl golwg yn llwytho cofnodion sefydliad i'r cof; ni fesurwyd unrhyw ffigur llwyth ([readiness-assessment.md](../../../../spec/readiness-assessment.md), bwlch 8).

Gweler hefyd y [Prif Swyddog Pobl](../prif-swyddog-pobl/index.md) a'r [Prif Swyddog Gweithredol](../prif-swyddog-gweithredol/index.md).

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
