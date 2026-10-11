# Pour la direction de la sécurité

**La question que vous vous posez probablement :** comment est-ce protégé, comment saurions-nous qu’il est attaqué ou détourné, et qu’est-ce que personne n’a testé ?

## Ce que vous obtenez

| Sujet | Ce qui existe | Lire |
| --- | --- | --- |
| Menaces | Un modèle de menaces : actifs, acteurs, frontières de confiance, le contrôle tel que construit, et ce qui n’a **pas été évalué** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Authentification | Connexion activée par défaut ; la production refuse de démarrer si elle est désactivée ou sans source de clés de jeton ; connexion par annuaire d’entreprise (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Autorisation | La politique décide par action seulement ; une table classée décide du besoin d’en connaître sur chaque route de lecture, et un test d’analyse du code source échoue devant une route non classée | [auth.md](../../../../spec/auth.md) |
| Audit | Journal en ajout seul, chaîné par condensat, avec un point de vérification ; les entrées ne portent ni texte de motif ni valeur | [audit.md](../../../../spec/audit.md) |
| Transport et navigateur | En-têtes de sécurité sur chaque réponse de l’API, une liste CORS explicite qui empêche le démarrage si elle manque, `no-store` sur `/api` | WPM-R73 dans [production-readiness.md](../../../../spec/production-readiness.md) |
| Abus | Limites de débit par appelant, plus strictes pour l’effacement, l’import, l’export et les routes de jeton | WPM-R74, même fichier |
| Configuration non sûre | Le démarrage refuse une connexion de base de données ou de Redis en clair vers un autre hôte, et une origine CORS absente | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Données au repos | Sauvegardes chiffrées ; le cache partagé ne contient que des recherches non personnelles et échoue en ouvert | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Chaîne d’approvisionnement | Contrôles des licences et interdictions avec `cargo deny`, crates partagés embarqués, une nomenclature logicielle à chaque publication | [production-readiness.md](../../../../spec/production-readiness.md) |
| Incidents | Un manuel d’exploitation des premières actions ; vous ajoutez les contacts et les obligations de notification | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Auto-évaluation | Une liste de contrôle des mesures : satisfaite, partiellement satisfaite, non satisfaite, à fournir par le déployeur | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Écarts connus, tels que consignés

- Aucun test d’intrusion indépendant n’a été fait. Les tests ont été écrits par l’auteur.
- Les Actions sont épinglées par étiquette et non par condensat, et les images ne sont pas signées.
- Les limites de débit sont par instance.
- L’application utilise le rôle de base de données qu’on lui donne ; aucun jeu de rôles à moindre privilège n’est fourni.
- La révocation des jetons et leur âge maximal relèvent de votre fournisseur d’identité.
- Le chemin du gestionnaire de contenu charge un script depuis un CDN public sans version épinglée ni condensat d’intégrité.
- Les journaux n’ont pas été contrôlés pour les données personnelles, et le journal d’audit n’est pas exporté vers un système de supervision.
- Redis géré (cluster, sentinelle, utilisateurs à contrôle d’accès) n’a pas été essayé.

## Ce que seules des personnes peuvent faire

Commander le test d’intrusion indépendant, examiner le modèle de menaces pour votre environnement, exécuter un exercice de restauration sur votre plateforme et définir la protection des branches sur la plateforme d’hébergement. Voir l’[évaluation de préparation](../../../../spec/readiness-assessment.md), dont le verdict est **pas prêt** pour un déploiement à haut niveau de garantie.

Voir aussi les pages de la [direction des risques](../direction-des-risques/index.md) et de la [direction des systèmes d’information](../direction-des-systèmes-dinformation/index.md).

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
