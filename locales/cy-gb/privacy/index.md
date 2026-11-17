# Hysbysiad preifatrwydd

Mae'r hysbysiad hwn yn dweud beth y mae gwefan Workforce Planning Management (WPM) a rhaglen WPM yn ei gadw yn eich porwr, pam, am ba hyd, a sut i'w ddileu. Y wefan yw <https://workforce-planning-management.github.io/>; y rhaglen yw meddalwedd arddangos WPM, lle bynnag y caiff ei rhedeg. (Diweddarwyd 2026-11-17, WPM-R253.)

Nid oes gan yr un o'r ddwy ddadansoddeg, hysbysebu, na sgriptiau, ffontiau na delweddau trydydd parti. Nid yw dim y mae'r naill na'r llall yn ei gadw yn eich porwr yn cael ei ddefnyddio i'ch dilyn o wefan i wefan.

## Y wefan

Set o dudalennau statig yw'r wefan. Nid oes mewngofnodi ac nid oes ffurflenni. Mae'r hyn y mae'n ei gadw yn eich porwr eich hun:

| Beth | Lle caiff ei gadw | Enw | Beth a gedwir | Pam |
| --- | --- | --- | --- | --- |
| Eich thema | Storfa leol eich porwr | `mxi.wpm.theme` | Enw'r thema, er enghraifft `dark` | Fel bod y dudalen nesaf a'ch ymweliad nesaf yn edrych fel y dewisoch |
| Eich thema | Cwci ar gyfer y wefan hon yn unig | `wpm-theme` | Enw'r thema, er enghraifft `dark` | Fel bod y thema'n dod yn ôl hyd yn oed os yw eich porwr yn cadw cwcis ond yn clirio'r storfa leol, neu i'r gwrthwyneb |
| Maint eich testun | Storfa leol eich porwr | `mxi.wpm.text-size` | Enw'r maint, er enghraifft `large` | Fel bod y dudalen nesaf a'ch ymweliad nesaf yn defnyddio maint y testun a ddewisoch |
| Eich iaith | Ni chedwir dim | | | Mae'r iaith yn rhan o gyfeiriad pob tudalen (er enghraifft `/cy-001/`), felly nid oes dim i'w gofio |
| Lle'r oeddech ar dudalen | Storfa sesiwn eich porwr, ar gyfer y tab hwn yn unig | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Safleoedd sgrolio, a threfn y tudalennau a agoroch yn y tab hwn | Fel bod Yn ôl ac Ymlaen yn mynd â chi'n ôl i'r lle'r oeddech. Y fframwaith y mae'r wefan wedi'i hadeiladu arno (SvelteKit) sy'n cadw'r rhain |

Ni chedwir unrhyw ddewis nes i chi ei wneud. Nes i chi ddewis, mae'r wefan yn dilyn gosodiad golau neu dywyll eich porwr a'i faint testun arferol, ac nid yw'n cadw thema na maint testun. Caiff dewis ei gadw pan fyddwch yn ei wneud yn y bar dewis, ac mae dewis eto yn cymryd ei le. Caiff y storfa sesiwn ei hysgrifennu wrth i chi symud rhwng tudalennau.

Nid yw rhannau eraill y wefan yn cadw dim:

- **Mae chwilio** yn rhedeg yn eich porwr, ar fynegai y mae'r wefan yn ei gyhoeddi, ac nid yw'n cadw dim. Caiff eich geiriau eu rhoi yng nghyfeiriad y dudalen (`?q=`) fel y gellir rhannu tudalen ganlyniadau; pan gaiff y cyfeiriad hwnnw ei lwytho o'r gwesteiwr gwe (wrth ail-lwytho, neu o ddolen a rannwyd), mae'r gwesteiwr yn ei dderbyn gyda'r cais, fel y mae gydag unrhyw gyfeiriad.
- **Nid yw'r llithryddion parodrwydd** ar y llwybr i ddefnydd cynhyrchu yn cadw dim, ac mae ail-lwytho yn adfer amcangyfrifon yr awdur.
- **Mae rhannu** yn agor y wefan a ddewiswch (e-bost, LinkedIn, Reddit, Bluesky neu Mastodon) dim ond pan fyddwch yn ei dewis. Hysbysiad preifatrwydd y wefan honno sy'n berthnasol wedyn.

## Rhaglen WPM

Mae'r rhaglen yn cadw'r un mathau o ddewisiadau, a'r cwci sy'n eich cadw wedi mewngofnodi:

| Beth | Lle caiff ei gadw | Enw | Beth a gedwir | Pam |
| --- | --- | --- | --- | --- |
| Eich thema | Storfa leol eich porwr | `mxi.wpm.theme` | Enw'r thema | Fel bod pob tudalen yn edrych fel y dewisoch |
| Maint eich testun | Storfa leol eich porwr | `mxi.wpm.text-size` | Enw'r maint | Fel bod pob tudalen yn defnyddio maint y testun a ddewisoch |
| Eich iaith | Storfa leol eich porwr | `mxi.wpm.locale` | Cod iaith y dudalen olaf a agoroch, er enghraifft `cy-001` | Fel bod cyfeiriad heb iaith yn agor yn eich iaith chi |
| Eich iaith | Cwci ar gyfer y rhaglen yn unig | `wpm-locale` | Yr un cod iaith | Fel y gall y gweinydd anfon cyfeiriad heb iaith i'ch iaith chi |
| Eich mewngofnodi | Cwci ar gyfer y rhaglen yn unig, na all sgriptiau'r dudalen ei ddarllen | `__Host-mxi_session` (`mxi_session` ar gyfrifiadur datblygwr ei hun) | Rhif ar hap sy'n sefyll dros eich sesiwn fewngofnodi | Fel eich bod yn aros wedi mewngofnodi o dudalen i dudalen; hebddo byddech yn mewngofnodi ar bob tudalen |
| Iaith y golygydd cynnwys | Storfa leol eich porwr, dim ond os byddwch yn agor y golygydd cynnwys (`/admin`) | `sveltia-cms.prefs` | Gosodiadau'r golygydd, gan gynnwys ei iaith | Fel bod y golygydd yn agor yn iaith y rhaglen |
| Lle'r oeddech ar dudalen | Storfa sesiwn eich porwr, ar gyfer y tab hwn yn unig | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Safleoedd sgrolio, a threfn y tudalennau a agoroch yn y tab hwn | Fel bod Yn ôl ac Ymlaen yn mynd â chi'n ôl i'r lle'r oeddech |

