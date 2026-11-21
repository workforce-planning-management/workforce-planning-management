# Kwa Afisa Mkuu wa Habari

**Mtazamo:** hii inaingiaje katika mazingira yetu ya TEHAMA, nani anaweza kuingia, data huishi wapi,
na tunaiendeshaje na kuirejesha?

## Unachopata

| Jambo linalokuhusu | Kinachopatikana | Soma |
| --- | --- | --- |
| Umbo | API ya JSON na kiteja tofauti cha kivinjari; PostgreSQL huhifadhi data | [architecture.md](../../../../spec/architecture.md) |
| Utambulisho | WPM haihifadhi utambulisho. Mtu, mfanyakazi na shirika ni marejeo ya huduma nyingine, yanayotatuliwa kwa jina pekee na kuhifadhiwa kwa muda | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Kuingia | Imewashwa kwa chaguomsingi. Kuingia kupitia saraka ya shirika (OIDC, inayooana na Entra, moja kwa moja au kupitia Keycloak); majukumu na sifa huamua ufikiaji | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Usambazaji | Ujenzi wa kontena na faili ya compose kwa onyesho; kwa uzalishaji, seva mbadala inayokatisha TLS, mipangilio inayohitajika, na ukaguzi wa kuanza unaokataa mipangilio isiyo salama | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Kache ya pamoja | Redis ya hiari kwa utafutaji usiohusu data za kibinafsi pekee; kila simu hushindwa kwa kuacha wazi | `REDIS_URL` katika mwongozo wa usambazaji (WPM-D78) |
| Hifadhi nakala na urejeshaji | Nakala rudufu zilizosimbwa, hati ya kurejesha, na zoezi la kurejesha ambalo CI huliendesha | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Kuboresha toleo | Uhamishaji wa hifadhidata na mwongozo wa kuboresha | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Mambo yakiharibika | Kitabu cha uendeshaji chenye hatua za kwanza | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Mwendo wa data | Hati ya mtiririko wa data | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Mambo ya kujua kabla ya kupanga

- Kiolesura cha mtumiaji **hakijawekwa kwenye kontena**; unachagua kiunganishi cha jukwaa lako.
- Vikomo vya maombi ni kwa kila mfano. Hifadhi ya pamoja ni yako kuitoa.
- Programu hutumia jukumu lolote la hifadhidata linalotajwa na `DATABASE_URL`; seti ya majukumu yenye haki za chini kabisa bado haijatolewa.
- Kubatilisha tokeni na umri wa juu wa tokeni ni jukumu la mtoa utambulisho wako.
- Redis inayosimamiwa (kundi, sentinel, watumiaji wa udhibiti wa ufikiaji) haijajaribiwa.
- Hakuna kipimo cha mzigo kilichopo, na baadhi ya maonyesho husoma shirika zima kwenye kumbukumbu.
- Ujenzi wa taswira ya kontena na faili ya compose viliandikwa lakini vijaribiwe kwanza katika mazingira yako.

> Tafsiri hii ilitolewa na akili bandia na haijakaguliwa na mzungumzaji asilia wa Kiswahili. Ikitofautiana na toleo la Kiingereza (`locales/en-001/for/`), toleo la Kiingereza ndilo linalotangulia.
