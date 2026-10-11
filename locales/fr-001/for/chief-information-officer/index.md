# Pour la direction des systèmes d’information (CIO)

**La question que vous vous posez probablement :** comment cela s’insère-t-il dans notre parc, qui peut se connecter, où vivent les données, et comment l’exploiter et le rétablir ?

## Ce que vous obtenez

| Sujet | Ce qui existe | Lire |
| --- | --- | --- |
| Forme | Une API JSON et un client navigateur distinct ; PostgreSQL conserve les données | [architecture.md](../../../../spec/architecture.md) |
| Identité | WPM ne conserve aucune identité. Une personne, un travailleur et une organisation sont des références à d’autres services, résolues par le nom seulement et mises en cache | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Connexion | Activée par défaut. Connexion par annuaire d’entreprise (OIDC, compatible Entra, directement ou via Keycloak) ; les rôles et attributs décident de l’accès | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Déploiement | Une construction de conteneur et un fichier compose pour une démonstration ; pour la production, un proxy qui termine TLS, des paramètres obligatoires et des contrôles au démarrage qui refusent une configuration non sûre | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Cache partagé | Redis facultatif pour les seules recherches non personnelles ; chaque appel échoue en ouvert | `REDIS_URL` dans le guide de déploiement (WPM-D78) |
| Sauvegarde et rétablissement | Sauvegardes chiffrées, un script de restauration et un exercice de restauration que la CI exécute | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Mise à niveau | Migrations et un guide de mise à niveau | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Quand ça tourne mal | Un manuel d’exploitation des premières actions | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Circulation des données | Un document de flux de données | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## À savoir avant de planifier

- L’interface **n’est pas conteneurisée** ; vous choisissez un adaptateur pour votre plateforme.
- Les limites de débit sont par instance. Un magasin partagé reste à fournir par vous.
- L’application utilise le rôle de base de données que nomme `DATABASE_URL` ; un jeu de rôles à moindre privilège n’est pas encore fourni.
- La révocation des jetons et leur âge maximal relèvent de votre fournisseur d’identité.
- Redis géré (cluster, sentinelle, utilisateurs à contrôle d’accès) n’a pas été essayé.
- Aucune mesure de charge n’existe, et certaines vues lisent une organisation en mémoire.
- La construction de l’image et le fichier compose ont été écrits, mais doivent d’abord être essayés dans votre environnement.

Voir [readiness-assessment.md](../../../../spec/readiness-assessment.md) pour la liste complète des écarts et le verdict, et la page de la [direction technique](../chief-technology-officer/index.md) pour la vue ingénierie.

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
