# Taarifa ya faragha

Taarifa hii inaeleza tovuti ya Usimamizi wa Mipango ya Wafanyakazi (WPM) na programu ya WPM zinahifadhi nini
kwenye kivinjari chako, kwa nini, kwa muda gani, na jinsi ya kukiondoa. Tovuti ni
<https://workforce-planning-management.github.io/>; programu ni programu ya maonyesho ya WPM, popote
inapoendeshwa. (Imesasishwa 2026-11-17, WPM-R253.)

Hakuna kati ya hizo mbili chenye uchanganuzi wa takwimu, matangazo, wala hati, fonti au picha za wahusika wengine. Hakuna
kinachohifadhiwa na kimojawapo kwenye kivinjari chako kinachotumika kukufuatilia kutoka tovuti hadi tovuti.

## Tovuti

Tovuti ni seti ya kurasa tuli. Haina kuingia wala fomu. Kinachohifadhiwa kiko kwenye kivinjari chako mwenyewe:

| Nini | Kinahifadhiwa wapi | Jina | Kinachohifadhiwa | Kwa nini |
| --- | --- | --- | --- | --- |
| Mandhari yako | Hifadhi ya ndani ya kivinjari chako | `mxi.wpm.theme` | Jina la mandhari, kwa mfano `dark` | Ili ukurasa unaofuata na ziara yako ijayo zionekane kama ulivyochagua |
| Mandhari yako | Kuki ya tovuti hii pekee | `wpm-theme` | Jina la mandhari, kwa mfano `dark` | Ili mandhari irudi hata kama kivinjari chako kinahifadhi kuki lakini kinafuta hifadhi ya ndani, au kinyume chake |
| Ukubwa wa maandishi yako | Hifadhi ya ndani ya kivinjari chako | `mxi.wpm.text-size` | Jina la ukubwa, kwa mfano `large` | Ili ukurasa unaofuata na ziara yako ijayo zitumie ukubwa wa maandishi ulioupenda |
| Lugha yako | Hakuna kinachohifadhiwa | | | Lugha ni sehemu ya anwani ya kila ukurasa (kwa mfano `/cy-001/`), kwa hiyo hakuna cha kukumbuka |
| Ulipokuwa kwenye ukurasa | Hifadhi ya kipindi ya kivinjari chako, kwa kichupo hiki pekee | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Nafasi za kusogeza ukurasa, na mpangilio wa kurasa ulizofungua kwenye kichupo hiki | Ili Nyuma na Mbele zikurudishe ulipokuwa. Mfumo uliotumika kujenga tovuti (SvelteKit) huzihifadhi |

Hakuna chaguo linalohifadhiwa hadi uliufanye. Hadi uchague, tovuti hufuata mpangilio wa mwanga au giza wa kivinjari chako
na ukubwa wake wa kawaida wa maandishi, na haihifadhi mandhari wala ukubwa wa maandishi. Chaguo huhifadhiwa unapolifanya kwenye upau wa kuchagua,
na kuchagua tena huchukua nafasi yake. Hifadhi ya kipindi huandikwa unaposogea kati ya kurasa.

Sehemu nyingine za tovuti hazihifadhi chochote:

- **Utafutaji** hufanyika kwenye kivinjari chako, kwa faharasa ambayo tovuti huichapisha, na hauhifadhi chochote. Maneno yako huwekwa kwenye
  anwani ya ukurasa (`?q=`) ili ukurasa wa matokeo ushirikiwe; anwani hiyo inapopakiwa kutoka kwa mwenyeji wa wavuti (wakati wa
  kupakia upya, au kutoka kiungo kilichoshirikiwa), mwenyeji huipokea pamoja na ombi, kama anavyopokea anwani yoyote.
- **Vitelezi vya utayari** kwenye njia ya kuelekea uzalishaji havihifadhi chochote, na kupakia upya hurudisha
  makadirio ya mwandishi.
- **Shiriki** hufungua tovuti unayoichagua (barua pepe, LinkedIn, Reddit, Bluesky, au Mastodon) pale tu unapoichagua. Taarifa ya
  faragha ya tovuti hiyo ndipo inapotumika.

## Programu ya WPM

Programu huhifadhi aina zilezile za chaguo, na kuki inayokuweka umeingia:

| Nini | Kinahifadhiwa wapi | Jina | Kinachohifadhiwa | Kwa nini |
| --- | --- | --- | --- | --- |
| Mandhari yako | Hifadhi ya ndani ya kivinjari chako | `mxi.wpm.theme` | Jina la mandhari | Ili kila ukurasa uonekane kama ulivyochagua |
| Ukubwa wa maandishi yako | Hifadhi ya ndani ya kivinjari chako | `mxi.wpm.text-size` | Jina la ukubwa | Ili kila ukurasa utumie ukubwa wa maandishi ulioupenda |
| Lugha yako | Hifadhi ya ndani ya kivinjari chako | `mxi.wpm.locale` | Msimbo wa lugha wa ukurasa wa mwisho ulioufungua, kwa mfano `cy-001` | Ili anwani isiyo na lugha ifunguke kwa lugha yako |
| Lugha yako | Kuki ya programu pekee | `wpm-locale` | Msimbo uleule wa lugha | Ili seva iweze kutuma anwani isiyo na lugha kwa lugha yako |
| Kuingia kwako | Kuki ya programu pekee, ambayo hati za ukurasa haziwezi kuisoma | `__Host-mxi_session` (`mxi_session` kwenye kompyuta ya msanidi mwenyewe) | Namba ya nasibu inayowakilisha kipindi chako cha kuingia | Ili ubaki umeingia kutoka ukurasa hadi ukurasa; bila hiyo ungeingia kwenye kila ukurasa |
| Lugha ya kihariri cha maudhui | Hifadhi ya ndani ya kivinjari chako, ikiwa tu unafungua kihariri cha maudhui (`/admin`) | `sveltia-cms.prefs` | Mipangilio ya kihariri, ikiwemo lugha yake | Ili kihariri kifunguke kwa lugha ya programu |
| Ulipokuwa kwenye ukurasa | Hifadhi ya kipindi ya kivinjari chako, kwa kichupo hiki pekee | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Nafasi za kusogeza ukurasa, na mpangilio wa kurasa ulizofungua kwenye kichupo hiki | Ili Nyuma na Mbele zikurudishe ulipokuwa |

