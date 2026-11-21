# Kwa Afisa Mkuu wa Usalama wa Habari

**Mtazamo:** inalindwa vipi, tungejuaje ikiwa imeshambuliwa au kutumiwa vibaya, na
nini hakuna aliyekijaribu?

## Unachopata

| Jambo linalokuhusu | Kinachopatikana | Soma |
| --- | --- | --- |
| Vitisho | Mfano wa vitisho: mali, wahusika, mipaka ya uaminifu, udhibiti kama ulivyojengwa, na kile ambacho **hakijatathminiwa** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Uthibitishaji wa utambulisho | Kuingia kumewashwa kwa chaguomsingi; uzalishaji hukataa kuanza ikiwa kumezimwa au hakuna chanzo cha ufunguo wa tokeni; kuingia kupitia saraka ya shirika (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Uidhinishaji | Sera huamua kwa kitendo pekee; jedwali lililoainishwa huamua haja ya kujua kwenye kila njia ya usomaji, na jaribio la kuchanganua chanzo hushindwa kwenye njia isiyoainishwa | [auth.md](../../../../spec/auth.md) |
| Ukaguzi | Kumbukumbu ya kuongeza tu, iliyounganishwa kwa heshi, yenye njia ya kuthibitisha; maingizo hayabebi maandishi ya sababu wala thamani | [audit.md](../../../../spec/audit.md) |
| Usafirishaji na kivinjari | Vichwa vya usalama kwenye kila jibu la API, orodha ruhusa ya wazi ya CORS inayofanya kuanza kushindwa isipowekwa, `no-store` kwenye `/api` | WPM-R73 katika [production-readiness.md](../../../../spec/production-readiness.md) |
| Matumizi mabaya | Vikomo vya maombi kwa kila mpigaji, vikali zaidi kwa njia za kufuta, kuingiza, kusafirisha na tokeni | WPM-R74, faili hilohilo |
| Usanidi usio salama | Kuanza hukataa muunganisho wa hifadhidata usiosimbwa au Redis isiyosimbwa kwenda mwenyeji mwingine, na asili ya CORS inayokosekana | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Data zilizohifadhiwa | Nakala rudufu zilizosimbwa; kache ya pamoja inashikilia utafutaji usiohusu data za kibinafsi pekee na hushindwa kwa kuacha wazi | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Mnyororo wa ugavi | Ukaguzi wa leseni na makatazo kwa `cargo deny`, kreti za pamoja zilizobandikwa kwa toleo, orodha ya vipengele vya programu wakati wa toleo | [production-readiness.md](../../../../spec/production-readiness.md) |
| Matukio | Kitabu cha uendeshaji chenye hatua za kwanza; wewe unaongeza mawasiliano na wajibu wa kutoa taarifa | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Tathmini binafsi | Orodha hakiki ya udhibiti: limetimizwa, limetimizwa kwa sehemu, halijatimizwa, mwekaji kujibu | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Mapengo yanayojulikana, kama yalivyorekodiwa

- Hakuna jaribio huru la kuvamia mfumo lililofanyika. Majaribio yaliandikwa na mwandishi.
- Vitendo vimebandikwa kwa lebo, si kwa heshi, na taswira hazijasainiwa.
- Vikomo vya maombi ni kwa kila mfano.
- Programu hutumia jukumu lolote la hifadhidata ililopewa; hakuna seti ya majukumu yenye haki za chini kabisa iliyotolewa.
- Kubatilisha tokeni na umri wa juu wa tokeni ni jukumu la mtoa utambulisho wako.
- Njia ya kidhibiti maudhui hupakia hati kutoka CDN ya umma bila toleo lililobandikwa wala heshi ya uadilifu.
- Kumbukumbu za matukio hazijakaguliwa kwa data za kibinafsi, na kumbukumbu ya ukaguzi haisafirishwi kwenda mfumo wa ufuatiliaji.
- Redis inayosimamiwa (kundi, sentinel, watumiaji wa udhibiti wa ufikiaji) haijajaribiwa.

## Kile ambacho watu pekee wanaweza kufanya

Kuagiza jaribio huru la kuvamia mfumo, kupitia upya mfano wa vitisho kwa mazingira yako, kuendesha zoezi la kurejesha kwenye
jukwaa lako, na kuweka ulinzi wa matawi kwenye jukwaa la upangishaji. Tazama
[tathmini ya utayari](../../../../spec/readiness-assessment.md), ambayo uamuzi wake ni **haijawa tayari** kwa usambazaji unaohitaji uhakika wa juu.

> Tafsiri hii ilitolewa na akili bandia na haijakaguliwa na mzungumzaji asilia wa Kiswahili. Ikitofautiana na toleo la Kiingereza (`locales/en-001/for/`), toleo la Kiingereza ndilo linalotangulia.
