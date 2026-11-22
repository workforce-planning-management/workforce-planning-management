# Le chemin vers la production

WPM est un logiciel de démonstration. Cette page dit ce que signifierait pour lui un usage en production, et liste ce qui sépare WPM de cet usage. Chaque élément porte une estimation de son degré de préparation, de 0 à 100 %.

## Ce que signifie la production

Un usage en production signifie qu’une organisation s’appuie sur WPM pour un travail réel :

- les dossiers sont ceux de personnes réelles : salariés, candidats et leurs contacts en cas d’urgence ;
- la paie calculée par WPM est versée, et le droit du lieu de travail s’y applique ;
- des personnes se connectent chaque jour, et une partie de ce qu’elles lisent est sensible, comme les absences pour maladie ;
- l’organisation rend compte de son fonctionnement à une autorité de régulation, à un auditeur et aux personnes figurant dans les dossiers.

Cette page mesure WPM par rapport au cas le plus exigeant pour lequel il a été évalué : une grande organisation médicale ou publique ([l’évaluation de préparation](../../../spec/readiness-assessment.md), en anglais). Une organisation plus petite peut avoir moins besoin de certains éléments, mais elle a besoin des mêmes types de preuves.

La production exige trois types de preuves, et le logiciel seul ne fournit que le premier :

1. **Un logiciel** qui fait le travail en sécurité, avec des tests qui le montrent.
2. **Un exploitant** qui l’héberge et le fait fonctionner : sauvegardes, supervision, support et réponse aux incidents.
3. **Des personnes autres que l’auteur** qui l’ont vérifié : un testeur d’intrusion, un auditeur d’accessibilité, des traducteurs et le délégué à la protection des données de l’exploitant.

## Comment lire les estimations

- Une estimation est le jugement de l’auteur au 2026-11-13, établi à partir de la spécification et des tests. Ce n’est ni une mesure ni une certification.
- 0 % signifie que rien n’existe encore. 100 % signifie construit, testé et vérifié par une personne indépendante sur un déploiement réel. Aucun élément n’est à 100 %.
- Chaque élément indique sur quoi repose son estimation et ce qui reste à faire.
- Les estimations ne sont pas moyennées. Un seul élément à une estimation basse suffit à tenir WPM hors de la production.

## Le logiciel

- **75%** **Connexion et accès selon le besoin d’en connaître.** La connexion est activée par défaut, et le service refuse de démarrer en production sans elle (WPM-R72). Les lectures sont limitées aux personnes qui ont besoin d’en connaître (WPM-T188), et un jeton a une durée de vie maximale (WPM-T200). Repose sur : les tests de contrôle d’application qui s’exécutent dans la CI. Reste à faire : il n’existe pas de liste de révocation des jetons, et aucun test d’intrusion indépendant n’a été réalisé.
- **70%** **Une piste d’audit complète et inaltérée.** La piste d’audit est en ajout seul et chaînée par condensats, et peut être exportée vers un système de supervision (WPM-T189, WPM-T197). Repose sur : des tests cassés exprès pour prouver qu’ils échouent. Reste à faire : une fin de chaîne tronquée n’est pas détectée, et un propriétaire de la base de données peut encore désactiver les déclencheurs.
- **70%** **Protection des données dès la conception.** Les personnes sont identifiées par des références, il n’existe pas de colonnes démographiques, et le droit d’accès, l’effacement et la conservation sont prévus pour chaque type de dossier. Repose sur : les tests, et la section 1 de l’évaluation de préparation. Reste à faire : l’analyse propre de l’exploitant et sa base juridique (ci-dessous).
- **75%** **Connexions et sauvegardes chiffrées.** En production, le service refuse une connexion non chiffrée à la base de données et interrompt une requête bloquée au bout d’un délai (WPM-T191), et les sauvegardes sont chiffrées (WPM-T192). Repose sur : les tests de ces deux points. Reste à faire : la gestion des clés et des certificats relève de l’exploitant.
- **10%** **Calculs réglementaires.** Les calculs réglementaires sont illustratifs, et l’impôt sur les salaires est un bouchon ([la position réglementaire](../../../spec/regulatory.md), en anglais). Repose sur : la spécification, qui le dit. Reste à faire : les règles de chaque lieu de travail, rédigées et vérifiées par une personne qualifiée. Ne payez personne à partir de WPM.
- **50%** **Montée en charge.** Sur un seul ordinateur portable, les pages d’annuaire et la vue de capacité n’échouent pas à 50 000 travailleurs, et les vues de planification refusent au-delà de 10 000 (WPM-T196, WPM-T230). Repose sur : une seule exécution mesurée ([BENCHMARKS.md](../../../BENCHMARKS.md), en anglais). Reste à faire : la vue de capacité croît encore proportionnellement à l’effectif, et rien n’a été mesuré sur le matériel d’un exploitant.
- **65%** **Chaîne d’approvisionnement.** Chaque action de la CI est épinglée à un commit et l’historique est analysé à la recherche de secrets. L’image de conteneur est signée et porte sa provenance, et cela a été exécuté pour la version v0.1.2 (WPM-T194, WPM-T195). Repose sur : cette exécution de publication. Reste à faire : le paquet de l’image est privé, donc personne en dehors du projet ne peut encore le vérifier.
- **60%** **Supervision.** Chaque réponse porte un identifiant de requête, et les lignes de journal ne contiennent ni chaîne de requête ni identifiant personnel (WPM-T197). Repose sur : les tests du journal des requêtes. Reste à faire : les tableaux de bord et les alertes relèvent de l’exploitant, et les limites de débit ne s’appliquent qu’à une seule instance (WPM-T198).
- **60%** **Sauvegarde et restauration.** Des sauvegardes chiffrées, un exercice de restauration qui s’exécute dans la CI, et un guide d’exploitation. Repose sur : l’exercice. Reste à faire : un exercice de restauration sur la plateforme propre de l’exploitant au cours des 90 derniers jours calendaires (jalon WPM-G4).
- **40%** **Contrôles d’accessibilité.** Des contrôles automatisés par rapport au référentiel WCAG 2.2 AA s’exécutent sur l’application et sur ce site, en thème clair et en thème sombre. Repose sur : les tests Playwright et axe. Reste à faire : l’audit indépendant (ci-dessous) ; aucun lecteur d’écran n’a été utilisé.
- **20%** **Traductions.** L’application compte 25 locales et les pages par rôle en comptent 21. Repose sur : le texte existe. Reste à faire : une IA a écrit les traductions, et personne ne les a relues de façon professionnelle (WPM-T155).

