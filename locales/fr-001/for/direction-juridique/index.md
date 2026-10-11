# Pour la direction juridique (CLO)

**La question que vous vous posez probablement :** quelles obligations juridiques ce logiciel crée-t-il ou aide-t-il à remplir, et que devons-nous décider avant que des données réelles n’y entrent ?

> Rien ici ne constitue un avis juridique. La spécification expose une position, non une conclusion sur une juridiction quelconque.

## Ce que vous obtenez

| Sujet | Ce qui existe | Lire |
| --- | --- | --- |
| Position | Position sur le RGPD britannique et les dossiers d’emploi ; statut de démonstration ; ce que la production exigerait en plus | [regulatory.md](../../../../spec/regulatory.md) |
| Analyse d’impact | Un modèle d’analyse d’impact relative à la protection des données, rempli pour la démonstration, avec les parties que vous devez compléter signalées | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Registre des traitements et flux de données | Documents adaptables | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md), [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Conservation | Un calendrier par type de dossier, avec un balayage pour certains | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Droits des personnes | Export d’accès, effacement et un registre d’effacement | [regulatory.md](../../../../spec/regulatory.md) (WPM-R30) |
| Données de catégorie particulière | Le suivi de l’égalité et les exigences de santé au travail sont désactivés tant que vous ne consignez pas une base légale, puis bornés : volontaires, pour la personne seule ou limités par rôle, agrégats au-dessus d’un seuil, jamais utilisés dans une décision | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Droits légaux | Demandes de travail flexible avec un délai de décision et des motifs que vous configurez ; un préavis de démission que vous configurez | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Obligations d’égalité | Un modèle d’analyse d’impact sur l’égalité et un registre de chaque score | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| Accessibilité et langue | Une déclaration d’accessibilité et une déclaration sur la langue officielle | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Organes collectifs | Recherche et proposition sur les organes reconnus et la consultation, jamais l’adhésion individuelle | [worker-unions.md](../../../../spec/worker-unions.md) |
| Prestataires | La base de la mission, le fournisseur, la voie et le taux masqué sont consignés. Savoir si une mission est licite ou correctement classée n’est pas décidé par le logiciel | [engagements.md](../../../../spec/engagements.md) |
| Licences | Fichiers de licence, une politique de dépendances et une nomenclature logicielle à chaque publication | [production-readiness.md](../../../../spec/production-readiness.md) |

## Valeurs par défaut qui ne sont pas la loi

Plusieurs réglages sont des valeurs par défaut, non des avis juridiques : le préavis de démission, le délai de décision et d’appel pour le travail flexible, et les motifs de refus. Le guide de déploiement demande de les **confirmer** pour votre juridiction. Les calculs légaux comme l’impôt sur la paie ne sont pas mis en œuvre.

## Décisions qui vous reviennent

La base légale de chaque finalité de traitement, l’analyse d’impact relative à la protection des données et sa signature, les accords avec les sous-traitants, le calendrier de conservation que vous adoptez, et l’activation ou non d’une fonction de catégorie particulière.

Voir aussi la [direction des ressources humaines](../direction-des-ressources-humaines/index.md) et la [direction des risques](../direction-des-risques/index.md).

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
