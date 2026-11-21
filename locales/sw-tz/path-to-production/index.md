# Njia ya kuelekea uzalishaji

WPM ni programu ya maonyesho. Ukurasa huu unaeleza matumizi ya uzalishaji yangemaanisha nini kwa WPM, na unaorodhesha kinachosimama kati ya WPM na matumizi hayo. Kila kipengele kina makadirio ya kiasi kilichokuwa tayari, kuanzia asilimia 0 hadi 100.

## Uzalishaji maana yake nini

Matumizi ya uzalishaji yanamaanisha shirika linategemea WPM kwa kazi halisi:

- rekodi ni za watu halisi: wafanyakazi, waombaji na watu wao wa mawasiliano ya dharura;
- malipo yanayokokotolewa na WPM hulipwa, na sheria ya mahali pa kazi inayahusu;
- watu huingia kila siku, na baadhi ya wanachosoma ni nyeti, kama vile kutokuwepo kazini kwa sababu ya ugonjwa;
- shirika linawajibika kwa mdhibiti, mkaguzi na watu walio kwenye rekodi kuhusu jinsi linavyoendeshwa.

Ukurasa huu unapima WPM dhidi ya hali ngumu zaidi iliyotathminiwa: shirika kubwa la matibabu au la serikali ([tathmini ya utayari](../../../spec/readiness-assessment.md)). Shirika dogo linaweza kuhitaji kidogo kwa baadhi ya vipengele, lakini linahitaji aina zilezile za ushahidi.

Uzalishaji unahitaji aina tatu za ushahidi, na programu pekee hutoa ya kwanza tu:

1. **Programu** inayofanya kazi kwa usalama, pamoja na majaribio yanayoonyesha hivyo.
2. **Mwekaji** anayeipangisha na kuiendesha: nakala rudufu, ufuatiliaji, msaada na kukabiliana na matukio.
3. **Watu wasio mwandishi** ambao wameikagua: mjaribu wa kuvamia mfumo, mkaguzi wa ufikivu, watafsiri na afisa wa ulinzi wa data wa mwekaji.

## Jinsi ya kusoma makadirio

- Makadirio ni hukumu ya mwandishi tarehe 2026-11-13, iliyofanywa kutokana na maelezo ya kiufundi na majaribio. Si kipimo na si cheti cha uthibitisho.
- Asilimia 0 inamaanisha bado hakuna kitu kilichopo. Asilimia 100 inamaanisha imejengwa, kujaribiwa na kukaguliwa na mtu huru kwenye usambazaji halisi. Hakuna kipengele kilicho asilimia 100.
- Kila kipengele kinasema makadirio yake yanategemea nini na nini kinabaki.
- Makadirio hayajumlishwi wala kuwekewa wastani. Kipengele kimoja chenye makadirio ya chini kinatosha kuizuia WPM kuingia uzalishajini.

## Programu

- **75%** **Kuingia na ufikiaji wa haja ya kujua.** Kuingia kumewashwa kwa chaguomsingi, na huduma hukataa kuanza katika uzalishaji bila hiyo (WPM-R72). Usomaji unawekewa mipaka kwa watu wenye haja ya kujua (WPM-T188), na tokeni ina umri wa juu (WPM-T200). Inategemea: majaribio ya kutekeleza masharti yanayoendeshwa kwenye CI. Kinachobaki: hakuna orodha ya kubatilisha tokeni, na hakuna jaribio huru la kuvamia mfumo lililoendeshwa.
- **70%** **Kumbukumbu ya ukaguzi kamili isiyobadilishwa.** Kumbukumbu ya ukaguzi ni ya kuongeza tu na imeunganishwa kwa heshi, na inaweza kusafirishwa kwenda mfumo wa ufuatiliaji (WPM-T189, WPM-T197). Inategemea: majaribio yaliyovunjwa kwa makusudi ili kuthibitisha kwamba yanashindwa. Kinachobaki: mkia uliokatwa wa mnyororo haugunduliwi, na mmiliki wa hifadhidata bado anaweza kuzima vichochezi.
- **70%** **Ulinzi wa data kwa usanifu.** Watu hutambuliwa kwa marejeo, hakuna safu za taarifa za idadi ya watu, na kuna ufikiaji wa mhusika, ufutaji na uhifadhi kwa kila aina ya rekodi. Inategemea: majaribio, na sehemu ya 1 ya tathmini ya utayari. Kinachobaki: tathmini ya mwekaji mwenyewe na msingi wake halali wa kisheria (hapa chini).
- **75%** **Miunganisho na nakala rudufu zilizosimbwa.** Katika uzalishaji huduma hukataa muunganisho wa hifadhidata usiosimbwa na hukatisha ombi lililokwama (WPM-T191), na nakala rudufu husimbwa (WPM-T192). Inategemea: majaribio ya yote mawili. Kinachobaki: usimamizi wa funguo na vyeti ni wa mwekaji.
- **10%** **Hesabu za kisheria.** Hesabu za kisheria ni za mfano, na kodi ya mishahara ni sehemu ya mfano tu ([msimamo wa udhibiti](../../../spec/regulatory.md)). Inategemea: maelezo ya kiufundi, yanayosema hivyo. Kinachobaki: kanuni za kila mahali pa kazi, zilizoandikwa na kukaguliwa na mtu mwenye sifa. Usimlipe mtu yeyote kutoka WPM.
- **50%** **Ukubwa.** Kwenye kompyuta mpakato moja, kurasa za orodha na mwonekano wa uwezo hazishindwi kwa wafanyakazi 50,000, na maonyesho ya mipango hukataa zaidi ya 10,000 (WPM-T196, WPM-T230). Inategemea: mzunguko mmoja uliopimwa ([BENCHMARKS.md](../../../BENCHMARKS.md)). Kinachobaki: mwonekano wa uwezo bado unakua sawia na idadi ya wafanyakazi, na hakuna kilichopimwa kwenye vifaa vya mwekaji.
- **65%** **Mnyororo wa ugavi.** Kila kitendo cha CI kimebandikwa kwa commit na historia inachanganuliwa kwa siri. Taswira ya kontena imesainiwa na ina asili yake, na hili liliendeshwa kwa toleo v0.1.2 (WPM-T194, WPM-T195). Inategemea: mzunguko huo wa toleo. Kinachobaki: kifurushi cha taswira ni cha faragha, kwa hiyo hakuna aliye nje ya mradi anayeweza kukithibitisha bado.
- **60%** **Ufuatiliaji.** Kila jibu lina kitambulisho cha ombi, na mistari ya kumbukumbu haina mfuatano wa hoja wala kitambulisho cha kibinafsi (WPM-T197). Inategemea: majaribio ya kumbukumbu ya maombi. Kinachobaki: dashibodi na tahadhari ni za mwekaji, na vikomo vya maombi vinahusu mfano mmoja pekee (WPM-T198).
- **60%** **Nakala rudufu na kurejesha.** Nakala rudufu zilizosimbwa, zoezi la kurejesha linaloendeshwa kwenye CI, na kitabu cha uendeshaji. Inategemea: zoezi hilo. Kinachobaki: zoezi la kurejesha kwenye jukwaa la mwekaji mwenyewe ndani ya siku 90 za kalenda zilizopita (kizuizi WPM-G4).
- **40%** **Ukaguzi wa ufikivu.** Ukaguzi wa kiotomatiki dhidi ya WCAG 2.2 AA unaendeshwa kwenye programu na kwenye tovuti hii, katika mandhari ya mwanga na ya giza. Inategemea: majaribio ya Playwright na axe. Kinachobaki: ukaguzi huru (hapa chini); hakuna kisoma skrini kilichotumika.
- **20%** **Tafsiri.** Programu ina lugha na maeneo 17 na muhtasari wa majukumu una 21. Inategemea: maandishi yapo. Kinachobaki: akili bandia iliandika tafsiri, na hakuna aliyezikagua kitaalamu (WPM-T155).

