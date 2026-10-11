# Pour la direction des ressources humaines (CPO)

**La question que vous vous posez probablement :** cela couvre-t-il tout le cycle de vie des salariés, et puis-je l’utiliser sans détenir sur les personnes plus que je ne le dois ?

## Ce que vous obtenez

| Étape | Ce que WPM conserve | Lire |
| --- | --- | --- |
| Recrutement | Demandes de poste, un flux de candidatures, un vivier de candidats, listes de contrôle d’intégration | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| Le dossier | Un dossier d’emploi par personne, l’organigramme, les liens hiérarchiques, les groupes, les mutations entre organisations | [hr-core.md](../../../../spec/hr-core.md) |
| Libre-service | La personne voit et met à jour ses propres informations, demande un travail flexible, donne son préavis et voit ses congés | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Vie professionnelle | Congés, temps, plannings, avantages, rappels de bien-être, aménagements raisonnables, évaluations ergonomiques, le sondage anonyme | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Développement | Entretiens, évaluations à 360 degrés, formations via le registre des cours, évaluations, mentorat, plans de développement, succession, premières carrières | [talent-development.md](../../../../spec/talent-development.md) |
| Rémunération | Campagnes de paie, bulletins, comparaisons salariales, consultation d’une grille salariale nationale, niveaux de poste | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Personnel non permanent | Dates de fin par base, prolongations, évaluations de statut, plans de conversion décidés par une personne, financement à côté de la fin du contrat | [engagements.md](../../../../spec/engagements.md) |
| Départ | Démission avec préavis, passation le dernier jour, conservation par type de dossier, effacement | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Comment il protège les personnes

- **Besoin d’en connaître à la lecture.** Un responsable voit ses collaborateurs directs, une personne se voit elle-même, les RH voient ce dont elles ont besoin. Un inconnu connecté ne voit rien d’un collègue ([auth.md](../../../../spec/auth.md)).
- **Aucun diagnostic n’est conservé.** L’arrêt maladie enregistre un type, pas un motif.
- **Les fonctions de catégorie particulière sont désactivées par défaut.** Le suivi de l’égalité et les exigences de santé au travail demandent une base légale consignée, sont volontaires ou détenues par le rôle de santé au travail, et ne montrent que les groupes au-dessus d’un seuil. Elles n’alimentent jamais un score ni une décision concernant une personne.
- **Les scores conseillent.** La préparation à la succession, les écarts de compétences et les prévisions suggèrent ; une personne décide, et des décisions comme la conversion d’un prestataire sont consignées comme la décision d’une personne, jamais comme un résultat automatique.
- **La personne concernée peut consulter et faire effacer.** L’export d’accès et l’effacement couvrent les dossiers ci-dessus, avec un registre pour qu’une restauration ne ramène pas une personne effacée.

## Limites

- Pas encore construits : une interface pour les plans de conversion et le financement, la tendance du personnel occasionnel, et la page « Mes informations » du salarié. Voir [tasks.md](../../../../spec/tasks.md) pour l’état actuel.
- Les traductions sont assistées par machine ; une relecture professionnelle est un verrou ouvert.
- Les accords collectifs et les organes reconnus ne sont que recherche et proposition ([worker-unions.md](../../../../spec/worker-unions.md)).
- L’impôt sur la paie n’est qu’une ébauche.

Voir aussi la [direction juridique](../chief-legal-officer/index.md) et la [direction générale](../chief-executive-officer/index.md).

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
