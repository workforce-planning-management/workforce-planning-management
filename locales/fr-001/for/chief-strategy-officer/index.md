# Pour la direction de la stratégie

**La question que vous vous posez probablement :** nos effectifs sont-ils façonnés pour servir la stratégie, de quoi aurons-nous besoin, et que faire de l’écart ?

## Ce que vous obtenez

| Vous voulez savoir | Où WPM répond | Lire |
| --- | --- | --- |
| Ce qu’un rôle exige | Des profils de rôle avec les compétences requises et un niveau minimal, pour que l’écart se mesure par rapport à un rôle et non à une personne | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| De quoi nous aurons besoin | Des plans d’effectifs comme mondes brouillons, plusieurs côte à côte, avec des hypothèses énoncées et la demande par service, rôle et période | WPM-R36, même fichier |
| Où est l’écart | Offre projetée contre demande planifiée : un écart en effectifs et un écart en compétences, avec des leviers listés comme suggestions et leurs éléments de preuve | WPM-R37 |
| Si cela sert la stratégie | Des objectifs liés à la demande : objectifs sans ressources, demande non alignée, rôles critiques sans relève | WPM-R38 |
| Si nous pouvons livrer un programme | Viviers de compétences, demande des programmes, engagements des partenaires, le vivier contraignant, un contrôle de démarrage qui est une suggestion | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| À quel point nous dépendons de personnel non permanent | La part des ETP qui sont prestataires et celle à durée déterminée, séparément, et la part de financement qui prend fin chacun des douze prochains mois | `GET /api/workforce-intelligence/contingent` (WPM-T109, partiel) |
| Ce que nous prévoyons quand les contrats se terminent | Plans de conversion : la décision d’une personne de convertir, prolonger ou terminer, avec le financement à côté de la fin du contrat | [engagements.md](../../../../spec/engagements.md) |
| Ce qui est prévu d’autre | La feuille de route et la liste des tâches ouvertes | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Comment lire une prévision

- C’est une règle énoncée appliquée à des entrées énoncées, pas un modèle, et chaque résultat nomme ses hypothèses.
- Avec trop peu d’historique d’effectifs, elle dit `insufficient_history` au lieu d’extrapoler.
- Un plan ne détient que des effectifs hypothétiques. Il ne lit ni ne modifie jamais le dossier d’une personne.
- Les leviers sont des suggestions. Le planificateur choisit, et le logiciel ne décide de rien.

## Limites

- Le levier « convertir le personnel occasionnel en permanent » dans les plans est spécifié (WPM-R63) et pas encore dans le code.
- La part occasionnelle n’a pas encore de tendance : il faut des instantanés répartis par base (WPM-T139, non construit).
- La planification par chaîne critique et chemin critique relève de project-portfolio-management, pas d’ici.
- Il n’existe pas encore d’interface pour le financement ni pour les plans de conversion.

Voir aussi les pages de la [direction générale](../chief-executive-officer/index.md) et de la [direction des opérations](../chief-operating-officer/index.md).

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
