# Pour la direction des risques (CRO)

**La question que vous vous posez probablement :** que pourrait-il arriver de fâcheux avec ce système, qu’est-ce qui l’empêche, et qu’est-ce que personne n’a vérifié ?

## Le point de départ honnête

L’[évaluation de préparation](../../../../spec/readiness-assessment.md) du 2026-10-11 rend le verdict **pas prêt** pour un déploiement à haut niveau de garantie. Elle a trouvé un point bloquant en sondant le service en fonctionnement : toute personne connectée pouvait lire l’arrêt maladie d’un collègue et son motif en texte libre. C’est corrigé et testé. Les verrous que seules des personnes peuvent lever restent ouverts.

## Ce que vous obtenez

| Sujet | Ce qui existe | Lire |
| --- | --- | --- |
| Menaces à chaque frontière | Un modèle de menaces : actifs, acteurs, frontières, le contrôle tel que construit, et ce qui n’a **pas été évalué** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Lectures non autorisées | Une garde de besoin d’en connaître qui classe chaque route de lecture ; un test échoue devant une route non classée | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Altération et responsabilité | Un journal d’audit en ajout seul, chaîné par condensat, avec un point de vérification ; les entrées ne contiennent ni texte de motif ni valeur | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Données sensibles | Aucun diagnostic conservé ; fonctions de catégorie particulière désactivées par défaut ; un seuil pour les petits groupes | [regulatory.md](../../../../spec/regulatory.md) |
| Chemins non sûrs | La production refuse de démarrer avec la connexion désactivée, sans source de clés de jeton, avec une connexion de base de données en clair vers un autre hôte ou avec un Redis distant en clair | [production-readiness.md](../../../../spec/production-readiness.md) |
| Perte de données | Sauvegardes chiffrées, un exercice de restauration exécuté en CI, un registre d’effacement | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Biais dans les scores | Un modèle d’analyse d’impact sur l’égalité et un registre de chaque score avec un projet d’évaluation | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Une auto-évaluation des contrôles | Une liste de contrôle où chaque réponse est satisfaite, partiellement satisfaite, non satisfaite ou à fournir par le déployeur | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Risques ouverts, tels que consignés

L’épinglage et la signature de la chaîne d’approvisionnement, les preuves de charge, les contrôles des journaux pour les données personnelles, les limites de débit partagées entre instances, un jeu de rôles de base de données à moindre privilège et des consignes sur la révocation des jetons sont des points ouverts de l’évaluation de préparation, chacun avec une tâche. Les tests ont été écrits par l’auteur ; aucun n’est une revue indépendante.

## Ce que seules des personnes peuvent faire

Un test d’intrusion indépendant, un audit d’accessibilité indépendant, l’analyse d’impact du déployeur signée, un exercice de restauration sur votre plateforme, et l’approbation des analyses d’impact sur l’égalité. Le dépôt les déclare comme des verrous et ne les marque pas comme faits.

Voir aussi la [direction des systèmes d’information](../direction-des-systèmes-dinformation/index.md) et la [direction juridique](../direction-juridique/index.md).

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
