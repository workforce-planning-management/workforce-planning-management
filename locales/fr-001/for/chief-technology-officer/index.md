# Pour la direction technique (CTO)

**La question que vous vous posez probablement :** est-ce bien construit, mon équipe peut-elle le modifier en sécurité, et comment le savoir ?

## Ce que vous obtenez

| Sujet | Ce qui existe | Lire |
| --- | --- | --- |
| Pile technique | Rust avec Loco (Axum, SeaORM) et PostgreSQL 18 pour l’API ; SvelteKit 2 avec Svelte 5 et TypeScript pour le client | [architecture.md](../../../../spec/architecture.md) |
| Règle de conception | Un cœur pur : les règles sont sans base de données ni horloge, testées de façon exhaustive, puis branchées aux contrôleurs | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| La spécification d’abord | Un changement, c’est une exigence, une décision, une tâche, du code et des tests ensemble ; chaque tâche dit ce qui n’a **pas** été vérifié | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Tests | Tests unitaires des règles, tests de requêtes avec base de données, binaires d’application exécutés avec la connexion activée, tests unitaires et de navigateur du front-end. Contrôles par mutation : casser le code exprès et confirmer qu’un test échoue | [testing.md](../../../../spec/testing.md) |
| Décomptes | Générés à partir des fichiers, jamais recopiés à la main | [implementation-status.md](../../../../spec/implementation-status.md) |
| Chaîne d’approvisionnement | `cargo deny` pour les licences et les interdictions, crates partagés embarqués, une nomenclature logicielle à chaque publication | [production-readiness.md](../../../../spec/production-readiness.md) |
| Contrat d’API | OpenAPI, et un test qui échoue si une route enregistrée n’est pas documentée | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Contrôle d’accès | La politique décide par action seulement ; les gestionnaires et une table de routes classée décident du besoin d’en connaître, et une analyse du code source échoue devant une route de lecture non classée | [auth.md](../../../../spec/auth.md) |
| Performance | [BENCHMARKS.md](../../../../BENCHMARKS.md) consigne ce qui a été mesuré ; il n’existe pas de test de charge | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Dette technique connue

- Plusieurs vues chargent des organisations entières en mémoire.
- Les verrous de ligne sur les plans de conversion sont raisonnés, non prouvés par un test.
- La CI, la publication et l’analyse dynamique sont écrites mais n’ont pas tourné sur la plateforme d’hébergement.
- Les Actions sont épinglées par étiquette et non par condensat, et les images ne sont pas signées.
- Certains tests Redis avec mot de passe et TLS demandent des variables et une autorité de certification, donc la CI ne les exécute pas.
- L’arborescence `spec/` est vaste ; un fichier d’état généré permet de voir ce qui est déclaré par rapport à ce qui passe.

## Comment contribuer

Lisez d’abord [AGENTS.md](../../../../AGENTS.md) : cinq règles, puis une carte par tâche.

Voir aussi la page de la [direction des systèmes d’information](../chief-information-officer/index.md) pour la vue exploitation.

## Degré d’actualité

Rédigé le 2026-10-11 à partir de la spécification, et non en exécutant le logiciel pour cette page. Les chiffres qui changent (tests, migrations, tâches terminées) ne sont pas recopiés ici ; lisez-les dans [implementation-status.md](../../../../spec/implementation-status.md), que produit un script. WPM est un logiciel de démonstration avec des données synthétiques : cette page dit ce qu’il fait et ne fait pas, et ne prétend pas qu’il convienne à votre organisation. Rien ici ne constitue un avis juridique.

> Cette traduction a été écrite par une IA et n’a pas été relue par un locuteur natif. En cas de différence avec la version anglaise (`locales/en-gb-oxendict/for/`), la version anglaise fait foi.
