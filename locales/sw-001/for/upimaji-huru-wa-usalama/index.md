# Upimaji huru wa usalama

**Mtazamo:** tunawapataje watu ambao hawakuandika programu hii ili waishambulie, tuwaelekeze wapi, na
tufanye nini na watakachogundua?

Hakuna jaribio huru lililofanyika. [Mfano wa vitisho](../../../../spec/governance/threat-model.md) uliandikwa na
msaidizi yuleyule aliyeandika programu na haukukaguliwa na mtu mwingine yeyote, na kila udhibiti unaotajwa ulijaribiwa na
mwandishi huyohuyo. [Tathmini ya utayari](../../../../spec/readiness-assessment.md) inaorodhesha jaribio huru la kuvamia mfumo
kama kizuizi ambacho watu pekee wanaweza kukifunga (WPM-T201, **halijafanyika**). Ukurasa huu ni muhtasari kwa yeyote anayeliagiza. Si
mpango wa majaribio kutoka kwa mjaribu wala hauchukui nafasi ya mpango huo.

## Cha kuwapa wajaribu

| Kipengele | Kilipo | Maelezo |
| --- | --- | --- |
| Usambazaji wa kushambuliwa | Wako: [mwongozo wa usambazaji](../../../../spec/operations/deployment.md) | Mipangilio ya uzalishaji pamoja na kuingia **kumewashwa**, mtoa utambulisho wa majaribio, na **data bandia pekee**. Usijaribu dhidi ya rekodi za watu halisi |
| Msimbo chanzo | Hazina hii | Jaribio lenye msimbo chanzo ni la kina zaidi kuliko la sanduku jeusi; sema unataka lipi |
| Mfano wa vitisho | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Vitisho ishirini na vinne, udhibiti kama ulivyojengwa, na mapengo. Wape **baada ya** raundi ya kwanza ya sanduku jeusi ikiwa unataka kuona wanachogundua bila kuelekezwa |
| Maelezo ya API | `GET /api-docs/openapi.json` | Kila njia, pamoja na `Idempotency-Key` kwenye kila operesheni inayobadilisha kitu |
| Usanifu wa kuingia | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Watumiaji wa mfano, sifa, na kile ambacho sera huamua kwa kitendo pekee |
| Utambulisho wa majaribio | Unaundwa nawe | Angalau: mfanyakazi asiye na sifa, meneja wa moja kwa moja, meneja aliye nje ya mnyororo, rasilimali watu, malipo, msimamizi, na mwenza wa huduma |

## Cha kuwaomba waangalie

Mwandishi hakuweza kuvikagua. Kila safu inataja ingizo la mfano wa vitisho lenyewe, ili matokeo yafuatiliwe.

| Eneo | Cha kujaribu | Tishio |
| --- | --- | --- |
| Tokeni | Kuchanganya algoriti, seti ya funguo inayozunguka maombi yakiwa njiani, tokeni iliyoisha muda au isiyo halali bado, tokeni ya hadhira nyingine | T1, T2 |
| Haja ya kujua | Kila njia ya usomaji kama mpigaji asiye na sifa na kama jirani; njia ambayo jedwali la uainishaji halijui; uandishi ambao sera inaruhusu lakini kidhibiti kinapaswa kukataa | T3, T4, T6 |
| Sifa zenye mamlaka | Kama sifa katika tokeni inaweza kuongezwa, kurudiwa au kurithiwa; nani anaweza kuitoa; msimamizi wa pamoja au wa muda mrefu | T24 |
| Uingizaji wa msimbo | Jaribu kila kigezo, sehemu ya njia na sehemu ya mwili kwa data za nasibu; kauli za ufutaji na uhifadhi hujengwa kutoka vitambulisho vilivyoainishwa kwa aina na zilisomwa, hazikujaribiwa kwa data za nasibu | T12 |
| Kiteja cha kivinjari | Ushughulikiaji wa kipindi na seva ya kiolesura, jinsi inavyobadilisha kipindi kuwa tokeni, uandishi wa hati kati ya tovuti kupitia maandishi yaliyohifadhiwa, kukwepa sera ya usalama wa maudhui, ukurasa wa kidhibiti maudhui unaopakia hati kutoka mtandao wa umma bila toleo lililobandikwa | T13, T14 |
| Maombi ya kwenda nje | Kughushi maombi upande wa seva kupitia URL yoyote iliyosanidiwa (seti za funguo, huduma dada, kache ya pamoja) | T15 |
| Vikomo | Kikomo cha mwili, muda wa kuisha kwa ombi, vikomo vya maombi kwa kila mfano, na maonyesho yanayopakia rekodi za shirika zima kwenye kumbukumbu | T16, T17 |
| Kumbukumbu za matukio | Kama data yoyote ya kibinafsi, tokeni au mfuatano wa hoja unaingia kwenye kumbukumbu | T19 |
| Funguo za urudiaji salama | Ufunguo unaotumiwa na wapigaji wawili; ufunguo uleule wenye mwili, njia au hoja iliyobadilishwa; ufunguo ulio karibu kuwa halali; mwili mkubwa sana; maombi mawili yanayokimbizana chini ya ufunguo mmoja (dai hili limetokana na hoja, halijajaribiwa kwa mbio na jaribio); kama jibu lililohifadhiwa linaweza kusomwa na mtu yeyote isipokuwa mpigaji wake | mpya |
| Kumbukumbu ya ukaguzi | Njia yoyote ya kubadilisha, kuondoa au kuruka ingizo; njia ya kuthibitisha mnyororo | T7 |
| Nakala rudufu | Kurejesha kutoka nakala iliyochezewa au yenye ufunguo usio sahihi; kama mtu aliyefutwa anarudi | T10, T11 |
| Mnyororo wa ugavi | Kubandika vitendo vya ujenzi, kuchanganua siri, kusaini taswira | T20, T21 |

