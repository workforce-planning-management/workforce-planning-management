# Teste de segurança independente

**A pergunta que provavelmente faz:** como levamos pessoas que não escreveram este software a atacá-lo, para onde as orientamos e o que fazemos com o que encontrarem?

Não foi feito nenhum teste independente. O [modelo de ameaças](../../../../spec/governance/threat-model.md) foi escrito pelo mesmo assistente que escreveu o software e ninguém mais o reviu; cada controlo que nomeia foi exercitado por esse mesmo autor. A [avaliação de prontidão](../../../../spec/readiness-assessment.md) lista um teste de intrusão independente como uma porta que só as pessoas podem fechar (WPM-T201, **por fazer**). Esta página é um documento de enquadramento para quem encomenda um. Não é um plano de testes de um testador e não o substitui.

## O que dar aos testadores

| Item | Onde está | Notas |
| --- | --- | --- |
| Uma implementação para atacar | A sua: [guia de implementação](../../../../spec/operations/deployment.md) | Definições de produção com o início de sessão **ligado**, um fornecedor de identidade de teste e **apenas dados sintéticos**. Não teste contra registos de pessoas reais |
| O código-fonte | Este repositório | Um teste com o código-fonte é mais completo do que um de caixa preta; diga qual quer |
| O modelo de ameaças | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Vinte e quatro ameaças, o controlo tal como foi construído e as lacunas. Entregue-o **depois** de uma primeira passagem de caixa preta se quiser ver o que encontram sem pistas |
| A descrição da API | `GET /api-docs/openapi.json` | Todas as rotas, com a `Idempotency-Key` em cada operação que altera algo |
| O desenho do início de sessão | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Perfis, atributos e o facto de a política decidir apenas por ação |
| Identidades de teste | Cria-as você | Pelo menos: uma pessoa sem atributos, um gestor direto, um gestor fora da cadeia, RH, salários, uma administração e um par de serviço |

## O que lhes pedir que examinem

O autor não conseguiu verificar isto. Cada linha nomeia a entrada do próprio modelo de ameaças, para que uma constatação possa ser rastreada.

| Área | O que experimentar | Ameaça |
| --- | --- | --- |
| Tokens | Confusão de algoritmos; um conjunto de chaves que roda enquanto há pedidos em curso; um token expirado ou ainda não válido; um token para outra audiência | T1, T2 |
| Necessidade de conhecer | Cada rota de leitura como um chamador sem atributos e como um vizinho; uma rota que a tabela de classificação não conhece; uma escrita que a política permite mas que o tratador deveria recusar | T3, T4, T6 |
| Atributos privilegiados | Se um atributo num token pode ser acrescentado, repetido ou herdado; quem o pode emitir; uma administração partilhada ou de longa duração | T24 |
| Injeção | Fuzzing de cada parâmetro, segmento de caminho e campo do corpo; as instruções de apagamento e de conservação são construídas a partir de identificadores tipados e foram lidas, não submetidas a fuzzing | T12 |
| O cliente de navegador | Tratamento de sessões pelo servidor da interface, como troca uma sessão por um token, scripting entre sites através de texto armazenado, contornar a política de segurança de conteúdo e a página do gestor de conteúdos que carrega um script de uma rede pública sem versão fixada | T13, T14 |
| Pedidos de saída | Falsificação de pedidos do lado do servidor através de qualquer URL configurado (conjuntos de chaves, os serviços irmãos, a cache partilhada) | T15 |
| Limites | O limite do corpo, o tempo limite do pedido, os limites de taxa por instância e as vistas que carregam para memória os registos de uma organização inteira | T16, T17 |
| Registos | Se algum dado pessoal, token ou cadeia de consulta chega a um registo | T19 |
| Chaves de idempotência | Uma chave usada por dois chamadores; a mesma chave com corpo, caminho ou consulta alterados; uma chave quase válida; um corpo muito grande; dois pedidos em corrida sob a mesma chave (a reivindicação está fundamentada, não testada em corrida); se alguém além do seu chamador pode ler uma resposta armazenada | novo |
| O registo de auditoria | Qualquer forma de alterar, remover ou saltar uma entrada; o ponto de verificação da cadeia | T7 |
| Cópias de segurança | Restaurar a partir de uma cópia adulterada ou com a chave errada; se uma pessoa apagada regressa | T10, T11 |
| Cadeia de fornecimento | Fixação das ações de construção, análise de segredos, assinatura de imagens | T20, T21 |

## Regras a acertar antes de começarem

Preencha-as com as suas próprias equipas de segurança e jurídica. Nenhuma é decidida aqui.

- **Autoridade:** quem assina que o teste está autorizado, e para que sistemas e endereços.
- **Janela:** as datas e horas, contadas em dias de calendário, e a quem ligar para o parar.
- **Âmbito:** o que está dentro (a API, a interface, o fluxo de início de sessão, o script de cópia de segurança) e o que está fora (o serviço do próprio fornecedor de identidade, a sua rede, tudo o que é partilhado com outros sistemas).
- **Dados:** apenas sintéticos. Se um testador encontrar dados pessoais reais, quem é informado e como são tratados.
- **Testes destrutivos:** se são permitidos a negação de serviço, as escritas em massa e o apagamento, e onde.
- **Relatório:** o formato, a escala de gravidade, quem o recebe, durante quanto tempo o podem guardar e como são tratados os segredos que encontrarem.
- **Reteste:** que o contrato inclui verificar as correções.

## O que fazer com as constatações

1. Registe cada uma como uma tarefa em [tasks.md](../../../../spec/tasks.md) com a referência do testador, a evidência e a gravidade que atribuíram. Uma constatação contestada permanece registada como contestada, com o motivo.
2. Corrija com um teste que falhe primeiro. Se a correção for uma regra, acrescente-a à coluna de lacunas do modelo de ameaças e feche-a aí apenas quando o reteste passar.
3. Não marque WPM-T201 como concluída até ter o relatório, cada constatação ter um responsável e um reteste estar completo. Atualize a [avaliação de prontidão](../../../../spec/readiness-assessment.md) e a lista de portas (WPM-T203) na mesma alteração.
4. Mantenha o relatório fora do repositório se contiver algo explorável; ligue para onde está guardado.

## O que significa um relatório limpo e o que não significa

Um teste sem constatações mostra que estes testadores, nesta janela, com este âmbito e estas identidades, não encontraram nada. Não mostra a ausência de falhas, não cobre uma configuração que alterou depois e não substitui as outras portas: uma auditoria de acessibilidade, a sua avaliação de impacto sobre a proteção de dados assinada pelo seu responsável e um exercício de restauro na sua plataforma.

## O que esta página não faz

Não nomeia um fornecedor, não fixa um preço nem promete uma norma. Que esquema ou acreditação o seu setor espera de um testador é decisão sua.

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
