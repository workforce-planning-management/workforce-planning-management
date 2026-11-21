# Privacyverklaring

Deze verklaring beschrijft wat de website van Personeelsplanning (Workforce Planning Management, WPM) en de WPM-applicatie in uw browser opslaan, waarom, hoe lang en hoe u het verwijdert. De website is <https://workforce-planning-management.github.io/>; de applicatie is de demonstratiesoftware van WPM, waar zij ook draait. (Bijgewerkt 2026-11-17, WPM-R253.)

Geen van beide heeft analysehulpmiddelen, advertenties, scripts, lettertypen of afbeeldingen van derden. Niets wat een van beide in uw browser opslaat, wordt gebruikt om u van website naar website te volgen.

## De website

De website is een verzameling statische pagina's. Zij heeft geen aanmelding en geen formulieren. Wat zij opslaat, staat in uw eigen browser:

| Wat | Waar het wordt opgeslagen | Naam | Wat er wordt opgeslagen | Waarom |
| --- | --- | --- | --- | --- |
| Uw thema | Lokale opslag van de browser | `mxi.wpm.theme` | De naam van het thema, bijvoorbeeld `dark` | Zodat de volgende pagina en uw volgende bezoek eruitzien zoals u koos |
| Uw thema | Een cookie die alleen voor deze website geldt | `wpm-theme` | De naam van het thema, bijvoorbeeld `dark` | Zodat het thema terugkomt ook als uw browser cookies bewaart maar de lokale opslag wist, of andersom |
| Uw tekstgrootte | Lokale opslag van de browser | `mxi.wpm.text-size` | De naam van de grootte, bijvoorbeeld `large` | Zodat de volgende pagina en uw volgende bezoek de tekstgrootte gebruiken die u koos |
| Uw taal | Er wordt niets opgeslagen | | | De taal maakt deel uit van het adres van elke pagina (bijvoorbeeld `/cy-001/`), dus er is niets te onthouden |
| Waar u op een pagina was | Sessieopslag van de browser, alleen voor dit tabblad | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Schuifposities en de volgorde waarin u de pagina's in dit tabblad opende | Zodat Terug en Vooruit u terugbrengen naar waar u was. Het framework waarmee de website is gebouwd (SvelteKit) slaat deze op |

Er wordt niets opgeslagen voordat u een keuze maakt. Totdat u kiest, volgt de website de instelling van uw browser voor licht of donker en de normale tekstgrootte, en slaat zij geen thema of tekstgrootte op. Een keuze wordt opgeslagen wanneer u die maakt in de keuzebalk, en een nieuwe keuze vervangt haar. De sessieopslag wordt beschreven terwijl u tussen pagina's navigeert.

Andere onderdelen van de website slaan niets op:

- **Zoeken** draait in uw browser, tegen een index die de website publiceert, en slaat niets op. Uw woorden komen in het adres van de pagina (`?q=`) zodat een resultatenpagina kan worden gedeeld; wanneer dat adres vanaf de webhost wordt geladen (bij opnieuw laden of via een gedeelde link) ontvangt de host het samen met het verzoek, zoals bij elk adres.
- **De gereedheidsschuifjes** op de weg naar productie slaan niets op, en opnieuw laden herstelt de schattingen van de auteur.
- **Delen** opent de website van uw keuze (e-mail, LinkedIn, Reddit, Bluesky of Mastodon) pas wanneer u die kiest. Dan geldt de eigen privacyverklaring van die website.

## De WPM-applicatie

De applicatie slaat dezelfde soort keuzes op, en de cookie die u aangemeld houdt:

| Wat | Waar het wordt opgeslagen | Naam | Wat er wordt opgeslagen | Waarom |
| --- | --- | --- | --- | --- |
| Uw thema | Lokale opslag van de browser | `mxi.wpm.theme` | De naam van het thema | Zodat elke pagina eruitziet zoals u koos |
| Uw tekstgrootte | Lokale opslag van de browser | `mxi.wpm.text-size` | De naam van de grootte | Zodat elke pagina de tekstgrootte gebruikt die u koos |
| Uw taal | Lokale opslag van de browser | `mxi.wpm.locale` | De taalcode van de laatste pagina die u opende, bijvoorbeeld `cy-001` | Zodat een adres zonder taal in uw taal opent |
| Uw taal | Een cookie die alleen voor de applicatie geldt | `wpm-locale` | Dezelfde taalcode | Zodat de server een adres zonder taal naar uw taal kan doorsturen |
| Uw aanmelding | Een cookie die alleen voor de applicatie geldt en die de scripts van de pagina niet kunnen lezen | `__Host-mxi_session` (`mxi_session` op de eigen computer van een ontwikkelaar) | Een willekeurig getal dat uw aangemelde sessie vertegenwoordigt | Zodat u van pagina naar pagina aangemeld blijft; zonder deze zou u op elke pagina moeten aanmelden |
| Taal van de inhoudsbewerker | Lokale opslag van de browser, alleen als u de inhoudsbewerker opent (`/admin`) | `sveltia-cms.prefs` | De instellingen van de bewerker, onder meer de taal | Zodat de bewerker opent in de taal van de applicatie |
| Waar u op een pagina was | Sessieopslag van de browser, alleen voor dit tabblad | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Schuifposities en de volgorde waarin u de pagina's in dit tabblad opende | Zodat Terug en Vooruit u terugbrengen naar waar u was |

