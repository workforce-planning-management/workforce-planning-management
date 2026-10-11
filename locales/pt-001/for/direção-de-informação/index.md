# Para a direção de informação (CIO)

**A pergunta que provavelmente faz:** como se integra no nosso parque, quem pode iniciar sessão, onde vivem os dados e como o executamos e recuperamos?

## O que obtém

| Assunto | O que existe | Leia |
| --- | --- | --- |
| Forma | Uma API JSON e um cliente de navegador separado; o PostgreSQL guarda os dados | [architecture.md](../../../../spec/architecture.md) |
| Identidade | O WPM não guarda identidades. Uma pessoa, um trabalhador e uma organização são referências a outros serviços, resolvidas apenas pelo nome e guardadas em cache | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Início de sessão | Ligado por defeito. Início de sessão com diretório empresarial (OIDC, compatível com Entra, direto ou através do Keycloak); os papéis e atributos decidem o acesso | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Implementação | Uma construção de contentor e um ficheiro compose para uma demonstração; para produção, um proxy que termina o TLS, definições obrigatórias e verificações no arranque que recusam uma configuração insegura | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Cache partilhada | Redis opcional apenas para consultas não pessoais; cada chamada falha em aberto | `REDIS_URL` no guia de implementação (WPM-D78) |
| Cópia de segurança e recuperação | Cópias cifradas, um script de restauro e um exercício de restauro que a CI executa | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Atualização | Migrações e um guia de atualização | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Quando algo corre mal | Um manual operacional das primeiras ações | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Circulação de dados | Um documento de fluxo de dados | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## O que convém saber antes de planear

- A interface **não está em contentor**; escolhe um adaptador para a sua plataforma.
- Os limites de taxa são por instância. Um armazenamento partilhado fica por sua conta.
- A aplicação usa o papel de base de dados que `DATABASE_URL` indicar; um conjunto de papéis de privilégio mínimo ainda não é fornecido.
- A revogação de tokens e a sua idade máxima são do seu fornecedor de identidade.
- O Redis gerido (cluster, sentinela, utilizadores de controlo de acesso) não foi experimentado.
- Não existe medição de carga, e algumas vistas leem uma organização para memória.
- A construção da imagem e o ficheiro compose foram escritos, mas devem ser experimentados primeiro no seu ambiente.

Veja [readiness-assessment.md](../../../../spec/readiness-assessment.md) para a lista completa de lacunas e o veredicto, e a página da [direção de tecnologia](../direção-de-tecnologia/index.md) para a visão de engenharia.

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
