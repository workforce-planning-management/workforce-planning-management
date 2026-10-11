# Para a direção de pessoas (CPO)

**A pergunta que provavelmente faz:** isto apoia todo o ciclo de vida do colaborador, e posso usá-lo sem guardar sobre as pessoas mais do que devo?

## O que obtém

| Fase | O que o WPM guarda | Leia |
| --- | --- | --- |
| Recrutamento | Pedidos de vaga, um fluxo de candidaturas, uma reserva de candidatos, listas de verificação de integração | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| O registo | Um registo de emprego por trabalhador, o organigrama, linhas hierárquicas, grupos, transferências entre organizações | [hr-core.md](../../../../spec/hr-core.md) |
| Autosserviço | A pessoa vê e atualiza os seus próprios dados, pede trabalho flexível, dá o pré-aviso e vê o seu tempo livre | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Vida profissional | Licenças, tempo, turnos, benefícios, avisos de bem-estar, ajustamentos razoáveis, avaliações ergonómicas, o pulso anónimo | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Desenvolvimento | Avaliações, avaliações de 360 graus, cursos através do registo de cursos, avaliações de competências, mentoria, planos de desenvolvimento, sucessão, primeiras carreiras | [talent-development.md](../../../../spec/talent-development.md) |
| Remuneração | Processamentos salariais, recibos de vencimento, comparações salariais, consulta de uma tabela salarial nacional, níveis de função | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Pessoal não permanente | Datas de fim por base, prorrogações, avaliações de estado, planos de conversão decididos por uma pessoa, financiamento ao lado do fim do contrato | [engagements.md](../../../../spec/engagements.md) |
| Saída | Demissão com pré-aviso, passagem de pasta no último dia, conservação por tipo de registo, apagamento | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Como protege as pessoas

- **Necessidade de conhecer na leitura.** Um gestor vê a sua equipa direta, uma pessoa vê-se a si própria, os RH veem o que precisam. Um desconhecido com sessão iniciada não vê nada sobre um colega ([auth.md](../../../../spec/auth.md)).
- **Nenhum diagnóstico é guardado.** A baixa por doença regista um tipo, não um motivo.
- **As funções de categoria especial estão desligadas por defeito.** A monitorização da igualdade e os requisitos de saúde no trabalho precisam de uma base legal registada, são voluntários ou ficam com o papel de saúde ocupacional, e só mostram grupos acima de um limiar. Nunca são entrada de qualquer pontuação ou decisão sobre uma pessoa.
- **As pontuações aconselham.** A prontidão para a sucessão, as lacunas de competências e as previsões sugerem; uma pessoa decide, e decisões como converter um prestador são registadas como decisão de uma pessoa, nunca como resultado automático.
- **O titular pode consultar e apagar.** A exportação de acesso e o apagamento cobrem os registos acima, com um livro para que um restauro não traga de volta uma pessoa apagada.

## Limites

- Ainda não construídos: uma interface para planos de conversão e financiamento, a tendência contingente e a página «Os meus dados» do trabalhador. Consulte [tasks.md](../../../../spec/tasks.md) para o estado atual.
- As traduções são assistidas por máquina; uma revisão profissional é uma porta aberta.
- Os acordos coletivos e os órgãos reconhecidos são apenas investigação e proposta ([worker-unions.md](../../../../spec/worker-unions.md)).
- O imposto sobre salários é um marcador provisório.

Veja também a [direção jurídica](../chief-legal-officer/index.md) e a [direção executiva](../chief-executive-officer/index.md).

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