Anders dan de website slaat de applicatie het thema en de tekstgrootte op zodra zij die toont, vóór u kiest, en slaat zij de taal op van elke pagina die u opent. Sleutels van vóór de naamswijziging van het project (`mxi.hcm.theme`, `mxi.hcm.locale`) worden verplaatst naar de bovenstaande namen en gewist.

De aanmeldcookie is het enige dat voor u staat. Het willekeurige getal erin betekent op zichzelf niets: de aangemelde sessie waarnaar het wijst, wordt opgeslagen door de server van de applicatie en afmelden beëindigt haar. Wat de applicatie bewaart over de personen in haar dossiers, wordt behandeld in de voorlaatste sectie hieronder.

## Wie het ontvangt

Lokale opslag en sessieopslag verlaten uw browser nooit.

Een cookie wordt meegestuurd met elk verzoek dat uw browser doet aan de website die haar plaatste, en aan geen enkele andere website (`SameSite=Lax`, en `Secure` via HTTPS):

- De cookie `wpm-theme` van de website gaat naar GitHub Pages, dat de website host. Geen programma van het project leest haar.
- De cookies `wpm-locale` en de aanmeldcookie van de applicatie gaan naar de server waarop de applicatie draait, die ze leest om uw taal te kiezen en te weten dat u bent aangemeld.

GitHub, dat de website host, ontvangt de verzoeken die uw browser doet, zoals elke webhost. De [privacyverklaring van GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) zegt wat het registreert. De organisatie die de applicatie beheert, bepaalt wat haar server registreert.

## Hoe lang het wordt bewaard

- De cookie `wpm-theme` van de website: 400 kalenderdagen nadat u voor het laatst een thema koos, de langste tijd die een browser een cookie bewaart.
- De cookie `wpm-locale` van de applicatie: 365 kalenderdagen nadat u voor het laatst een pagina opende.
- De aanmeldcookie: totdat u zich afmeldt of de browser sluit.
- Lokale opslag: totdat u haar wist.
- Sessieopslag: totdat u het tabblad sluit.

## Hoe u het verwijdert

Wis de gegevens van de website in de instellingen van uw browser (cookies en websitegegevens), voor de website, de applicatie of beide. De website gedraagt zich dan als bij een eerste bezoek; de applicatie vraagt u opnieuw aan te melden. Afmelden bij de applicatie verwijdert de aanmeldcookie.

## Waarom er geen toestemmingsvenster is

Elk item wordt opgeslagen om u de dienst te geven waarom u vroeg: de website of de applicatie zoals u haar instelde, in uw taal en, in de applicatie, aangemeld. Dat is het soort opslag dat de regels over cookies in het Verenigd Koninkrijk en de EU (de Privacy and Electronic Communications Regulations en de e-privacyrichtlijn) als strikt noodzakelijk beschouwen, waarvoor geen toestemming nodig is. Dit is de uitleg van het project van de regels, geen juridisch advies.

## De dossiers die de applicatie bevat

Deze verklaring gaat over wat in uw browser wordt opgeslagen. De applicatie bevat ook dossiers over mensen (medewerkers, sollicitanten en hun contactpersonen) op haar server. Wie de applicatie beheert, is verantwoordelijk voor die dossiers en schrijft de privacyverklaring voor de personen die erin staan ([de beoordelingschecklist](../../../spec/governance/assessment-checklist.md), vraag 3.10). De demonstratiesoftware bevat uitsluitend synthetische gegevens.

## Vragen

Stel ze in de repository van het project, <https://github.com/workforce-planning-management/workforce-planning-management>, of schrijf de beheerder die daar wordt genoemd.

> Deze vertaling is door AI gemaakt en niet beoordeeld door een moedertaalspreker van het Nederlands. Wijkt zij af van de Engelse versie (`locales/en-001/privacy/`), dan geldt de Engelse versie.
