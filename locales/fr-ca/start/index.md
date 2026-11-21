# Tutoriel de démonstration

Ce tutoriel explique comment lancer WPM sur un ordinateur local, charger des données fictives et suivre le parcours guidé de chaque page.

> **Données fictives uniquement.** La connexion est désactivée pendant le parcours. Ne le raccordez pas à un service qui contient de véritables données.

## Prérequis

- [Rust](https://www.rust-lang.org/tools/install), version stable
- [Node.js](https://nodejs.org/) et [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), avec une base vide pour le parcours
- Python 3

Le [guide d’installation](../../../INSTALL.md) indique les versions requises et les réglages de la base de données.

## Étape 1 : Récupérer le code

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Étape 2 : Démarrer le service

Le service lit `DATABASE_URL`. Sa valeur par défaut est `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development` ; définissez la variable si votre base a un autre nom, un autre utilisateur ou un autre port. Préparez la base vide et démarrez l’API. La connexion n’est désactivée que pour cette instance et la limite de requêtes est relevée pour le parcours.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Étape 3 : Charger les données fictives

Dans un second terminal, depuis la racine du dépôt :

```sh
workforce-demo-data http://localhost:1241
```

## Étape 4 : Démarrer l’application

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Étape 5 : Suivre le parcours

Ouvrez l’adresse affichée par l’application et connectez-vous avec **example@example.com**. Aucun courriel n’est envoyé. La connexion ouvre `/demo`, un parcours de toutes les pages. Un bandeau sur chaque page indique que les données sont fictives et propose les liens Précédent et Suivant.

## Pour aller plus loin

- [Notes par rôle](../for/index.md)
- [Comparaison produit par produit](../../../docs/compare/index.md)
- [Spécification](../../../spec/index.md)
- [Comment commenter](../../../RFC.md)
