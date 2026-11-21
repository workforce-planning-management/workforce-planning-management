# Integritetsmeddelande

Det här meddelandet beskriver vad webbplatsen för Arbetskraftsplanering (Workforce Planning Management, WPM) och WPM-applikationen sparar i din
webbläsare, varför, hur länge och hur du tar bort det. Webbplatsen är
<https://workforce-planning-management.github.io/>; applikationen är WPM:s demonstrationsprogramvara, var den
än körs. (Uppdaterad 2026-11-17, WPM-R253.)

Ingen av dem har analysverktyg, reklam eller skript, typsnitt eller bilder från tredje part. Inget som någon av dem sparar i
din webbläsare används för att följa dig från webbplats till webbplats.

## Webbplatsen

Webbplatsen är en samling statiska sidor. Den har ingen inloggning och inga formulär. Det den sparar finns i din egen webbläsare:

| Vad | Var det sparas | Namn | Vad som sparas | Varför |
| --- | --- | --- | --- | --- |
| Ditt tema | Webbläsarens lokala lagring | `mxi.wpm.theme` | Temats namn, till exempel `dark` | Så att nästa sida och ditt nästa besök ser ut som du valde |
| Ditt tema | En kaka (cookie) enbart för den här webbplatsen | `wpm-theme` | Temats namn, till exempel `dark` | Så att temat kommer tillbaka även om din webbläsare behåller kakor men rensar lokal lagring, eller tvärtom |
| Din textstorlek | Webbläsarens lokala lagring | `mxi.wpm.text-size` | Storlekens namn, till exempel `large` | Så att nästa sida och ditt nästa besök använder den textstorlek du valde |
| Ditt språk | Inget sparas | | | Språket är en del av varje sidas adress (till exempel `/cy-001/`), så det finns inget att komma ihåg |
| Var du var på en sida | Webbläsarens sessionslagring, enbart för den här fliken | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Rullningspositioner och i vilken ordning du öppnade sidorna i den här fliken | Så att Bakåt och Framåt tar dig tillbaka dit du var. Ramverket som webbplatsen är byggd med (SvelteKit) sparar dessa |

Inget val sparas förrän du gör det. Tills du väljer följer webbplatsen webbläsarens inställning för ljust eller mörkt läge
och dess normala textstorlek, och sparar varken tema eller textstorlek. Ett val sparas när du gör det i väljarfältet,
och ett nytt val ersätter det. Sessionslagringen skrivs medan du rör dig mellan sidor.

Andra delar av webbplatsen sparar ingenting:

- **Sök** körs i din webbläsare, mot ett index som webbplatsen publicerar, och sparar ingenting. Dina ord läggs i
  sidans adress (`?q=`) så att en resultatsida kan delas; när den adressen läses in från webbhotellet (vid en
  omladdning eller från en delad länk) tar värden emot den tillsammans med begäran, som med vilken adress som helst.
- **Beredskapsreglagen** på vägen till produktion sparar ingenting, och en omladdning återställer
  upphovspersonens uppskattningar.
- **Dela** öppnar den webbplats du väljer (e-post, LinkedIn, Reddit, Bluesky eller Mastodon) först när du väljer den. Den
  webbplatsens eget integritetsmeddelande gäller då.

## WPM-applikationen

Applikationen sparar samma slags val, och kakan som håller dig inloggad:

| Vad | Var det sparas | Namn | Vad som sparas | Varför |
| --- | --- | --- | --- | --- |
| Ditt tema | Webbläsarens lokala lagring | `mxi.wpm.theme` | Temats namn | Så att varje sida ser ut som du valde |
| Din textstorlek | Webbläsarens lokala lagring | `mxi.wpm.text-size` | Storlekens namn | Så att varje sida använder den textstorlek du valde |
| Ditt språk | Webbläsarens lokala lagring | `mxi.wpm.locale` | Språkkoden för den senaste sidan du öppnade, till exempel `cy-001` | Så att en adress utan språk öppnas på ditt |
| Ditt språk | En kaka enbart för applikationen | `wpm-locale` | Samma språkkod | Så att servern kan skicka en adress utan språk till ditt |
| Din inloggning | En kaka enbart för applikationen, som sidans skript inte kan läsa | `__Host-mxi_session` (`mxi_session` på en utvecklares egen dator) | Ett slumpmässigt tal som står för din inloggade session | Så att du förblir inloggad från sida till sida; utan den skulle du logga in på varje sida |
| Innehållsredigerarens språk | Webbläsarens lokala lagring, bara om du öppnar innehållsredigeraren (`/admin`) | `sveltia-cms.prefs` | Redigerarens inställningar, bland annat dess språk | Så att redigeraren öppnas på applikationens språk |
| Var du var på en sida | Webbläsarens sessionslagring, enbart för den här fliken | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Rullningspositioner och i vilken ordning du öppnade sidorna i den här fliken | Så att Bakåt och Framåt tar dig tillbaka dit du var |