## Vérifications par des personnes autres que l’auteur

- **0%** **Test d’intrusion indépendant.** Aucun test d’intrusion d’une instance déployée n’a été réalisé (WPM-T201). Repose sur : la tâche est ouverte. Reste à faire : le test, et la correction de ce qu’il trouve.
- **0%** **Audit d’accessibilité indépendant.** Aucun audit par rapport au référentiel WCAG 2.2 AA avec des personnes qui utilisent des technologies d’assistance n’a été mené (WPM-T202). Repose sur : la tâche est ouverte. Reste à faire : l’audit, et la correction de ce qu’il trouve.
- **25%** **Modèle de menaces.** Un modèle de menaces initial existe ([threat-model.md](../../../spec/governance/threat-model.md), en anglais), rédigé par l’auteur. Repose sur : le document. Reste à faire : une équipe indépendante le refait pour un déploiement réel.

## La part de l’exploitant

- **20%** **Analyse d’impact relative à la protection des données.** Un dossier à adapter existe : une analyse d’impact, un flux de données, un registre des traitements et un calendrier de conservation ([spec/governance](../../../spec/governance/dpia.md), en anglais). Repose sur : le dossier. Reste à faire : il n’y a pas d’exploitant, donc aucun délégué à la protection des données n’a validé une analyse, une base juridique, un calendrier de conservation ou un contrat de sous-traitance (jalon WPM-G5).
- **15%** **Analyses d’impact sur l’égalité.** Un modèle et un registre existent ([equality-impact](../../../spec/equality-impact/template.md), en anglais). Repose sur : le modèle. Reste à faire : l’exploitant rédige et approuve une analyse pour son propre usage.
- **40%** **Hébergement et exploitation.** Une configuration de production, une image de conteneur, et des documents pour le déploiement et les mises à niveau existent. Repose sur : ces documents et l’exercice de restauration. Reste à faire : rien n’a été exécuté sur un hôte durci, et un exploitant doit faire fonctionner le service.
- **10%** **Support.** Il existe un outil de suivi des tickets et une politique de sécurité ([SECURITY.md](../../../SECURITY.md), en anglais). Repose sur : ces deux éléments. Reste à faire : il n’existe ni contrat de support, ni niveau de service, ni astreinte, ni organisation derrière WPM.

## Ce que cette page n’affirme pas

La page ne consigne aucune décision de mise en service. Cette décision appartient à un exploitant et se prend en dehors de WPM. Les estimations ne couvrent que les éléments listés, et la liste propre à l’évaluation de préparation de ce qu’elle n’a pas vérifié s’applique aussi ici. Lorsqu’un système clinique est connecté, une revue de sécurité clinique relève de la décision de l’exploitant.
