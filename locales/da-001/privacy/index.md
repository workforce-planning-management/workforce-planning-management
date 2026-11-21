# Privatlivspolitik

Denne politik siger, hvad webstedet for Planlægning af arbejdsstyrken (Workforce Planning Management, WPM) og WPM-applikationen gemmer i din
browser, hvorfor, hvor længe, og hvordan du fjerner det. Webstedet er
<https://workforce-planning-management.github.io/>; applikationen er WPM-demonstrationssoftwaren, uanset hvor
den køres. (Opdateret 2026-11-17, WPM-R253.)

Ingen af dem har analyseværktøjer, reklamer eller scripts, skrifttyper eller billeder fra tredjepart. Intet, nogen af dem gemmer i
din browser, bruges til at følge dig fra websted til websted.

## Webstedet

Webstedet er en samling statiske sider. Det har intet login og ingen formularer. Det, det gemmer, ligger i din egen browser:

| Hvad | Hvor det gemmes | Navn | Hvad der gemmes | Hvorfor |
| --- | --- | --- | --- | --- |
| Dit tema | Din browsers lokale lager | `mxi.wpm.theme` | Temaets navn, for eksempel `dark` | Så den næste side og dit næste besøg ser ud, som du valgte |
| Dit tema | En cookie kun til dette websted | `wpm-theme` | Temaets navn, for eksempel `dark` | Så temaet vender tilbage, selv om din browser beholder cookies, men rydder det lokale lager, eller omvendt |
| Din tekststørrelse | Din browsers lokale lager | `mxi.wpm.text-size` | Størrelsens navn, for eksempel `large` | Så den næste side og dit næste besøg bruger den tekststørrelse, du valgte |
| Dit sprog | Intet gemmes | | | Sproget er en del af hver sides adresse (for eksempel `/cy-001/`), så der er intet at huske |
| Hvor du var på en side | Din browsers sessionslager, kun for denne fane | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Rulleposition og rækkefølgen af de sider, du åbnede i denne fane | Så Tilbage og Frem fører dig til det sted, du var. Rammeværket, webstedet er bygget med (SvelteKit), gemmer dem |

Intet valg gemmes, før du træffer det. Indtil du vælger, følger webstedet din browsers lyse eller mørke indstilling
og dens normale tekststørrelse og gemmer intet tema eller nogen tekststørrelse. Et valg gemmes, når du træffer det i vælgerlinjen,
og et nyt valg erstatter det. Sessionslageret skrives, mens du bevæger dig mellem sider.

Andre dele af webstedet gemmer intet:

- **Søgning** kører i din browser, på et indeks, som webstedet udgiver, og gemmer intet. Dine ord sættes i
  sidens adresse (`?q=`), så en resultatside kan deles; når den adresse indlæses fra webværten (ved
  genindlæsning eller fra et delt link), modtager værten den sammen med anmodningen, som den gør med enhver adresse.
- **Skyderne for parathed** på vejen til produktion gemmer intet, og en genindlæsning gendanner forfatterens
  skøn.
- **Del** åbner det websted, du vælger (e-mail, LinkedIn, Reddit, Bluesky eller Mastodon), kun når du vælger det.
  Det websteds egen privatlivspolitik gælder da.

## WPM-applikationen

Applikationen gemmer de samme slags valg og den cookie, der holder dig logget ind:

| Hvad | Hvor det gemmes | Navn | Hvad der gemmes | Hvorfor |
| --- | --- | --- | --- | --- |
| Dit tema | Din browsers lokale lager | `mxi.wpm.theme` | Temaets navn | Så hver side ser ud, som du valgte |
| Din tekststørrelse | Din browsers lokale lager | `mxi.wpm.text-size` | Størrelsens navn | Så hver side bruger den tekststørrelse, du valgte |
| Dit sprog | Din browsers lokale lager | `mxi.wpm.locale` | Sprogkoden for den sidste side, du åbnede, for eksempel `cy-001` | Så en adresse uden sprog åbner på dit |
| Dit sprog | En cookie kun til applikationen | `wpm-locale` | Den samme sprogkode | Så serveren kan sende en adresse uden sprog videre til dit |
| Dit login | En cookie kun til applikationen, som sidens scripts ikke kan læse | `__Host-mxi_session` (`mxi_session` på en udviklers egen computer) | Et tilfældigt tal, der står for din session som logget ind | Så du forbliver logget ind fra side til side; uden den ville du logge ind på hver side |
| Indholdsredaktørens sprog | Din browsers lokale lager, kun hvis du åbner indholdsredaktøren (`/admin`) | `sveltia-cms.prefs` | Redaktørens indstillinger, blandt dem dens sprog | Så redaktøren åbner på applikationens sprog |
| Hvor du var på en side | Din browsers sessionslager, kun for denne fane | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Rulleposition og rækkefølgen af de sider, du åbnede i denne fane | Så Tilbage og Frem fører dig til det sted, du var |

