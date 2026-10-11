# Para a direção de tecnologia (CTO)

**A pergunta que provavelmente faz:** está bem construído, a minha equipa pode alterá-lo com segurança e como sei?

## O que obtém

| Assunto | O que existe | Leia |
| --- | --- | --- |
| Pilha | Rust com Loco (Axum, SeaORM) e PostgreSQL 18 para a API; SvelteKit 2 com Svelte 5 e TypeScript para o cliente | [architecture.md](../../../../spec/architecture.md) |
| Regra de conceção | Um núcleo puro: as regras não dependem de base de dados nem de relógio, são testadas exaustivamente e só depois ligadas aos controladores | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Primeiro a especificação | Uma alteração é requisito, decisão, tarefa, código e testes em conjunto; cada entrada de tarefa diz o que **não** foi verificado | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Testes | Testes unitários das regras, testes de pedidos com base de dados, binários de imposição executados com o início de sessão ligado, testes unitários e de navegador do front-end. Verificações de mutação: partir o código de propósito e confirmar que um teste falha | [testing.md](../../../../spec/testing.md) |
| Contagens | Geradas a partir dos ficheiros, nunca copiadas à mão | [implementation-status.md](../../../../spec/implementation-status.md) |
| Cadeia de fornecimento | `cargo deny` para licenças e proibições, crates partilhados incluídos, uma lista de materiais de software em cada publicação | [production-readiness.md](../../../../spec/production-readiness.md) |
| Contrato da API | OpenAPI e um teste que falha se uma rota registada não estiver documentada | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Controlo de acesso | A política decide apenas por ação; os tratadores e uma tabela de rotas classificada decidem a necessidade de conhecer, e uma análise do código-fonte falha perante uma rota de leitura não classificada | [auth.md](../../../../spec/auth.md) |
| Desempenho | [BENCHMARKS.md](../../../../BENCHMARKS.md) regista o que foi medido; não existe teste de carga | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Dívida de engenharia conhecida

- Várias vistas carregam organizações inteiras em memória.
- Os bloqueios de linha nos planos de conversão estão fundamentados, não provados por um teste.
- A CI, a publicação e a análise dinâmica estão escritas, mas não correram na plataforma de alojamento.
- As Actions estão fixadas por etiqueta, não por resumo, e as imagens não estão assinadas.
- Alguns testes de Redis com palavra-passe e TLS precisam de variáveis e de uma autoridade de certificação, pelo que a CI não os executa.
- A árvore `spec/` é grande; um ficheiro de estado gerado é a forma de ver o que está declarado face ao que passa.

## Como contribuir

Leia primeiro [AGENTS.md](../../../../AGENTS.md): cinco regras e depois um mapa por tarefa.

Veja também a página da [direção de informação](../chief-information-officer/index.md) para a visão operacional.

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
