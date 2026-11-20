# Ar gyfer y Prif Swyddog Pobl

**Y cwestiwn rydych chi'n debygol o'i ofyn:** a yw hwn yn cefnogi cylch bywyd cyfan y gweithiwr, ac a allaf ei ddefnyddio heb gadw mwy am bobl nag y dylwn?

## Beth gewch chi

| Cam | Beth mae WPM yn ei gadw | Darllenwch |
| --- | --- | --- |
| Recriwtio | Ceisiadau am swyddi, piblinell ymgeiswyr, cronfa ymgeiswyr, rhestrau gwirio ymsefydlu | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| Y cofnod | Un cofnod cyflogaeth i bob gweithiwr, y siart sefydliad, llinellau adrodd, grwpiau, trosglwyddo rhwng sefydliadau | [hr-core.md](../../../../spec/hr-core.md) |
| Hunanwasanaeth | Mae gweithiwr yn gweld ac yn diweddaru ei fanylion ei hun, yn gofyn am weithio hyblyg, yn rhoi rhybudd, ac yn gweld ei amser i ffwrdd | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Bywyd gwaith | Gwyliau, amser, shifftiau, buddion, awgrymiadau lles, addasiadau rhesymol, asesiadau ergonomig, y pwls dienw | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Datblygu | Adolygiadau, arfarniadau 360, cyrsiau trwy'r gofrestr cyrsiau, asesiadau, mentora, cynlluniau datblygu, olyniaeth, gyrfaoedd cynnar | [talent-development.md](../../../../spec/talent-development.md) |
| Cyflog | Cylchoedd cyflogres, slipiau cyflog, meincnodi cyflogau, chwilio graddfa gyflog genedlaethol, lefelau swydd | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Staff nad ydynt yn barhaol | Dyddiadau gorffen yn ôl sail, estyniadau, asesiadau statws, cynlluniau trosi a benderfynir gan berson, ariannu wrth ymyl diwedd y contract | [engagements.md](../../../../spec/engagements.md) |
| Gadael | Ymddiswyddo gyda chyfnod rhybudd, trosglwyddo ar y diwrnod olaf, cadw yn ôl math o gofnod, dileu | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Sut mae'n diogelu pobl

- **Angen gwybod wrth ddarllen.** Mae rheolwr yn gweld ei adrodd uniongyrchol ei hun, gweithiwr yn gweld ei hun, a Adnoddau Dynol yn gweld yr hyn sydd ei angen arnynt. Nid yw dieithryn sydd wedi mewngofnodi yn gweld dim am gydweithiwr ([auth.md](../../../../spec/auth.md)).
- **Ni chaiff diagnosis ei storio.** Mae absenoldeb salwch yn cofnodi math, nid rheswm.
- **Mae nodweddion categori arbennig wedi'u diffodd yn ddiofyn.** Mae monitro cydraddoldeb a gofynion iechyd yn y gweithle angen sail gyfreithlon wedi'i chofnodi, maen nhw'n wirfoddol neu'n cael eu cadw gan y rôl iechyd galwedigaethol, a dim ond grwpiau uwchlaw trothwy y maen nhw'n eu dangos. Nid ydynt byth yn fewnbwn i unrhyw sgôr na phenderfyniad am berson.
- **Mae sgorau'n cynghori.** Mae parodrwydd olyniaeth, bylchau sgiliau a rhagolygon yn awgrymu; mae person yn penderfynu, a chofnodir penderfyniadau fel trosi contractwr fel penderfyniad person, byth fel canlyniad awtomatig.
- **Gall y gwrthrych weld a dileu.** Mae allforio mynediad gwrthrych a dileu yn cwmpasu'r cofnodion uchod, gyda llyfr cyfrifon fel nad yw adfer yn dod â pherson a ddilëwyd yn ôl.

## Cyfyngiadau

- Heb ei adeiladu eto: rhyngwyneb defnyddiwr ar gyfer cynlluniau trosi ac ariannu, y duedd wrth gefn, a thudalen "fy manylion" y gweithiwr. Edrychwch ar [tasks.md](../../../../spec/tasks.md) am y cyflwr presennol.
- Mae cyfieithiadau wedi'u cynorthwyo gan beiriant; mae adolygiad proffesiynol yn gât agored.
- Dim ond ymchwil a chynnig yw cytundebau cyfunol a chyrff cydnabyddedig ([worker-unions.md](../../../../spec/worker-unions.md)).
- Bôn yw treth y gyflogres.

Gweler hefyd y [Prif Swyddog Cyfreithiol](../prif-swyddog-cyfreithiol/index.md) a'r [Prif Swyddog Gweithredol](../prif-swyddog-gweithredol/index.md).

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
