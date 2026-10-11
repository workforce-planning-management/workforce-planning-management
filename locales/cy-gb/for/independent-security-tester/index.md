# Profi diogelwch annibynnol

**Y cwestiwn rydych chi'n debygol o'i ofyn:** sut mae cael pobl na ysgrifennodd y feddalwedd hon i ymosod arni, beth ddylem ni eu cyfeirio ato, a beth ydyn ni'n ei wneud â'r hyn a ganfyddant?

Ni wnaed unrhyw brawf annibynnol. Ysgrifennwyd y [model bygythiadau](../../../../spec/governance/threat-model.md) gan yr un cynorthwyydd a ysgrifennodd y feddalwedd ac ni adolygwyd ef gan neb arall, a'r un awdur a roddodd bob rheolaeth a enwir ynddo ar waith. Mae'r [asesiad parodrwydd](../../../../spec/readiness-assessment.md) yn rhestru prawf treiddio annibynnol fel gât na all ond pobl ei chau (WPM-T201, **heb ei wneud**). Briff yw'r dudalen hon ar gyfer y sawl sy'n comisiynu un. Nid cynllun prawf gan brofwr mohoni ac nid yw'n cymryd ei le.

## Beth i'w roi i'r profwyr

| Eitem | Ble mae | Nodiadau |
| --- | --- | --- |
| Defnydd i ymosod arno | Eich un chi: [canllaw defnyddio](../../../../spec/operations/deployment.md) | Gosodiadau cynhyrchu gyda mewngofnodi **ymlaen**, darparwr hunaniaeth prawf, a **data synthetig yn unig**. Peidiwch â phrofi yn erbyn cofnodion pobl go iawn |
| Y ffynhonnell | Yr ystorfa hon | Mae prawf gyda'r ffynhonnell yn fwy trylwyr nag un bocs du; dywedwch pa un rydych chi ei eisiau |
| Y model bygythiadau | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Pedwar bygythiad ar hugain, y rheolaeth fel y'i hadeiladwyd, a'r bylchau. Rhowch ef iddynt **ar ôl** y rownd bocs du gyntaf os ydych am weld beth maen nhw'n ei ganfod heb ei awgrymu |
| Disgrifiad yr API | `GET /api-docs/openapi.json` | Pob llwybr, gyda'r `Idempotency-Key` ar bob gweithrediad sy'n newid rhywbeth |
| Dyluniad mewngofnodi | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Personâu, priodoleddau, a'r ffaith bod y polisi'n penderfynu yn ôl gweithred yn unig |
| Hunaniaethau prawf | Chi sy'n eu creu | O leiaf: gweithiwr heb unrhyw briodoleddau, rheolwr llinell, rheolwr y tu allan i'r gadwyn, Adnoddau Dynol, cyflogres, gweinyddwr, a chymar gwasanaeth |

## Beth i ofyn iddynt edrych arno

Ni allai'r awdur wirio'r rhain. Mae pob rhes yn enwi cofnod y model bygythiadau ei hun, fel y gellir olrhain canfyddiad.

| Maes | Beth i'w roi ar brawf | Bygythiad |
| --- | --- | --- |
| Tocynnau | Dryswch algorithm; set allweddi sy'n cylchdroi tra bo ceisiadau ar y gweill; tocyn sydd wedi dod i ben neu heb ddod yn ddilys eto; tocyn ar gyfer cynulleidfa arall | T1, T2 |
| Angen gwybod | Pob llwybr darllen fel galwr heb briodoleddau ac fel cymydog; llwybr nad yw'r tabl dosbarthu'n ei adnabod; ysgrifennu y mae'r polisi'n ei ganiatáu ond y dylai'r trinwr ei wrthod | T3, T4, T6 |
| Priodoleddau breintiedig | A ellir ychwanegu, ailchwarae neu etifeddu priodoledd mewn tocyn; pwy all ei bathu; gweinyddwr a rennir neu hirhoedlog | T24 |
| Chwistrellu | Ffwsio pob paramedr, segment llwybr a maes corff; adeiladir y datganiadau dileu a chadw o ddynodwyr wedi'u teipio, a darllenwyd hwy ond ni chawsant eu ffwsio | T12 |
| Y cleient porwr | Trin sesiynau gan weinydd y rhyngwyneb, sut mae'n cyfnewid sesiwn am docyn, sgriptio traws-safle drwy destun wedi'i storio, osgoi'r polisi diogelwch cynnwys, a thudalen y rheolwr cynnwys sy'n llwytho sgript o rwydwaith cyhoeddus heb fersiwn wedi'i phinio | T13, T14 |
| Ceisiadau allanol | Ffugio ceisiadau ochr-gweinydd drwy unrhyw URL wedi'i ffurfweddu (setiau allweddi, y gwasanaethau cyfatebol, y storfa dros dro a rennir) | T15 |
| Terfynau | Terfyn y corff, terfyn amser y cais, y terfynau cyfradd fesul enghraifft, a'r golygon sy'n llwytho cofnodion sefydliad cyfan i'r cof | T16, T17 |
| Logiau | A yw unrhyw ddata personol, tocyn neu linyn ymholiad yn cyrraedd log | T19 |
| Allweddi anunig-weithredol | Allwedd a ddefnyddir gan ddau alwr; yr un allwedd gyda chorff, llwybr neu ymholiad wedi'i newid; allwedd bron yn ddilys; corff mawr iawn; dau gais yn rasio dan un allwedd (rhesymwyd ynghylch yr hawlio, ni phrofwyd ef mewn ras); a all neb ond ei alwr ddarllen ymateb wedi'i storio | newydd |
| Y llwybr archwilio | Unrhyw ffordd i newid, tynnu neu hepgor cofnod; y pwynt terfyn dilysu cadwyn | T7 |
| Copïau wrth gefn | Adfer o gopi wrth gefn wedi'i ymyrryd ag ef neu â'r allwedd anghywir; a yw person a ddilëwyd yn dychwelyd | T10, T11 |
| Cadwyn gyflenwi | Pinio gweithredoedd adeiladu, sganio cyfrinachau, llofnodi delweddau | T20, T21 |

