# Usimamizi wa Mipango ya Wafanyakazi

**Jukwaa la rasilimali watu la chanzo huria linalohusu mzunguko mzima wa maisha ya mfanyakazi, kuanzia kuajiriwa hadi kustaafu.**

Usimamizi wa Mipango ya Wafanyakazi (WPM) unaunganisha uajiri, muda na likizo, maendeleo ya wafanyakazi, mipango ya wafanyakazi na malipo ya mishahara katika programu moja. Ni programu ya maonyesho, iliyokusudiwa kwa tathmini na kama muundo wa marejeo.

Lugha: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Programu ya maonyesho, bado haijawa tayari kutumika katika uzalishaji.** WPM hutumia data bandia pekee, na hesabu zake za kisheria ni za mfano. Usiweke data halisi ya kibinafsi. [Njia ya kuelekea uzalishaji](path-to-production/index.md) inaeleza kinachohitajika kwa matumizi ya uzalishaji na kila sehemu iko tayari kwa kiasi gani. Tazama pia [msimamo wa udhibiti](../../spec/regulatory.md).

[Endesha onyesho](start/index.md) [Muhtasari wa majukumu](for/index.md) [Msimbo chanzo kwenye GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Faida

Data za rasilimali watu mara nyingi huwekwa katika zana tofauti, kama vile visanduku vya barua vya waombaji, lahajedwali za likizo na ubao wa ratiba za zamu. Kila uhamishaji kati ya zana hizo huongeza kazi ya mikono na hatari ya makosa. WPM imeundwa kupunguza uhamishaji huo.

- **Mfumo mmoja.** Rekodi za kuwapokea wafanyakazi wapya, likizo, mafunzo na malipo huwekwa katika programu moja, jambo linalopunguza kuandika upya data.
- **Kupunguza data.** Data ambayo haipaswi kuhifadhiwa haina sehemu ya kuijaza, na data ambayo haipaswi kufichuliwa haina njia ya API. Kwa mfano, hakuna sehemu ya utambuzi wa ugonjwa, na majibu ya utafiti mfupi wa mara kwa mara hayana mwandishi.
- **Kanuni za kuripoti.** Thamani zisizojulikana haziripotiwi kama sifuri, jumla hazimtambui mtu binafsi, na makadirio hueleza msingi wake.
- **Ujanibishaji.** Programu inapatikana katika lugha na maeneo 25, na muhtasari wa majukumu katika 21, ikiwemo lugha zinazoandikwa kutoka kulia kwenda kushoto. Kanuni za kila nchi huwekwa kama data. Tafsiri zilitolewa na akili bandia na hazijakaguliwa na wazungumzaji asilia.
- **Ufuatiliaji.** Msimbo chanzo uko wazi, na kila tabia inafuatiliwa hadi hitaji lenye nambari, uamuzi wa usanifu na kazi.
- **Usambazaji huru.** Mfumo ni API ya JSON ya Rust, kiteja cha SvelteKit na PostgreSQL. Hujengwa kutoka nakala mpya, na miunganisho na programu dada ni ya hiari.

## Vipengele

- **Uajiri na rekodi.** Maombi ya nafasi, mtiririko wa waombaji, kuwapokea wafanyakazi wapya, rekodi za wafanyakazi na chati ya shirika.
- **Muda na ratiba.** Muda na likizo, zamu zenye ukaguzi wa saa za kazi, ratiba ya kuwa tayari kuitwa pamoja na kubadilishana na vikumbusho, na orodha ya wafanyakazi.
- **Maendeleo.** Tathmini za utendaji, maoni ya digrii 360, kujifunza, uchambuzi wa pengo la ujuzi, mapendekezo ya muda wa mafunzo na mafunzo ya lazima.
- **Mipango na kuripoti.** Mipango ya kimkakati ya wafanyakazi, vipimo na maarifa, na dashibodi ya Afisa Mkuu Mtendaji.
- **Wanaoingia na kuondoka.** Orodha hakiki zenye tarehe na makabidhiano ya siku ya mwisho yenye kumbukumbu ya ukaguzi.
- **Mawasiliano.** Matangazo yenye hadhira, viungo na idadi ya waliosoma.
- **Haki za mfanyakazi na ustawi.** Vidokezo vya ustawi, utafiti mfupi wa mara kwa mara usiotaja majina, marekebisho ya kuwezesha, ufikiaji wa data na mhusika, ufutaji na uhifadhi.
- **Malipo.** Malipo ya mishahara na fidia, kwa hesabu za kisheria za mfano.

## Ulinganisho

Jedwali linalinganisha WPM na aina tatu za mifumo ya rasilimali watu. Ni mwelekeo wa jumla, si kipimo cha ulinganifu. Tazama pia [ulinganisho kwa bidhaa](../../docs/compare/index.md).

| | WPM | Mifumo kamili (Workday, SAP SuccessFactors, BambooHR) | Rasilimali watu ya chanzo huria (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Zana za kazi moja (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Wigo** | Mzunguko mzima, kuanzia kuajiriwa hadi kustaafu | Wigo unaofanana katika jukwaa moja | Kazi kuu zilezile | Kazi moja |
| **Leseni** | Chanzo huria | Ya kibiashara | Chanzo huria | Ya kibiashara |
| **Nguvu kuu** | Kupunguza data, usanifu unaofuatilika, uchambuzi wa pengo la ujuzi na muda wa mafunzo | Jukwaa moja lililounganishwa na kuungwa mkono | Miunganisho, ujanibishaji na usambazaji zaidi | Kina katika kazi moja |
| **Matumizi ya kawaida** | Tathmini, muundo wa marejeo au maendeleo zaidi | Mashirika yanayohitaji mfumo unaoungwa mkono | Mashirika yanayotaka chanzo huria katika uzalishaji | Mashirika yanayohitaji kina katika kazi moja na yanaweza kuunganisha mengine |

## Hadhira iliyokusudiwa

- **Watendaji wakuu.** [Muhtasari wa majukumu](for/index.md) unaeleza kile WPM inachotoa kwa kila jukumu la uongozi na jinsi ya kusoma matokeo yake.
- **Timu za rasilimali watu.** [Mafunzo ya onyesho](start/index.md) yanashughulikia kila ukurasa kwa kutumia data bandia.
- **Wasanidi programu na wasanifu.** Tazama [maelezo ya kiufundi](../../spec/index.md) na [mwongozo wa usakinishaji](../../INSTALL.md).
- **Watathmini.** [Tathmini ya utayari](../../spec/readiness-assessment.md) inaorodhesha mapengo kati ya WPM na matumizi ya uzalishaji.

## Kupata msaada

- **Ripoti hitilafu au uliza swali** katika [kifuatilia masuala](https://github.com/workforce-planning-management/workforce-planning-management/issues). Taja ukurasa, ulichofanya, ulichotarajia na kilichotokea. Usijumuishe data halisi ya kibinafsi.
- **Ripoti tatizo la usalama** kwa barua pepe, kama ilivyoelezwa katika [sera ya usalama](../../SECURITY.md), si katika suala la hadharani.
- **Jifunze kazi katika programu.** Kiungo cha **Msaada** kwenye kichwa cha programu hufungua ukurasa wa msaada na miongozo ya hatua kwa hatua kwa wafanyakazi, mameneja na rasilimali watu. Kila ujumbe wa hitilafu pia unaelekeza huko.

## Hatua zinazofuata

1. **Soma muhtasari wa jukumu lako.** Kila muhtasari unaeleza kile WPM inachotoa na kinachobaki wazi.
2. **Endesha onyesho.** Ziara hii inaendeshwa kwenye kompyuta ya hapa nawe kwa data bandia.
3. **Tuma maoni.** Tazama [jinsi ya kutoa maoni](../../RFC.md) na [jinsi ya kuchangia](../../CONTRIBUTING.md).

[Endesha onyesho](start/index.md) [Nyaraka](../../index.md)

> Tafsiri hii ilitolewa na akili bandia na haijakaguliwa na mzungumzaji asilia wa Kiswahili. Ikitofautiana na toleo la Kiingereza (`locales/en-001/`), toleo la Kiingereza ndilo linalotangulia.