Tofauti na tovuti, programu huhifadhi mandhari na ukubwa wa maandishi mara tu inapovionyesha, kabla hujachagua, na huhifadhi lugha ya kila ukurasa unaoufungua. Funguo za kabla ya mradi kubadilishwa jina (`mxi.hcm.theme`, `mxi.hcm.locale`) huhamishiwa kwenye
majina yaliyo hapo juu na kufutwa.

Kuki ya kuingia ndicho kitu pekee kinachokuwakilisha. Namba ya nasibu iliyomo haimaanishi chochote yenyewe: kipindi
cha kuingia inachoelekeza kinahifadhiwa na seva ya programu, na kutoka huukomesha. Kile ambacho programu
huhifadhi kuhusu watu walio kwenye rekodi zake kimeelezwa katika sehemu ya pili kutoka mwisho, hapa chini.

## Nani anapokea

Hifadhi ya ndani na hifadhi ya kipindi hazitoki kamwe nje ya kivinjari chako.

Kuki hutumwa pamoja na kila ombi ambalo kivinjari chako hufanya kwa tovuti iliyoiweka, na si kwa tovuti nyingine yoyote
(`SameSite=Lax`, na `Secure` kupitia HTTPS):

- Kuki ya `wpm-theme` ya tovuti huenda kwa GitHub Pages, inayopangisha tovuti. Hakuna programu ya mradi
  inayoisoma.
- Kuki za `wpm-locale` na za kuingia za programu huenda kwa seva ambayo programu inaendeshwa juu yake, ambayo huzisoma
  ili kuchagua lugha yako na kujua umeingia.

GitHub, inayopangisha tovuti, hupokea maombi ambayo kivinjari chako hufanya, kama mwenyeji yeyote wa wavuti.
[Taarifa ya faragha ya GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
inasema inarekodi nini. Shirika linaloendesha programu huamua seva yake inarekodi nini.

## Kinahifadhiwa kwa muda gani

- Kuki ya `wpm-theme` ya tovuti: siku 400 za kalenda baada ya kuchagua mandhari mara ya mwisho, muda mrefu zaidi ambao kivinjari huhifadhi
  kuki.
- Kuki ya `wpm-locale` ya programu: siku 365 za kalenda baada ya kufungua ukurasa mara ya mwisho.
- Kuki ya kuingia: hadi utoke au ufunge kivinjari.
- Hifadhi ya ndani: hadi uifute.
- Hifadhi ya kipindi: hadi ufunge kichupo.

## Jinsi ya kukiondoa

Futa data za tovuti kwenye mipangilio ya kivinjari chako (kuki na data za tovuti), kwa tovuti, programu, au
zote mbili. Tovuti kisha hufanya kazi kama katika ziara ya kwanza; programu inakuomba uingie tena. Kutoka kwenye
programu huondoa kuki ya kuingia.

## Kwa nini hakuna bango la idhini

Kila kipengele kinahifadhiwa ili kukupa huduma uliyoiomba: tovuti au programu kama ulivyoipanga, kwa
lugha yako, na, kwenye programu, ukiwa umeingia. Hiyo ndiyo aina ya uhifadhi ambayo kanuni za Uingereza na Umoja wa Ulaya kuhusu kuki (the
Privacy and Electronic Communications Regulations, na ePrivacy Directive) huchukulia kuwa ni wa lazima kabisa, ambao
hauhitaji idhini. Huu ni uelewa wa mradi wa kanuni hizo, si ushauri wa kisheria.

## Rekodi ambazo programu inashikilia

Taarifa hii inahusu kinachohifadhiwa kwenye kivinjari chako. Programu pia inashikilia rekodi kuhusu watu (wafanyakazi,
waombaji, na watu wao wa mawasiliano) kwenye seva yake. Yeyote anayeendesha programu anawajibika kwa rekodi hizo na
huandika taarifa ya faragha kwa watu walio ndani yake
([orodha hakiki ya tathmini](../../../spec/governance/assessment-checklist.md), swali la 3.10). Programu ya
maonyesho inashikilia data bandia pekee.

## Maswali

Uliza kwenye hazina ya mradi, <https://github.com/workforce-planning-management/workforce-planning-management>,
au mwandikie msimamizi aliyetajwa huko.

> Tafsiri hii ilitolewa na akili bandia na haijakaguliwa na mzungumzaji asilia wa Kiswahili. Ikitofautiana na toleo la Kiingereza (`locales/en-001/privacy/`), toleo la Kiingereza ndilo linalotangulia.
