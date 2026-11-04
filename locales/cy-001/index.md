# Rheoli Cynllunio'r Gweithlu

**Platfform adnoddau dynol cod agored sy'n cwmpasu cylch bywyd y cyflogai, o'r penodi i'r ymddeol.**

Mae Rheoli Cynllunio'r Gweithlu (Workforce Planning Management, WPM) yn cyfuno recriwtio, amser ac absenoldeb, datblygu gweithwyr, cynllunio'r gweithlu a chyflogres mewn un rhaglen. Meddalwedd arddangos ydyw, at ddibenion gwerthuso ac fel dyluniad cyfeirio.

> **Meddalwedd arddangos.** Nid yw WPM yn system adnoddau dynol na chyflogres ar gyfer defnydd cynhyrchu. Mae'n defnyddio data synthetig yn unig, ac enghreifftiau yw ei gyfrifiadau statudol. Peidiwch â rhoi data personol go iawn ynddo. Gweler [y sefyllfa reoleiddiol](../../spec/regulatory.md) (Saesneg).

[Rhedeg yr arddangosiad](start/index.md) [Dogfennau rôl](for/index.md) [Cod ffynhonnell ar GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Manteision

Yn aml, cedwir data adnoddau dynol mewn offer ar wahân, fel mewnflychau ymgeiswyr, taenlenni absenoldeb a byrddau rota. Mae pob trosglwyddiad rhyngddynt yn ychwanegu gwaith llaw a risg o gamgymeriad. Mae WPM wedi'i ddylunio i leihau'r trosglwyddiadau hynny.

- **Un system.** Cedwir cofnodion cynefino, absenoldeb, hyfforddiant a chyflog mewn un rhaglen, sy'n lleihau ailgofnodi.
- **Lleihau data.** Nid oes maes ar gyfer data na ddylid ei gadw, nac API ar gyfer data na ddylid ei ddatgelu. Er enghraifft, nid oes maes diagnosis, ac nid oes awdur i ymatebion arolwg.
- **Rheolau adrodd.** Ni adroddir gwerthoedd anhysbys fel sero, nid yw cyfansymiau'n adnabod unigolion, ac mae amcangyfrifon yn nodi eu sail.
- **Lleoleiddio.** Mae'r rhaglen ar gael mewn 17 locale a'r dogfennau rôl mewn 21, gan gynnwys ieithoedd o'r dde i'r chwith. Cedwir rheolau gwlad-benodol fel data. Cynhyrchwyd y cyfieithiadau gan ddeallusrwydd artiffisial ac nid yw siaradwyr brodorol wedi'u hadolygu.
- **Olrheiniadwyedd.** Mae'r cod ffynhonnell yn agored, ac olrheinir pob ymddygiad i ofyniad, penderfyniad dylunio a thasg wedi'u rhifo.
- **Defnydd annibynnol.** Y pentwr yw API JSON Rust, cleient SvelteKit a PostgreSQL. Mae'n adeiladu o glôn newydd, ac mae integreiddio ag apiau eraill y teulu yn ddewisol.

## Nodweddion

- **Recriwtio a chofnodion.** Ceisiadau am swyddi, llif ymgeiswyr, cynefino, cofnodion gweithwyr a siart sefydliad.
- **Amser ac amserlennu.** Amser ac absenoldeb, shifftiau gyda gwiriadau amser gwaith, y rota ar alwad gyda chyfnewidiadau a nodiadau atgoffa, a chyfeiriadur staff.
- **Datblygiad.** Adolygiadau, adborth 360°, dysgu, dadansoddiad bylchau sgiliau, argymhellion amser hyfforddi a dysgu gorfodol.
- **Cynllunio ac adrodd.** Cynllunio strategol y gweithlu, metrigau a mewnwelediadau, a dangosfwrdd i'r Prif Swyddog Gweithredol.
- **Dechreuwyr ac ymadawyr.** Rhestrau gwirio wedi'u dyddio a throsglwyddiad diwrnod olaf gyda llwybr archwilio.
- **Cyfathrebu.** Cyhoeddiadau gyda chynulleidfaoedd, dolenni a chyfrifon darllen.
- **Hawliau a llesiant gweithwyr.** Negeseuon llesiant, arolwg dienw, addasiadau, mynediad gwrthrych, dileu a chadw.
- **Cyflog.** Cyflogres a thâl, gyda chyfrifiadau statudol enghreifftiol.

## Cymhariaeth

Mae'r tabl yn cymharu WPM â thri chategori o system adnoddau dynol. Canllaw cyffredinol ydyw, nid meincnod. Gweler hefyd y [gymhariaeth fesul cynnyrch](../../docs/compare/index.md) (Saesneg).

| | WPM | Systemau popeth-mewn-un (Workday, SAP SuccessFactors, BambooHR) | Adnoddau dynol cod agored (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Offer penodol (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Cwmpas** | Y cylch bywyd cyfan, o'r penodi i'r ymddeol | Cwmpas tebyg ar un platfform | Yr un swyddogaethau craidd | Un swyddogaeth |
| **Trwyddedu** | Cod agored | Masnachol | Cod agored | Masnachol |
| **Addas ar gyfer defnydd cynhyrchu** | Na | Ie | Ie | Ie |
| **Prif gryfder** | Lleihau data, dylunio olrheiniadwy, dadansoddi bylchau sgiliau ac amser hyfforddi | Un platfform integredig â chymorth | Mwy o integreiddiadau, lleoleiddio a defnydd | Dyfnder mewn un swyddogaeth |
| **Addas ar gyfer** | Gwerthuso, dyluniad cyfeirio neu ddatblygu pellach | Sefydliadau sydd angen system â chymorth | Sefydliadau sydd am ddefnyddio cod agored mewn cynhyrchiad | Sefydliadau sydd angen dyfnder mewn un swyddogaeth ac sy'n gallu integreiddio'r gweddill |

## Cynulleidfa

- **Swyddogion gweithredol.** Mae'r [dogfennau rôl](for/index.md) yn nodi beth mae WPM yn ei ddarparu i bob rôl arwain a sut i ddarllen ei allbwn.
- **Timau adnoddau dynol.** Mae'r [tiwtorial arddangos](start/index.md) yn cwmpasu pob tudalen gan ddefnyddio data ffug.
- **Datblygwyr a phenseiri.** Gweler y [fanyleb](../../spec/index.md) a'r [canllaw gosod](../../INSTALL.md) (Saesneg).
- **Aseswyr.** Mae'r [asesiad parodrwydd](../../spec/readiness-assessment.md) (Saesneg) yn rhestru'r bylchau rhwng WPM a defnydd cynhyrchu.

## Camau nesaf

1. **Darllenwch y ddogfen ar gyfer eich rôl.** Mae pob dogfen yn disgrifio beth mae WPM yn ei ddarparu a beth sydd heb ei gwblhau.
2. **Rhedwch yr arddangosiad.** Mae'r daith yn rhedeg ar beiriant lleol gyda data ffug.
3. **Anfonwch adborth.** Gweler [sut i wneud sylw](../../RFC.md) a [sut i gyfrannu](../../CONTRIBUTING.md) (Saesneg).

[Rhedeg yr arddangosiad](start/index.md) [Dogfennaeth](../../index.md)
