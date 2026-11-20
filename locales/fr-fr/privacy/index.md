# Avis de confidentialité

Cet avis dit ce que le site web de Workforce Planning Management (WPM) et l’application WPM conservent dans votre navigateur, pourquoi, pendant combien de temps, et comment le supprimer. Le site web est <https://workforce-planning-management.github.io/> ; l’application est le logiciel de démonstration WPM, où qu’il soit exécuté. (Mis à jour le 2026-11-17, WPM-R253.)

Ni l’un ni l’autre n’a de mesure d’audience, de publicité, ni de scripts, polices ou images de tiers. Rien de ce que l’un ou l’autre conserve dans votre navigateur ne sert à vous suivre d’un site à l’autre.

## Le site web

Le site web est un ensemble de pages statiques. Il n’a ni connexion ni formulaire. Ce qu’il conserve se trouve dans votre propre navigateur :

| Quoi | Où c’est conservé | Nom | Ce qui est conservé | Pourquoi |
| --- | --- | --- | --- | --- |
| Votre thème | Le stockage local de votre navigateur | `mxi.wpm.theme` | Le nom du thème, par exemple `dark` | Pour que la page suivante et votre prochaine visite aient l’aspect que vous avez choisi |
| Votre thème | Un cookie pour ce site web uniquement | `wpm-theme` | Le nom du thème, par exemple `dark` | Pour que le thème revienne même si votre navigateur garde les cookies mais efface le stockage local, ou l’inverse |
| Votre taille de texte | Le stockage local de votre navigateur | `mxi.wpm.text-size` | Le nom de la taille, par exemple `large` | Pour que la page suivante et votre prochaine visite utilisent la taille de texte que vous avez choisie |
| Votre langue | Rien n’est conservé | | | La langue fait partie de l’adresse de chaque page (par exemple `/cy-001/`), il n’y a donc rien à retenir |
| Où vous étiez sur une page | Le stockage de session de votre navigateur, pour cet onglet uniquement | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Les positions de défilement, et l’ordre des pages ouvertes dans cet onglet | Pour que Précédent et Suivant vous ramènent là où vous étiez. Le framework avec lequel le site web est construit (SvelteKit) conserve ces éléments |

Aucun choix n’est conservé avant que vous le fassiez. Tant que vous n’avez pas choisi, le site web suit le réglage clair ou sombre de votre navigateur et sa taille de texte normale, et ne conserve ni thème ni taille de texte. Un choix est conservé quand vous le faites dans la barre de sélection, et un nouveau choix le remplace. Le stockage de session est écrit à mesure que vous passez d’une page à l’autre.

Les autres parties du site web ne conservent rien :

- **La recherche** s’exécute dans votre navigateur, sur un index que le site web publie, et ne conserve rien. Vos mots sont placés dans l’adresse de la page (`?q=`) pour qu’une page de résultats puisse être partagée ; quand cette adresse est chargée depuis l’hébergeur web (lors d’un rechargement, ou depuis un lien partagé), l’hébergeur la reçoit avec la requête, comme toute adresse.
- **Les curseurs de préparation** du chemin vers la production ne conservent rien, et un rechargement rétablit les estimations de l’auteur.
- **Partager** ouvre le site que vous choisissez (courriel, LinkedIn, Reddit, Bluesky ou Mastodon) seulement quand vous le choisissez. C’est alors l’avis de confidentialité de ce site qui s’applique.

## L’application WPM

L’application conserve les mêmes types de choix, ainsi que le cookie qui vous garde connecté :

| Quoi | Où c’est conservé | Nom | Ce qui est conservé | Pourquoi |
| --- | --- | --- | --- | --- |
| Votre thème | Le stockage local de votre navigateur | `mxi.wpm.theme` | Le nom du thème | Pour que chaque page ait l’aspect que vous avez choisi |
| Votre taille de texte | Le stockage local de votre navigateur | `mxi.wpm.text-size` | Le nom de la taille | Pour que chaque page utilise la taille de texte que vous avez choisie |
| Votre langue | Le stockage local de votre navigateur | `mxi.wpm.locale` | Le code de langue de la dernière page ouverte, par exemple `cy-001` | Pour qu’une adresse sans langue s’ouvre dans la vôtre |
| Votre langue | Un cookie pour l’application uniquement | `wpm-locale` | Le même code de langue | Pour que le serveur puisse diriger une adresse sans langue vers la vôtre |
| Votre connexion | Un cookie pour l’application uniquement, que les scripts de la page ne peuvent pas lire | `__Host-mxi_session` (`mxi_session` sur l’ordinateur personnel d’un développeur) | Un nombre aléatoire qui représente votre session connectée | Pour que vous restiez connecté d’une page à l’autre ; sans lui, vous devriez vous connecter sur chaque page |
| La langue de l’éditeur de contenu | Le stockage local de votre navigateur, seulement si vous ouvrez l’éditeur de contenu (`/admin`) | `sveltia-cms.prefs` | Les réglages de l’éditeur, dont sa langue | Pour que l’éditeur s’ouvre dans la langue de l’application |
| Où vous étiez sur une page | Le stockage de session de votre navigateur, pour cet onglet uniquement | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Les positions de défilement, et l’ordre des pages ouvertes dans cet onglet | Pour que Précédent et Suivant vous ramènent là où vous étiez |

