# Tutorial da demonstração

Este tutorial descreve como executar o WPM numa máquina local, carregar dados fictícios e seguir a visita guiada de cada página.

> **Apenas dados fictícios.** O início de sessão está desativado na visita. Não o ligue a um serviço que contenha dados reais.

## Pré-requisitos

- [Rust](https://www.rust-lang.org/tools/install), versão estável
- [Node.js](https://nodejs.org/) e [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), com uma base de dados vazia para a visita
- Python 3

O [guia de instalação](../../../INSTALL.md) indica as versões necessárias e as definições da base de dados.

## Passo 1: Obter o código

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Passo 2: Iniciar o serviço

O serviço lê `DATABASE_URL`. O valor por omissão é `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; defina a variável se a sua base de dados tiver outro nome, utilizador ou porta. Prepare a base de dados vazia e inicie a API. O início de sessão está desativado apenas nesta instância, e o limite de pedidos é aumentado para a visita.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Passo 3: Carregar os dados fictícios

Num segundo terminal, a partir da raiz do repositório:

```sh
workforce-demo-data http://localhost:1241
```

## Passo 4: Iniciar a aplicação

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Passo 5: Seguir a visita

Abra o endereço indicado pela aplicação e inicie sessão como **example@example.com**. Não é enviado nenhum correio eletrónico. O início de sessão abre `/demo`, uma visita a todas as páginas. Um aviso em cada página indica que os dados são fictícios e oferece ligações Anterior e Seguinte.

## Leitura adicional

- [Documentação por função](../for/index.md)
- [Comparação por produto](../../../docs/compare/index.md)
- [Especificação](../../../spec/index.md)
- [Como comentar](../../../RFC.md)

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-001/start/`), prevalece a versão em inglês.