Yn wahanol i'r wefan, mae'r rhaglen yn cadw'r thema a maint y testun cyn gynted ag y mae'n eu dangos, cyn i chi ddewis, ac mae'n cadw iaith pob tudalen a agorwch. Caiff allweddi o'r cyfnod cyn i'r prosiect newid ei enw (`mxi.hcm.theme`, `mxi.hcm.locale`) eu symud i'r enwau uchod a'u dileu.

Y cwci mewngofnodi yw'r unig eitem sy'n sefyll drosoch chi. Nid yw'r rhif ar hap ynddo yn golygu dim ar ei ben ei hun: mae'r sesiwn fewngofnodi y mae'n pwyntio ati yn cael ei chadw gan weinydd y rhaglen, ac mae allgofnodi yn ei dod i ben. Mae'r hyn y mae'r rhaglen yn ei ddal am y bobl yn ei chofnodion wedi'i gynnwys yn yr adran olaf ond un, isod.

## Pwy sy'n ei dderbyn

Nid yw'r storfa leol na'r storfa sesiwn byth yn gadael eich porwr.

Caiff cwci ei anfon gyda phob cais y mae eich porwr yn ei wneud i'r wefan a'i gosododd, ac i ddim gwefan arall (`SameSite=Lax`, a `Secure` dros HTTPS):

- Mae cwci `wpm-theme` y wefan yn mynd i GitHub Pages, sy'n lletya'r wefan. Nid oes yr un o raglenni'r prosiect yn ei ddarllen.
- Mae cwci `wpm-locale` a chwcis mewngofnodi'r rhaglen yn mynd i'r gweinydd y mae'r rhaglen yn rhedeg arno, sy'n eu darllen i ddewis eich iaith ac i wybod eich bod wedi mewngofnodi.

Mae GitHub, sy'n lletya'r wefan, yn derbyn y ceisiadau y mae eich porwr yn eu gwneud, fel y mae unrhyw westeiwr gwe. Mae [datganiad preifatrwydd GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) yn dweud beth y mae'n ei gofnodi. Y sefydliad sy'n rhedeg y rhaglen sy'n penderfynu beth y mae ei weinydd yn ei gofnodi.

## Am ba hyd y caiff ei gadw

- Cwci `wpm-theme` y wefan: 400 diwrnod calendr ar ôl i chi ddewis thema ddiwethaf, yr hiraf y mae porwr yn cadw cwci.
- Cwci `wpm-locale` y rhaglen: 365 diwrnod calendr ar ôl i chi agor tudalen ddiwethaf.
- Y cwci mewngofnodi: nes i chi allgofnodi neu gau'r porwr.
- Storfa leol: nes i chi ei chlirio.
- Storfa sesiwn: nes i chi gau'r tab.

## Sut i'w ddileu

Cliriwch ddata'r wefan yng ngosodiadau eich porwr (cwcis a data gwefannau), ar gyfer y wefan, y rhaglen, neu'r ddwy. Wedyn mae'r wefan yn ymddwyn fel ar ymweliad cyntaf; mae'r rhaglen yn gofyn i chi fewngofnodi eto. Mae allgofnodi o'r rhaglen yn dileu'r cwci mewngofnodi.

## Pam nad oes baner caniatâd

Caiff pob eitem ei chadw i roi'r gwasanaeth y gofynnoch amdano: y wefan neu'r rhaglen fel y gosodoch hi, yn eich iaith, ac, yn y rhaglen, wedi mewngofnodi. Dyna'r math o storio y mae rheolau'r DU a'r UE ar gwcis (y Rheoliadau Preifatrwydd a Chyfathrebu Electronig, a'r Gyfarwyddeb e-Breifatrwydd) yn ei drin fel rhywbeth cwbl angenrheidiol, nad oes angen caniatâd ar ei gyfer. Dyma ddehongliad y prosiect o'r rheolau hynny, nid cyngor cyfreithiol.

## Y cofnodion y mae'r rhaglen yn eu dal

Mae'r hysbysiad hwn yn ymdrin â'r hyn a gedwir yn eich porwr. Mae'r rhaglen hefyd yn dal cofnodion am bobl (cyflogeion, ymgeiswyr a'u cysylltiadau) ar ei gweinydd. Pwy bynnag sy'n rhedeg y rhaglen sy'n gyfrifol am y cofnodion hynny ac sy'n ysgrifennu'r hysbysiad preifatrwydd i'r bobl ynddynt ([y rhestr wirio asesu](../../../spec/governance/assessment-checklist.md), Saesneg, cwestiwn 3.10). Dim ond data synthetig y mae'r feddalwedd arddangos yn ei ddal.

## Cwestiynau

Gofynnwch yn ystorfa'r prosiect, <https://github.com/workforce-planning-management/workforce-planning-management>, neu ysgrifennwch at y cynhaliwr a enwir yno.