## Kanuni za kukubaliana kabla hawajaanza

Zijaze pamoja na timu zako za usalama na za sheria. Hakuna iliyoamuliwa hapa.

- **Mamlaka:** nani anasaini kuwa jaribio limeidhinishwa, na kwa mifumo na anwani zipi.
- **Kipindi:** tarehe na saa, zikihesabiwa kwa siku za kalenda, na nani wa kumpigia ili kulisimamisha.
- **Wigo:** kilicho ndani (API, kiolesura, mtiririko wa kuingia, hati ya nakala rudufu) na kilicho nje (huduma ya mtoa
  utambulisho yenyewe, mtandao wako, chochote kinachoshirikiwa na mifumo mingine).
- **Data:** bandia pekee. Mjaribu akipata data halisi ya kibinafsi, nani anaambiwa na inashughulikiwaje.
- **Majaribio ya kuharibu:** kama kunyima huduma, uandishi wa wingi na ufutaji vinaruhusiwa, na wapi.
- **Kuripoti:** muundo, kipimo cha ukali, nani anapokea, wanaweza kuihifadhi kwa muda gani, na siri
  wanazopata zinashughulikiwaje.
- **Kujaribu tena:** kwamba mkataba unajumuisha kukagua marekebisho.

## Cha kufanya na matokeo

1. Rekodi kila moja kama kazi katika [tasks.md](../../../../spec/tasks.md) pamoja na rejea ya mjaribu, ushahidi, na
   ukali aliotoa. Matokeo yanayopingwa hubaki yamerekodiwa kama yanayopingwa, pamoja na sababu.
2. Rekebisha kwa jaribio linalotangulia kushindwa. Marekebisho yakiwa kanuni, yaongeze kwenye safu ya mapengo ya mfano wa vitisho na uyafunge
   huko tu pale jaribio la kurudia likifaulu.
3. Usiweke WPM-T201 alama ya kukamilika hadi ripoti iwe mkononi, kila tokeo liwe na mmiliki, na jaribio la kurudia likamilike. Sasisha
   [tathmini ya utayari](../../../../spec/readiness-assessment.md) na orodha ya vizuizi (WPM-T203) katika mabadiliko yaleyale.
4. Iweke ripoti nje ya hazina ikiwa ina chochote kinachoweza kutumiwa vibaya; weka kiungo cha mahali ilipohifadhiwa.

## Ripoti safi inamaanisha nini na haimaanishi nini

Jaribio lisilo na matokeo linaonyesha kwamba wajaribu hawa, katika kipindi hiki, wakiwa na wigo huu na utambulisho huu,
hawakupata chochote.
Halionyeshi kutokuwepo kwa kasoro, halihusu usanidi uliobadilisha baadaye,
na halichukui nafasi ya vizuizi vingine: ukaguzi wa ufikivu, tathmini yako ya athari kwa ulinzi wa data
iliyoidhinishwa, na zoezi la kurejesha kwenye jukwaa lako.

## Ukurasa huu haufanyi nini

Haumtaji msambazaji, haupangi bei wala hauahidi kiwango. Ni mpango upi au uidhinishaji upi sekta yako inatarajia kutoka kwa
mjaribu ni jambo lako kuamua.

> Tafsiri hii ilitolewa na akili bandia na haijakaguliwa na mzungumzaji asilia wa Kiswahili. Ikitofautiana na toleo la Kiingereza (`locales/en-001/for/`), toleo la Kiingereza ndilo linalotangulia.