Till skillnad från webbplatsen sparar applikationen temat och textstorleken så snart den visar dem, innan du
väljer, och sparar språket för varje sida du öppnar. Nycklar från före projektets namnbyte (`mxi.hcm.theme`, `mxi.hcm.locale`) flyttas till
namnen ovan och raderas.

Inloggningskakan är det enda som står för dig. Det slumpmässiga talet i den betyder ingenting i sig: den
inloggade session det pekar på sparas av applikationens server, och utloggning avslutar den. Vad applikationen
har om de personer som finns i dess register behandlas i det näst sista avsnittet, nedan.

## Vem som tar emot det

Lokal lagring och sessionslagring lämnar aldrig din webbläsare.

En kaka skickas med varje begäran som din webbläsare gör till den webbplats som satte den, och till ingen annan webbplats
(`SameSite=Lax`, och `Secure` över HTTPS):

- Webbplatsens kaka `wpm-theme` går till GitHub Pages, som är värd för webbplatsen. Inget program i projektet
  läser den.
- Applikationens kakor `wpm-locale` och inloggningskakan går till den server som applikationen körs på, som läser dem
  för att välja ditt språk och veta att du är inloggad.

GitHub, som är värd för webbplatsen, tar emot de begäranden som din webbläsare gör, som vilket webbhotell som helst.
[GitHubs integritetsmeddelande](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
säger vad det registrerar. Den organisation som driver applikationen avgör vad dess server registrerar.

## Hur länge det sparas

- Webbplatsens kaka `wpm-theme`: 400 kalenderdagar efter att du senast valde ett tema, den längsta tid en webbläsare behåller en
  kaka.
- Applikationens kaka `wpm-locale`: 365 kalenderdagar efter att du senast öppnade en sida.
- Inloggningskakan: tills du loggar ut eller stänger webbläsaren.
- Lokal lagring: tills du rensar den.
- Sessionslagring: tills du stänger fliken.

## Hur du tar bort det

Rensa webbplatsens data i webbläsarens inställningar (kakor och webbplatsdata), för webbplatsen, applikationen eller
båda. Webbplatsen beter sig då som vid ett första besök; applikationen ber dig logga in igen. Utloggning från
applikationen tar bort inloggningskakan.

## Varför det inte finns någon samtyckesruta

Varje post sparas för att ge dig den tjänst du bett om: webbplatsen eller applikationen som du ställt in den, på
ditt språk och, i applikationen, inloggad. Det är den sorts lagring som reglerna om kakor i Storbritannien och EU (the
Privacy and Electronic Communications Regulations och ePrivacy-direktivet) betraktar som absolut nödvändig, vilket
inte kräver samtycke. Det här är projektets tolkning av reglerna, inte juridisk rådgivning.

## De register som applikationen innehåller

Det här meddelandet gäller det som sparas i din webbläsare. Applikationen innehåller också register om människor (anställda,
sökande och deras kontakter) på sin server. Den som driver applikationen ansvarar för de registren och
skriver integritetsmeddelandet till de personer som finns i dem
([utvärderingschecklistan](../../../spec/governance/assessment-checklist.md), fråga 3.10). Demonstrationsprogramvaran
innehåller endast syntetiska data.

## Frågor

Fråga i projektets arkiv, <https://github.com/workforce-planning-management/workforce-planning-management>,
eller skriv till den underhållare som anges där.

> Den här översättningen har gjorts av AI och har inte granskats av en person med svenska som modersmål. Om den avviker från den engelska versionen (`locales/en-001/privacy/`) gäller den engelska versionen.
