# Governança, riscos e conformidade

**As perguntas que provavelmente faz:** quem responde por quê, o que pode correr mal e o que o impede, e que evidência posso mostrar a um avaliador? Esta página reúne num só lugar o material de governança, riscos e conformidade. As páginas da [direção de riscos](../direção-de-riscos/index.md) e da [direção jurídica](../direção-jurídica/index.md) cobrem o mesmo terreno por função.

## Governança: quem decide e como fica registado

| Tema | O que existe | Leia |
| --- | --- | --- |
| Como o trabalho é governado | Primeiro a especificação: requisito, decisão, tarefa, código e testes chegam juntos; cada tarefa diz o que **não** foi verificado | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Governança do projeto | Governança, mantenedores, um processo de RFC e um ficheiro de responsáveis pelo código | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Decisões sobre pessoas | Uma pessoa decide; as pontuações aconselham. Conversões, demissões e pedidos de trabalho flexível são registados como decisão de uma pessoa | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Acesso e papéis | Os papéis e atributos decidem o acesso; uma tabela classificada decide a necessidade de conhecer em cada rota de leitura | [auth.md](../../../../spec/auth.md) |
| Autoria assistida por IA | Uma declaração sobre como a IA foi usada | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Riscos: o que pode correr mal

| Tema | O que existe | Leia |
| --- | --- | --- |
| Veredicto | **Não está pronto** para uma implementação de alta garantia; foi encontrado um bloqueio por sondagem (as leituras não seguiam a necessidade de conhecer) e corrigido | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Ameaças | Um modelo de ameaças: ativos, atores, fronteiras, o controlo tal como foi construído e o que **não foi avaliado** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Prova de adulteração | Um registo de auditoria só de acréscimo, encadeado por resumo, com um ponto de verificação; as entradas não contêm texto de motivo nem valor | [audit.md](../../../../spec/audit.md) |
| Arranque seguro em caso de falha | A produção recusa arrancar com o início de sessão desligado, sem chaves de token, com uma ligação à base de dados em claro para outro anfitrião ou com um Redis remoto em claro | [production-readiness.md](../../../../spec/production-readiness.md) |
| Recuperação | Cópias cifradas, um exercício de restauro executado na CI, um livro de apagamentos para que um restauro não traga de volta uma pessoa apagada | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Incidentes | Um manual operacional das primeiras ações; quem implementa acrescenta contactos e vias de escalamento | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Enviesamento nas pontuações | Um modelo de impacto sobre a igualdade e um registo de cada pontuação, cada uma com um rascunho de avaliação | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Riscos em aberto | Fixação e assinatura da cadeia de fornecimento, evidência de carga, verificações dos registos quanto a dados pessoais, limites de taxa partilhados, conjunto de papéis de base de dados de privilégio mínimo, orientação sobre revogação de tokens | [readiness-assessment.md](../../../../spec/readiness-assessment.md), secção 3 |

## Conformidade: o que pode mostrar a um avaliador

| Tema | O que existe | Leia |
| --- | --- | --- |
| Autoavaliação dos controlos | Uma lista de verificação em que cada resposta é cumprida, parcialmente cumprida, não cumprida ou a responder por quem implementa | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Avaliação de impacto sobre a proteção de dados | Um modelo, preenchido para a demonstração, com as partes que tem de completar assinaladas | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Registo de tratamentos | Um registo adaptável | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Fluxo de dados | Para onde se movem os dados pessoais | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Conservação | Um calendário por tipo de registo | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Direitos dos titulares | Exportação de acesso e apagamento | [regulatory.md](../../../../spec/regulatory.md) |
| Dados de categoria especial | A monitorização da igualdade e os requisitos de saúde no trabalho estão desligados até se registar uma base legal, e depois limitados | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Acessibilidade e língua | Uma declaração de acessibilidade e uma declaração sobre a língua oficial | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Licenças e dependências | Ficheiros de licença, uma política de dependências verificada com `cargo deny` e uma lista de materiais de software em cada publicação | [production-readiness.md](../../../../spec/production-readiness.md) |

## O que só as pessoas podem fazer

São portas, não código, e o repositório não as marca como concluídas: um teste de intrusão independente, uma auditoria de acessibilidade independente, a sua avaliação de impacto sobre a proteção de dados assinada pelo seu responsável, um exercício de restauro na sua plataforma, uma revisão profissional das traduções e a sua aprovação das avaliações de impacto sobre a igualdade.

## Limites desta página

As tabelas apontam para documentos. Se cada documento é verdadeiro para a sua implementação, cabe a si e ao seu avaliador julgar. Os testes do autor não são uma revisão independente, e nada aqui é aconselhamento jurídico nem certificação.

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