## Ukaguzi unaofanywa na watu wasio mwandishi

- **0%** **Jaribio huru la kuvamia mfumo.** Hakuna jaribio la kuvamia mfumo wa mfano uliosambazwa lililoendeshwa (WPM-T201). Inategemea: kazi iko wazi. Kinachobaki: jaribio, na kurekebisha kitakachopatikana.
- **0%** **Ukaguzi huru wa ufikivu.** Hakuna ukaguzi dhidi ya WCAG 2.2 AA pamoja na watu wanaotumia teknolojia saidizi uliofanyika (WPM-T202). Inategemea: kazi iko wazi. Kinachobaki: ukaguzi, na kurekebisha kitakachopatikana.
- **25%** **Mfano wa vitisho.** Mfano wa awali wa vitisho upo ([threat-model.md](../../../spec/governance/threat-model.md)), ulioandikwa na mwandishi. Inategemea: hati hiyo. Kinachobaki: timu huru iufanye upya kwa usambazaji halisi.

## Sehemu ya mwekaji

- **20%** **Tathmini ya athari kwa ulinzi wa data.** Kifurushi cha kurekebisha kipo: tathmini ya athari, mtiririko wa data, rekodi ya uchakataji na ratiba ya uhifadhi ([spec/governance](../../../spec/governance/dpia.md)). Inategemea: kifurushi. Kinachobaki: hakuna mwekaji, kwa hiyo hakuna afisa wa ulinzi wa data aliyesaini tathmini, msingi halali wa kisheria, ratiba ya uhifadhi wala makubaliano na mchakataji (kizuizi WPM-G5).
- **15%** **Tathmini za athari kwa usawa.** Kiolezo na rejista zipo ([equality-impact](../../../spec/equality-impact/template.md)). Inategemea: kiolezo. Kinachobaki: mwekaji anaandika na kuidhinisha tathmini kwa matumizi yake mwenyewe.
- **40%** **Upangishaji na uendeshaji.** Usanidi wa uzalishaji, taswira ya kontena, na nyaraka za usambazaji na uboreshaji zipo. Inategemea: nyaraka hizo na zoezi la kurejesha. Kinachobaki: hakuna kilichoendeshwa kwenye mwenyeji aliyeimarishwa, na mwekaji lazima aiendeshe huduma.
- **10%** **Msaada.** Kuna kifuatilia masuala na sera ya usalama ([SECURITY.md](../../../SECURITY.md)). Inategemea: hivyo viwili. Kinachobaki: hakuna mkataba wa msaada, kiwango cha huduma, ratiba ya zamu za kuitwa wala shirika nyuma ya WPM.

## Kile ambacho ukurasa huu hauthibitishi

Ukurasa huu hauonyeshi uamuzi wowote wa kuanza kutumika. Uamuzi huo ni wa mwekaji na hufanywa nje ya WPM. Makadirio yanahusu vipengele vilivyoorodheshwa tu, na orodha ya tathmini ya utayari yenyewe ya mambo ambayo haikuyakagua inatumika hapa pia. Pale mfumo wa kliniki unapounganishwa, ukaguzi wa usalama wa kliniki ni uamuzi wa mwekaji.

> Tafsiri hii ilitolewa na akili bandia na haijakaguliwa na mzungumzaji asilia wa Kiswahili. Ikitofautiana na toleo la Kiingereza (`locales/en-001/path-to-production/`), toleo la Kiingereza ndilo linalotangulia.
