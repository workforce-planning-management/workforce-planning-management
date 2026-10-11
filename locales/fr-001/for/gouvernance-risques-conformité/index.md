# Gouvernance, risques, conformité

**Les questions que vous vous posez probablement :** qui est responsable de quoi, que pourrait-il arriver de fâcheux et qu’est-ce qui l’empêche, et quelles preuves puis-je montrer à un évaluateur ? Cette page rassemble en un seul lieu la matière sur la gouvernance, les risques et la conformité. Les pages de la [direction des risques](../direction-des-risques/index.md) et de la [direction juridique](../direction-juridique/index.md) couvrent le même terrain par rôle.

## Gouvernance : qui décide, et comment c’est consigné

| Sujet | Ce qui existe | Lire |
| --- | --- | --- |
| Comment le travail est gouverné | La spécification d’abord : exigence, décision, tâche, code et tests arrivent ensemble ; chaque tâche dit ce qui n’a **pas** été vérifié | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Gouvernance du projet | Gouvernance, mainteneurs, un processus de RFC et un fichier de propriétaires du code | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Décisions concernant des personnes | Une personne décide ; les scores conseillent. Conversions, démissions et demandes de travail flexible sont consignées comme la décision d’une personne | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Accès et rôles | Les rôles et attributs décident de l’accès ; une table classée décide du besoin d’en connaître sur chaque route de lecture | [auth.md](../../../../spec/auth.md) |
| Rédaction assistée par IA | Une déclaration sur la manière dont l’IA a été utilisée | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Risques : ce qui pourrait mal tourner

| Sujet | Ce qui existe | Lire |
| --- | --- | --- |
| Verdict | **Pas prêt** pour un déploiement à haut niveau de garantie ; un point bloquant a été trouvé par sondage (les lectures ne suivaient pas le besoin d’en connaître) et corrigé | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Menaces | Un modèle de menaces : actifs, acteurs, frontières, le contrôle tel que construit, et ce qui n’a **pas été évalué** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Preuve d’altération | Un journal d’audit en ajout seul, chaîné par condensat, avec un point de vérification ; les entrées ne contiennent ni texte de motif ni valeur | [audit.md](../../../../spec/audit.md) |
| Démarrage sûr en cas d’échec | La production refuse de démarrer avec la connexion désactivée, sans clés de jeton, avec une connexion de base de données en clair vers un autre hôte ou avec un Redis distant en clair | [production-readiness.md](../../../../spec/production-readiness.md) |
| Rétablissement | Sauvegardes chiffrées, un exercice de restauration exécuté en CI, un registre d’effacement pour qu’une restauration ne ramène pas une personne effacée | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Incidents | Un manuel d’exploitation des premières actions ; le déployeur ajoute les contacts et les voies d’escalade | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Biais dans les scores | Un modèle d’analyse d’impact sur l’égalité et un registre de chaque score, chacun avec un projet d’évaluation | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Risques ouverts | Épinglage et signature de la chaîne d’approvisionnement, preuves de charge, contrôles des journaux pour les données personnelles, limites de débit partagées, jeu de rôles de base de données à moindre privilège, consignes sur la révocation des jetons | [readiness-assessment.md](../../../../spec/readiness-assessment.md), section 3 |

## Conformité : ce que vous pouvez montrer à un évaluateur

| Sujet | Ce qui existe | Lire |
| --- | --- | --- |
| Auto-évaluation des contrôles | Une liste de contrôle où chaque réponse est satisfaite, partiellement satisfaite, non satisfaite ou à fournir par le déployeur | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Analyse d’impact relative à la protection des données | Un modèle, rempli pour la démonstration, avec les parties que vous devez compléter signalées | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Registre des traitements | Un registre adaptable | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Flux de données | Où circulent les données personnelles | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Conservation | Un calendrier par type de dossier | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Droits des personnes | Export d’accès et effacement | [regulatory.md](../../../../spec/regulatory.md) |
| Données de catégorie particulière | Le suivi de l’égalité et les exigences de santé au travail sont désactivés tant qu’une base légale n’est pas consignée, puis bornés | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Accessibilité et langue | Une déclaration d’accessibilité et une déclaration sur la langue officielle | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Licences et dépendances | Fichiers de licence, une politique de dépendances vérifiée par `cargo deny`, et une nomenclature logicielle à chaque publication | [production-readiness.md](../../../../spec/production-readiness.md) |

## Ce que seules des personnes peuvent faire

Ce sont des verrous, pas du code, et le dépôt ne les marque pas comme faits : un test d’intrusion indépendant, un audit d’accessibilité indépendant, votre analyse d’impact relative à la protection des données signée par votre délégué, un exercice de restauration sur votre plateforme, une relecture professionnelle des traductions et votre approbation des analyses d’impact sur l’égalité.

## Limites de cette page

Les tableaux renvoient à des documents. Que chaque document soit vrai pour votre déploiement, c’est à vous et à votre évaluateur d’en juger. Les tests de l’auteur ne sont pas une revue indépendante, et rien ici ne constitue un avis juridique ni une certification.

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
