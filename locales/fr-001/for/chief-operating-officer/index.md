# Pour la direction des opérations (COO)

**La question que vous vous posez probablement :** avons-nous les bonnes personnes, au bon endroit, au bon moment, et qu’est-ce qui casse si quelqu’un est absent ?

## Ce que vous obtenez

| Vous voulez savoir | Où WPM répond | Lire |
| --- | --- | --- |
| Si une équipe peut couvrir une absence | Des suppléants par personne, le tableau d’astreinte avec échanges et rappels, et une vue de couverture qui dit clairement quand personne n’est disponible | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Temps, présence et congés | Saisies de temps, demandes de congé avec circuit d’approbation, droits acquis et plannings | [workforce-management.md](../../../../spec/workforce-management.md) |
| Si les limites de temps de travail sont respectées | Garde-fous consultatifs sur le temps de travail ; ils avertissent et ne bloquent pas | WPM-R27, même fichier |
| La capacité par compétence, mois par mois | Viviers de compétences, offre et demande des programmes, engagements des partenaires, le vivier contraignant, et un contrôle de démarrage qui est une suggestion, avec des décisions de démarrage consignées | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Ce dont nous aurons besoin et où est l’écart | Plans d’effectifs, prévision et analyse des écarts par service et par rôle | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Quels contrats se terminent bientôt | Une date de fin selon la base de la mission, des prolongations avec historique, des rappels avant la fin, une liste des missions sans date de fin et des plans de conversion | [engagements.md](../../../../spec/engagements.md) |
| Où le personnel occasionnel pèse lourd | La part des ETP qui sont prestataires et celle à durée déterminée, séparément, et le financement qui prend fin mois par mois | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Arrivées et départs | Listes de contrôle datées et une passation le dernier jour | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Écarts de compétences et temps de formation | Analyse des écarts avec les éléments sur lesquels repose une recommandation | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Comment lire la vue de capacité

La capacité est planifiée par vivier de compétences, pas par personne. Chaque mois d’un vivier aboutit à l’un de quatre résultats, et une compétence non déclarée est *inconnue*, pas un manque. Le contrôle de démarrage est une suggestion, et une personne consigne la décision de démarrer. La planification par chaîne critique et chemin critique n’est pas ici : elle relève de project-portfolio-management.

## Limites

- La part occasionnelle n’a pas encore de tendance : il faut des instantanés d’effectifs répartis par base (WPM-T139, non construit).
- Dans la vue du personnel occasionnel, un vivier est le service, tant que les plans ne définissent pas de viviers.
- La planification des équipes et les calculs d’impôt sur la paie ne remplacent pas vos systèmes de planning ou de paie.
- Plusieurs vues chargent en mémoire les données d’une organisation ; aucun chiffre de charge n’a été mesuré ([readiness-assessment.md](../../../../spec/readiness-assessment.md), écart 8).

Voir aussi la [direction des ressources humaines](../chief-people-officer/index.md) et la [direction générale](../chief-executive-officer/index.md).

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
