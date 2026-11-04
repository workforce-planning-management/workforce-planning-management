# Rheoli Cynllunio'r Gweithlu

**Popeth am eich pobl, o'r penodi i'r ymddeol, mewn un lle.**

Croeso! Mae Rheoli Cynllunio'r Gweithlu (Workforce Planning Management, WPM) yn blatfform adnoddau dynol cod agored sy'n dod â recriwtio, amser ac absenoldeb, datblygiad, cynllunio a chyflog at ei gilydd, fel nad oes dim yn mynd ar goll rhwng mewnflychau a thaenlenni. Mae'n arddangosiad gweithredol y gallwch ei redeg, ei ddarllen a dysgu ohono.

> ⚠️ **Meddalwedd arddangos.** Nid system adnoddau dynol na chyflogres ar gyfer defnydd go iawn yw WPM: mae'n defnyddio data synthetig yn unig, ac enghreifftiau yn unig yw ei gyfrifiadau statudol. Peidiwch â rhoi data personol go iawn ynddo. Gweler [y sefyllfa reoleiddiol](../../spec/regulatory.md) (yn Saesneg).

[Dilynwch y tiwtorial](start/index.md) [Gweld beth mae'n ei olygu i'ch rôl chi](for/index.md) [Gweld y cod ar GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Pam WPM

Ymgeiswyr mewn mewnflychau, absenoldeb mewn taenlenni, rotâu ar fyrddau gwyn: mae pob trosglwyddiad rhwng y seilos hynny'n colli rhywbeth. Mae WPM wedi'i adeiladu ar syniad gwahanol.

- **Un system, nid chwech.** Mae holl gylch bywyd y cyflogai yn byw gyda'i gilydd, felly nid oes angen aildeipio cynefino, absenoldeb, hyfforddiant a chyflog aelod newydd o staff.
- **Preifatrwydd o'r cychwyn.** Nid yw'r hyn na ddylid ei storio yn cael colofn, ac nid yw'r hyn na ddylid ei ddatgelu yn cael pwynt terfyn. Nid oes colofn diagnosis, na neb wedi'i enwi wrth ateb arolwg dienw.
- **Rhifau y gallwch ymddiried ynddynt.** Ni ddangosir anhysbys byth fel sero, nid yw crynodebau'n enwi neb, ac mae pob amcangyfrif yn dweud ar beth mae'n seiliedig.
- **Wedi'i wneud ar gyfer y byd.** Mae'r ap yn siarad 17 locale a'r dogfennau rôl hyn 21, gan gynnwys ieithoedd o'r dde i'r chwith, ac mae rheolau sy'n wahanol rhwng gwledydd yn ddata, nid cod. Ysgrifennwyd y cyfieithiadau gan ddeallusrwydd artiffisial ac nid yw siaradwyr brodorol wedi'u hadolygu eto.
- **Agored ac olrheiniadwy.** Mae'n god agored, ac mae pob ymddygiad yn olrhain yn ôl i ofyniad, penderfyniad dylunio a thasg wedi'u rhifo.
- **Yn gweithio ar ei ben ei hun.** API JSON Rust, cleient SvelteKit a PostgreSQL. Mae'n adeiladu ac yn rhedeg o glôn newydd, ac mae cysylltiadau ag apiau eraill y teulu yn ddewisol.

## Beth mae'n ei wneud

- 🤝 **Recriwtio a'r cofnod.** Ceisiadau am swyddi, y llif ymgeiswyr, cynefino, cofnodion gweithwyr a'r siart sefydliad.
- 🗓️ **Gwaith bob dydd.** Amser ac absenoldeb, shifftiau gyda chanllawiau amser gwaith, y rota ar alwad gyda chyfnewidiadau a nodiadau atgoffa, a'r cyfeiriadur.
- 🌱 **Datblygiad.** Adolygiadau, adborth 360°, dysgu, dadansoddiad bylchau sgiliau, argymhellion amser hyfforddi a dysgu gorfodol.
- 📊 **Arweinyddiaeth.** Cynllunio strategol y gweithlu, metrigau a mewnwelediadau, a dangosfwrdd i'r Prif Swyddog Gweithredol sy'n ffitio ar un sgrin iPad.
- 👋 **Dechreuwyr ac ymadawyr.** Rhestrau gwirio wedi'u dyddio, a throsglwyddiad diwrnod olaf gyda llwybr archwilio.
- 📣 **Cyfathrebu.** Cyhoeddiadau gyda chynulleidfaoedd, dolenni a chyfrifon darllen.
- 💚 **Hawliau a llesiant.** Negeseuon llesiant, yr arolwg dienw, addasiadau, mynediad gwrthrych, dileu a chadw.
- 💳 **Cyflog.** Cyflogres a thâl, gyda chyfrifiadau statudol enghreifftiol.

## Sut mae'n cymharu

Canllaw yw hwn, nid sgorfwrdd: mae pob math o system isod yn ateb cywir i rywun. [Cymharwch WPM â phob cynnyrch](../../docs/compare/index.md) (yn Saesneg).

| | WPM | Systemau popeth-mewn-un (Workday, SAP SuccessFactors, BambooHR) | Adnoddau dynol cod agored (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Offer penodol (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Cwmpas** | Y cylch bywyd cyfan, o'r penodi i'r ymddeol | Tir tebyg, ar un platfform | Yr un craidd | Un maes, wedi'i wneud yn dda |
| **Ffynhonnell** | Cod agored | Masnachol | Cod agored | Masnachol |
| **Ar gyfer gwaith adnoddau dynol go iawn heddiw?** | Na: arddangosiad | Ie | Ie | Ie |
| **Yn rhagori ar** | Preifatrwydd o'r cychwyn, dylunio olrheiniadwy, bylchau sgiliau ac amser hyfforddi | Un platfform integredig â chymorth | Llawer mwy o integreiddiadau, lleoleiddio a defnydd | Dyfnder mewn un dasg |
| **Dewiswch pan** | Rydych am werthuso, dysgu o ddyluniad agored neu adeiladu arno | Mae angen system â chymorth arnoch nawr | Rydych am ddefnyddio cod agored mewn cynhyrchiad nawr | Mae un dasg angen dyfnder a gallwch gysylltu'r gweddill |

## Ar gyfer pwy mae

- **Prif swyddogion.** Mae un dudalen ar gyfer pob rôl arwain yn dweud beth mae WPM yn ei roi i chi a sut i'w ddarllen: [y dogfennau rôl](for/index.md).
- **Timau adnoddau dynol a phobl.** Gwelwch sut mae'r darnau'n ffitio, dudalen wrth dudalen, yn [y daith dywys](start/index.md).
- **Datblygwyr a phenseiri.** Dechreuwch gyda [y fanyleb](../../spec/index.md) a [y canllaw gosod](../../INSTALL.md) (yn Saesneg).
- **Aseswyr.** Mae [yr asesiad parodrwydd](../../spec/readiness-assessment.md) (yn Saesneg) yn nodi'n blaen beth sy'n sefyll rhwng WPM a chynhyrchiad.

## Dechrau mewn tri cham

1. **Darllenwch eich dogfen.** Dewiswch eich rôl a gweld beth mae WPM yn ei gynnig iddi.
2. **Ewch ar y daith.** Rhedwch yr arddangosiad ar eich peiriant eich hun, gyda data ffug.
3. **Ymunwch.** Dywedwch wrthym beth i'w gymharu neu ei wella: [sut i wneud sylw](../../RFC.md) a [sut i gyfrannu](../../CONTRIBUTING.md) (yn Saesneg).

[Dechrau'r tiwtorial](start/index.md) [Pori'r ddogfennaeth](../../index.md)