Contrairement au site web, l’application conserve le thème et la taille de texte dès qu’elle les affiche, avant que vous choisissiez, et conserve la langue de chaque page que vous ouvrez. Les clés antérieures au changement de nom du projet (`mxi.hcm.theme`, `mxi.hcm.locale`) sont déplacées vers les noms ci-dessus, puis supprimées.

Le cookie de connexion est le seul élément qui vous représente. Le nombre aléatoire qu’il contient ne signifie rien en lui-même : la session connectée à laquelle il renvoie est conservée par le serveur de l’application, et la déconnexion y met fin. Ce que l’application détient sur les personnes de ses dossiers est traité dans l’avant-dernière section, ci-dessous.

## Qui le reçoit

Le stockage local et le stockage de session ne quittent jamais votre navigateur.

Un cookie est envoyé avec chaque requête que votre navigateur fait au site qui l’a déposé, et à aucun autre site (`SameSite=Lax`, et `Secure` en HTTPS) :

- Le cookie `wpm-theme` du site web va à GitHub Pages, qui héberge le site web. Aucun programme du projet ne le lit.
- Le cookie `wpm-locale` et le cookie de connexion de l’application vont au serveur sur lequel l’application s’exécute, qui les lit pour choisir votre langue et savoir que vous êtes connecté.

GitHub, qui héberge le site web, reçoit les requêtes de votre navigateur, comme tout hébergeur web. [La déclaration de confidentialité de GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement) dit ce que GitHub enregistre. L’organisation qui exploite l’application décide de ce que son serveur enregistre.

## Combien de temps c’est conservé

- Le cookie `wpm-theme` du site web : 400 jours calendaires après le dernier choix d’un thème, la durée maximale pendant laquelle un navigateur garde un cookie.
- Le cookie `wpm-locale` de l’application : 365 jours calendaires après la dernière page ouverte.
- Le cookie de connexion : jusqu’à ce que vous vous déconnectiez ou fermiez le navigateur.
- Le stockage local : jusqu’à ce que vous l’effaciez.
- Le stockage de session : jusqu’à ce que vous fermiez l’onglet.

## Comment le supprimer

Effacez les données du site dans les réglages de votre navigateur (cookies et données de site), pour le site web, l’application, ou les deux. Le site web se comporte alors comme lors d’une première visite ; l’application vous demande de vous reconnecter. Se déconnecter de l’application supprime le cookie de connexion.

## Pourquoi il n’y a pas de bandeau de consentement

Chaque élément est conservé pour vous fournir le service que vous avez demandé : le site web ou l’application tels que vous les avez réglés, dans votre langue et, dans l’application, en restant connecté. C’est le type de stockage que les règles du Royaume-Uni et de l’UE sur les cookies (les Privacy and Electronic Communications Regulations britanniques, et la directive vie privée et communications électroniques) considèrent comme strictement nécessaire, et qui ne demande pas de consentement. C’est la lecture que fait le projet de ces règles, pas un conseil juridique.

## Les dossiers que détient l’application

Cet avis couvre ce qui est conservé dans votre navigateur. L’application détient aussi sur son serveur des dossiers sur des personnes (salariés, candidats et leurs contacts). Celui qui exploite l’application est responsable de ces dossiers et rédige l’avis de confidentialité destiné aux personnes qui y figurent ([la liste de contrôle d’évaluation](../../../spec/governance/assessment-checklist.md), en anglais, question 3.10). Le logiciel de démonstration ne contient que des données synthétiques.

## Questions

Posez vos questions dans le dépôt du projet, <https://github.com/workforce-planning-management/workforce-planning-management>, ou écrivez au mainteneur qui y est nommé.
