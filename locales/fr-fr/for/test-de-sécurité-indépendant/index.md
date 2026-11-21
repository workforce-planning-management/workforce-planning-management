# Test de sécurité indépendant

**La question que vous vous posez probablement :** comment faire attaquer ce logiciel par des personnes qui ne l’ont pas écrit, vers quoi les orienter, et que faire de ce qu’elles trouvent ?

Aucun test indépendant n’a été fait. Le [modèle de menaces](../../../../spec/governance/threat-model.md) a été écrit par le même assistant que celui qui a écrit le logiciel et personne d’autre ne l’a relu ; chaque mesure qu’il nomme a été mise à l’épreuve par ce même auteur. L’[évaluation de préparation](../../../../spec/readiness-assessment.md) range un test d’intrusion indépendant parmi les verrous que seules des personnes peuvent lever (WPM-T201, **non fait**). Cette page est une note de cadrage pour qui en commande un. Ce n’est pas un plan de test établi par un testeur et elle n’en tient pas lieu.

## Ce qu’il faut remettre aux testeurs

| Élément | Où il se trouve | Remarques |
| --- | --- | --- |
| Un déploiement à attaquer | Le vôtre : [guide de déploiement](../../../../spec/operations/deployment.md) | Paramètres de production avec la connexion **activée**, un fournisseur d’identité de test et **uniquement des données synthétiques**. Ne testez pas sur les dossiers de personnes réelles |
| Le code source | Ce dépôt | Un test avec le code source est plus approfondi qu’un test en boîte noire ; dites lequel vous voulez |
| Le modèle de menaces | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Vingt-quatre menaces, la mesure telle que construite et les écarts. Remettez-le **après** une première passe en boîte noire si vous voulez voir ce qu’ils trouvent sans indice |
| La description de l’API | `GET /api-docs/openapi.json` | Chaque route, avec l’`Idempotency-Key` sur chaque opération qui modifie quelque chose |
| La conception de la connexion | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Personas, attributs, et le fait que la politique décide par action seulement |
| Des identités de test | C’est à vous de les créer | Au moins : une personne sans attribut, un responsable direct, un responsable hors de la chaîne, les RH, la paie, une administration et un pair de service |

## Ce qu’il faut leur demander d’examiner

L’auteur n’a pas pu le vérifier. Chaque ligne nomme l’entrée du modèle de menaces elle-même, afin qu’un constat puisse être retracé.

| Domaine | Ce qu’il faut essayer | Menace |
| --- | --- | --- |
| Jetons | Confusion d’algorithmes ; un jeu de clés qui tourne pendant que des requêtes sont en vol ; un jeton expiré ou pas encore valide ; un jeton destiné à une autre audience | T1, T2 |
| Besoin d’en connaître | Chaque route de lecture en tant qu’appelant sans attribut et en tant que voisin ; une route que la table de classement ne connaît pas ; une écriture que la politique autorise mais que le gestionnaire devrait refuser | T3, T4, T6 |
| Attributs privilégiés | Si un attribut d’un jeton peut être ajouté, rejoué ou hérité ; qui peut le délivrer ; une administration partagée ou de longue durée | T24 |
| Injection | Fuzzing de chaque paramètre, segment de chemin et champ du corps ; les instructions d’effacement et de conservation sont construites à partir d’identifiants typés et ont été lues, non soumises au fuzzing | T12 |
| Le client navigateur | La gestion des sessions par le serveur d’interface, la façon dont il échange une session contre un jeton, le script intersite via un texte stocké, le contournement de la politique de sécurité du contenu et la page du gestionnaire de contenu qui charge un script depuis un réseau public sans version épinglée | T13, T14 |
| Requêtes sortantes | Falsification de requêtes côté serveur via toute URL configurée (jeux de clés, services frères, cache partagé) | T15 |
| Limites | La limite du corps, le délai d’expiration de la requête, les limites de débit par instance et les vues qui chargent en mémoire les données d’une organisation entière | T16, T17 |
| Journaux | Si des données personnelles, un jeton ou une chaîne de requête atteignent un journal | T19 |
| Clés d’idempotence | Une clé utilisée par deux appelants ; la même clé avec un corps, un chemin ou une requête modifiés ; une clé presque valide ; un corps très volumineux ; deux requêtes en concurrence sous une même clé (la revendication est raisonnée, non éprouvée en concurrence) ; si quelqu’un d’autre que son appelant peut lire une réponse stockée | nouveau |
| Le journal d’audit | Tout moyen de modifier, retirer ou sauter une entrée ; le point de vérification de la chaîne | T7 |
| Sauvegardes | Restaurer depuis une sauvegarde altérée ou avec une mauvaise clé ; si une personne effacée revient | T10, T11 |
| Chaîne d’approvisionnement | Épinglage des actions de construction, analyse des secrets, signature des images | T20, T21 |

## Règles à arrêter avant de commencer

Remplissez-les avec vos propres équipes de sécurité et juridique. Aucune n’est décidée ici.

- **Autorité :** qui signe que le test est autorisé, et pour quels systèmes et adresses.
- **Fenêtre :** les dates et heures, comptées en jours calendaires, et qui appeler pour l’arrêter.
- **Périmètre :** ce qui est inclus (l’API, l’interface, le flux de connexion, le script de sauvegarde) et ce qui est exclu (le service du fournisseur d’identité lui-même, votre réseau, tout ce qui est partagé avec d’autres systèmes).
- **Données :** synthétiques uniquement. Si un testeur trouve des données personnelles réelles, qui est prévenu et comment elles sont traitées.
- **Tests destructifs :** si le déni de service, les écritures en masse et l’effacement sont permis, et où.
- **Rapport :** le format, l’échelle de gravité, qui le reçoit, combien de temps il peut être conservé et comment les secrets trouvés sont traités.
- **Nouveau test :** que le contrat inclut la vérification des corrections.

## Que faire des constats

1. Consignez chacun comme une tâche dans [tasks.md](../../../../spec/tasks.md) avec la référence du testeur, la preuve et la gravité qu’il a donnée. Un constat contesté reste consigné comme contesté, avec le motif.
2. Corrigez avec un test qui échoue d’abord. Si la correction est une règle, ajoutez-la à la colonne des écarts du modèle de menaces et ne la fermez là que lorsque le nouveau test passe.
3. Ne marquez pas WPM-T201 comme faite tant que le rapport n’est pas en main, que chaque constat n’a pas de responsable et qu’un nouveau test n’est pas terminé. Mettez à jour l’[évaluation de préparation](../../../../spec/readiness-assessment.md) et la liste des verrous (WPM-T203) dans le même changement.
4. Gardez le rapport hors du dépôt s’il contient quoi que ce soit d’exploitable ; renvoyez vers l’endroit où il est conservé.

## Ce que signifie un rapport propre, et ce qu’il ne signifie pas

Un test sans constat montre seulement que ces testeurs, dans cette fenêtre, avec ce périmètre et ces identités, n’ont rien trouvé. Il ne montre pas l’absence de défauts, ne couvre pas une configuration que vous avez changée ensuite et ne remplace pas les autres verrous : un audit d’accessibilité, votre analyse d’impact relative à la protection des données signée par votre délégué et un exercice de restauration sur votre plateforme.

## Ce que cette page ne fait pas

Elle ne nomme aucun fournisseur, ne fixe aucun prix et ne promet aucune norme. Quel schéma ou quelle accréditation votre secteur attend d’un testeur, c’est à vous d’en décider.

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
