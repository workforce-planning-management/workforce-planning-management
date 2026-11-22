# Gestion de la planification des effectifs

**Une plateforme RH open source qui couvre le cycle de vie de l’employé, du recrutement au départ à la retraite.**

Workforce Planning Management (WPM) réunit dans une seule application le recrutement, le temps et les absences, le développement des employés, la planification des effectifs et la paie. C’est un logiciel de démonstration, conçu pour être évalué et servir de conception de référence.

Langue: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Logiciel de démonstration, pas encore prêt pour un usage en production.** WPM n’utilise que des données synthétiques et ses calculs juridiques sont illustratifs. Ne saisissez pas de véritables données personnelles. [Le chemin vers la production](path-to-production/index.md) explique ce que demanderait un usage en production et le degré de préparation de chaque partie. Voir aussi [la position réglementaire](../../spec/regulatory.md).

[Lancer la démonstration](start/index.md) [Notes par rôle](for/index.md) [Code source sur GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Avantages

Les données RH sont souvent dispersées entre des outils distincts, comme les boîtes de candidatures, les tableurs de congés et les tableaux de planning. Chaque passage de relais ajoute du travail manuel et un risque d’erreur. WPM est conçu pour réduire ces passages.

- **Un seul système.** L’intégration, les absences, la formation et les données de paie sont dans une seule application, ce qui limite les ressaisies.
- **Minimisation des données.** Les données qui ne doivent pas être conservées n’ont pas de champ, et celles qui ne doivent pas être divulguées n’ont pas de route dans l’API. Par exemple, il n’y a pas de champ de diagnostic et les réponses au sondage éclair n’ont pas d’auteur.
- **Règles de reporting.** Une valeur inconnue n’est jamais présentée comme zéro, les agrégats ne désignent personne et les estimations disent sur quoi elles reposent.
- **Localisation.** L’application existe dans 25 locales et les notes par rôle dans 21, y compris des langues qui s’écrivent de droite à gauche. Les règles de chaque pays sont stockées sous forme de données. Les traductions ont été produites par une IA et n’ont été relues par aucun locuteur natif.
- **Traçabilité.** Le code source est ouvert et chaque comportement renvoie à une exigence, une décision de conception et une tâche numérotées.
- **Déploiement indépendant.** La pile se compose d’une API JSON en Rust, d’un client SvelteKit et de PostgreSQL. Elle se compile depuis un clone propre et les intégrations avec les applications sœurs sont facultatives.

## Fonctionnalités

- **Recrutement et dossiers.** Postes ouverts, parcours des candidats, intégration, dossiers des travailleurs et organigramme.
- **Temps et planning.** Temps et absences, plannings avec contrôle du temps de travail, le planning d’astreinte avec échanges et rappels, et un annuaire du personnel.
- **Développement.** Évaluations, retours à 360°, formation, analyse des écarts de compétences, recommandations de temps de formation et formation obligatoire.
- **Planification et reporting.** Planification stratégique des effectifs, indicateurs et analyses, et un tableau de bord pour la direction générale.
- **Arrivées et départs.** Listes de tâches datées et passation du dernier jour avec journal d’audit.
- **Communication.** Annonces avec destinataires, liens et nombre de lectures.
- **Droits et bien-être des employés.** Messages de bien-être, sondage éclair anonyme, aménagements, accès à ses propres données, effacement et conservation.
- **Paie.** Paie et rémunération, avec des calculs juridiques illustratifs.

## Comparaison

Le tableau compare WPM à trois catégories de systèmes RH. C’est une orientation générale, pas un test comparatif. Voir aussi la [comparaison produit par produit](../../docs/compare/index.md).

| | WPM | Suites tout-en-un (Workday, SAP SuccessFactors, BambooHR) | RH open source (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Outils spécialisés (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Périmètre** | Cycle complet, du recrutement au départ à la retraite | Périmètre comparable sur une seule plateforme | Les mêmes fonctions de base | Une seule fonction |
| **Licence** | Open source | Commerciale | Open source | Commerciale |
| **Principal atout** | Minimisation des données, conception traçable, analyse des écarts de compétences et du temps de formation | Une plateforme intégrée et prise en charge | Plus d’intégrations, de localisations et de déploiements | Profondeur sur une fonction |
| **Usage typique** | Évaluation, conception de référence ou développement ultérieur | Organisations qui ont besoin d’un système pris en charge | Organisations qui veulent de l’open source en production | Organisations qui ont besoin de profondeur sur une fonction et peuvent intégrer le reste |

## À qui il s’adresse

- **Dirigeants.** Les [notes par rôle](for/index.md) exposent ce que WPM apporte à chaque rôle de direction et comment lire ce qu’il montre.
- **Équipes RH.** Le [tutoriel de démonstration](start/index.md) parcourt chaque page avec des données fictives.
- **Développeurs et architectes.** Voir la [spécification](../../spec/index.md) et le [guide d’installation](../../INSTALL.md).
- **Évaluateurs.** L’[évaluation de préparation](../../spec/readiness-assessment.md) énumère les écarts entre WPM et un usage en production.

## Obtenir de l’aide

- **Signalez un bogue ou posez une question** dans le [suivi des tickets](https://github.com/workforce-planning-management/workforce-planning-management/issues). Indiquez la page, ce que vous avez fait, ce que vous attendiez et ce qui s’est passé. N’incluez pas de véritables données personnelles.
- **Signalez un problème de sécurité** par courriel, comme l’indique la [politique de sécurité](../../SECURITY.md), et non dans un ticket public.
- **Apprenez une tâche dans l’application.** Le lien **Aide** de l’en-tête de l’application ouvre la page d’aide et les guides pour les employés, les managers et les RH. Chaque message d’erreur y renvoie aussi.

## Prochaines étapes

1. **Lisez la note de votre rôle.** Chaque note décrit ce que WPM apporte et ce qui reste ouvert.
2. **Lancez la démonstration.** Le parcours fonctionne sur un poste local avec des données fictives.
3. **Donnez votre avis.** Voir [comment commenter](../../RFC.md) et [comment contribuer](../../CONTRIBUTING.md).

[Lancer la démonstration](start/index.md) [Documentation](../../index.md)