I modsætning til webstedet gemmer applikationen tema og tekststørrelse, så snart den viser dem, før du
vælger, og gemmer sproget for hver side, du åbner. Nøgler fra før projektet fik nyt navn (`mxi.hcm.theme`, `mxi.hcm.locale`) flyttes til
ovenstående navne og slettes.

Login-cookien er det eneste, der står for dig. Det tilfældige tal i den betyder intet i sig selv: den
session som logget ind, den peger på, opbevares af applikationens server, og når du logger ud, ophører den. Det, applikationen
opbevarer om de mennesker, der står i dens poster, er omtalt i det næstsidste afsnit nedenfor.

## Hvem modtager det

Lokalt lager og sessionslager forlader aldrig din browser.

En cookie sendes med hver anmodning, din browser sender til det websted, der satte den, og til intet andet websted
(`SameSite=Lax`, og `Secure` over HTTPS):

- Webstedets `wpm-theme`-cookie går til GitHub Pages, som hoster webstedet. Intet af projektets programmer
  læser den.
- Applikationens `wpm-locale`- og login-cookies går til den server, applikationen kører på, som læser dem
  for at vælge dit sprog og for at vide, at du er logget ind.

GitHub, som hoster webstedet, modtager de anmodninger, din browser sender, som enhver webvært gør.
[GitHubs erklæring om privatliv](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
siger, hvad den registrerer. Den organisation, der driver applikationen, bestemmer, hvad dens server registrerer.

## Hvor længe det gemmes

- Webstedets `wpm-theme`-cookie: 400 kalenderdage efter, du sidst valgte et tema, det længste en browser beholder en
  cookie.
- Applikationens `wpm-locale`-cookie: 365 kalenderdage efter, du sidst åbnede en side.
- Login-cookien: indtil du logger ud eller lukker browseren.
- Lokalt lager: indtil du rydder det.
- Sessionslager: indtil du lukker fanen.

## Sådan fjerner du det

Ryd webstedets data i din browsers indstillinger (cookies og webstedsdata), for webstedet, for applikationen eller
for begge. Webstedet opfører sig da som ved et første besøg; applikationen beder dig logge ind igen. Når du logger ud af
applikationen, fjernes login-cookien.

## Hvorfor der ikke er noget banner om samtykke

Hvert punkt gemmes for at give dig den tjeneste, du bad om: webstedet eller applikationen, som du har indstillet den, på
dit sprog og, i applikationen, som logget ind. Det er den slags lagring, som de britiske og EU's regler om cookies (the
Privacy and Electronic Communications Regulations og e-databeskyttelsesdirektivet) betragter som strengt nødvendig, og som ikke
kræver samtykke. Det er projektets læsning af de regler, ikke juridisk rådgivning.

## De poster, applikationen opbevarer

Denne politik dækker det, der gemmes i din browser. Applikationen opbevarer også poster om mennesker (medarbejdere,
ansøgere og deres kontaktpersoner) på sin server. Den, der driver applikationen, er ansvarlig for de poster og
skriver privatlivspolitikken til de mennesker, der står i dem
([tjeklisten til vurdering](../../../spec/governance/assessment-checklist.md), spørgsmål 3.10). Demonstrationssoftwaren
opbevarer kun syntetiske data.

## Spørgsmål

Spørg i projektets repositorium, <https://github.com/workforce-planning-management/workforce-planning-management>,
eller skriv til den vedligeholder, der er nævnt dér.

> Denne oversættelse er lavet af en AI og er ikke gennemgået af en person med dansk som modersmål. Hvis den afviger fra den engelske version (`locales/en-001/privacy/`), er det den engelske version, der gælder.