## Rheolau i'w setlo cyn iddynt ddechrau

Llenwch y rhain gyda'ch timau diogelwch a chyfreithiol eich hunain. Ni phenderfynir yr un ohonynt yma.

- **Awdurdod:** pwy sy'n llofnodi bod y prawf wedi'i awdurdodi, ac ar gyfer pa systemau a chyfeiriadau.
- **Ffenestr:** y dyddiadau a'r oriau, wedi'u cyfrif mewn diwrnodau calendr, a phwy i'w ffonio i'w atal.
- **Cwmpas:** beth sydd i mewn (yr API, y rhyngwyneb, y llif mewngofnodi, y sgript copi wrth gefn) a beth sydd allan (gwasanaeth y darparwr hunaniaeth ei hun, eich rhwydwaith, unrhyw beth a rennir â systemau eraill).
- **Data:** synthetig yn unig. Os bydd profwr yn canfod data personol go iawn, pwy a hysbysir a sut y'i trinnir.
- **Profion dinistriol:** a ganiateir gwrthod gwasanaeth, ysgrifennu swmp a dileu, a lle.
- **Adrodd:** y fformat, y raddfa ddifrifoldeb, pwy sy'n ei dderbyn, am ba mor hir y gallant ei gadw, a sut y trinnir cyfrinachau a ganfyddant.
- **Ailbrofi:** bod y contract yn cynnwys gwirio'r atgyweiriadau.

## Beth i'w wneud â'r canfyddiadau

1. Cofnodwch bob un fel tasg yn [tasks.md](../../../../spec/tasks.md) gyda chyfeirnod y profwr, y dystiolaeth, a'r difrifoldeb a roddwyd ganddynt. Mae canfyddiad a herir yn aros wedi'i gofnodi fel un a herir, gyda'r rheswm.
2. Atgyweiriwch gyda phrawf sy'n methu'n gyntaf. Os rheol yw'r atgyweiriad, ychwanegwch hi at golofn bylchau'r model bygythiadau a'i chau yno dim ond pan fydd yr ailbrofi'n pasio.
3. Peidiwch â marcio WPM-T201 yn gyflawn nes bod yr adroddiad gennych, bod gan bob canfyddiad berchennog, a bod ailbrofi wedi'i gwblhau. Diweddarwch yr [asesiad parodrwydd](../../../../spec/readiness-assessment.md) a'r rhestr gatiau (WPM-T203) yn yr un newid.
4. Cadwch yr adroddiad allan o'r ystorfa os yw'n cynnwys unrhyw beth y gellir ei ecsbloetio; cysylltwch â lle y'i cedwir.

## Beth mae adroddiad glân yn ei olygu a beth nad yw'n ei olygu

Mae prawf heb ganfyddiadau yn dangos yn unig na chanfu'r profwyr hyn ddim byd, yn y ffenestr hon, gyda'r cwmpas hwn a'r hunaniaethau hyn. Nid yw'n dangos nad oes diffygion, nid yw'n cwmpasu ffurfwedd a newidiwyd gennych wedyn, ac nid yw'n cymryd lle'r gatiau eraill: archwiliad hygyrchedd, eich asesiad effaith diogelu data wedi'i lofnodi gan eich swyddog, ac ymarfer adfer ar eich platfform.

## Beth nad yw'r dudalen hon yn ei wneud

Nid yw'n enwi cyflenwr, yn gosod pris nac yn addo safon. Chi sydd i benderfynu pa gynllun neu achrediad y mae eich sector yn ei ddisgwyl gan brofwr.

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
