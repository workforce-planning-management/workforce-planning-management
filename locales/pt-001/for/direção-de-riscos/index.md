# Para a direção de riscos (CRO)

**A pergunta que provavelmente faz:** o que pode correr mal com este sistema, o que o impede e o que ninguém verificou?

## O ponto de partida honesto

A [avaliação de prontidão](../../../../spec/readiness-assessment.md) de 2026-10-11 dá o veredicto **não está pronto** para uma implementação de alta garantia. Encontrou um bloqueio ao sondar o serviço em funcionamento: qualquer pessoa com sessão iniciada podia ler a baixa por doença de um colega e o seu motivo em texto livre. Está corrigido e testado. As portas que só pessoas podem fechar continuam abertas.

## O que obtém

| Assunto | O que existe | Leia |
| --- | --- | --- |
| Ameaças em cada fronteira | Um modelo de ameaças: ativos, atores, fronteiras, o controlo tal como foi construído e o que **não foi avaliado** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Leituras não autorizadas | Uma proteção de necessidade de conhecer que classifica cada rota de leitura; um teste falha perante uma não classificada | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Adulteração e responsabilização | Um registo de auditoria só de acréscimo, encadeado por resumo, com um ponto de verificação; as entradas não contêm texto de motivo nem valor | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Dados sensíveis | Nenhum diagnóstico guardado; funções de categoria especial desligadas por defeito; um limiar para grupos pequenos | [regulatory.md](../../../../spec/regulatory.md) |
| Caminhos inseguros | A produção recusa arrancar com o início de sessão desligado, sem fonte de chaves de token, com uma ligação à base de dados em claro para outro anfitrião ou com um Redis remoto em claro | [production-readiness.md](../../../../spec/production-readiness.md) |
| Perda de dados | Cópias cifradas, um exercício de restauro executado na CI, um livro de apagamentos | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Enviesamento nas pontuações | Um modelo de impacto sobre a igualdade e um registo de cada pontuação com um rascunho de avaliação | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Uma autoavaliação dos controlos | Uma lista de verificação em que cada resposta é cumprida, parcialmente cumprida, não cumprida ou a responder por quem implementa | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Riscos em aberto, conforme registados

A fixação e a assinatura da cadeia de fornecimento, a evidência de carga, as verificações dos registos quanto a dados pessoais, os limites de taxa partilhados entre instâncias, um conjunto de papéis de base de dados de privilégio mínimo e a orientação sobre revogação de tokens são pontos em aberto da avaliação de prontidão, cada um com uma tarefa. Os testes foram escritos pelo autor; nenhum é uma revisão independente.

## O que só as pessoas podem fazer

Um teste de intrusão independente, uma auditoria de acessibilidade independente, a avaliação de impacto de quem implementa assinada, um exercício de restauro na sua plataforma e a aprovação das avaliações de impacto sobre a igualdade. O repositório declara-as como portas e não as marca como concluídas.

Veja também a [direção de informação](../direção-de-informação/index.md) e a [direção jurídica](../direção-jurídica/index.md).

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
