# Para a direção de operações (COO)

**A pergunta que provavelmente faz:** temos as pessoas certas, no lugar certo, na hora certa, e o que falha se alguém estiver ausente?

## O que obtém

| Quer saber | Onde o WPM responde | Leia |
| --- | --- | --- |
| Se uma equipa consegue cobrir ausências | Substitutos por pessoa, a escala de prevenção com trocas e lembretes, e uma vista de cobertura que diz com clareza quando ninguém está disponível | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Tempo, assiduidade e licenças | Registos de tempo, pedidos de licença com circuito de aprovação, acumulações e turnos | [workforce-management.md](../../../../spec/workforce-management.md) |
| Se os limites de tempo de trabalho são respeitados | Salvaguardas consultivas do tempo de trabalho; avisam e não bloqueiam | WPM-R27, o mesmo ficheiro |
| Capacidade por competência, mês a mês | Conjuntos de competências, oferta e procura dos programas, compromissos de parceiros, o conjunto restritivo e uma verificação de arranque que é uma sugestão, com decisões de arranque registadas | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| De que vamos precisar e onde está a lacuna | Planos de efetivos, previsão e análise de lacunas por departamento e por função | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Que contratos terminam em breve | Uma data de fim por base da contratação, prorrogações com histórico, lembretes antes do fim, uma lista de contratações sem data de fim e planos de conversão | [engagements.md](../../../../spec/engagements.md) |
| Onde o pessoal contingente é uma parte grande | A parte dos ETC que são prestadores e a que é a termo, em separado, e o financiamento que termina mês a mês | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Entradas e saídas | Listas de verificação datadas e uma passagem de pasta no último dia | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Lacunas de competências e tempo de formação | Análise de lacunas com a evidência em que assenta uma recomendação | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Como ler a vista de capacidade

A capacidade é planeada por conjunto de competências, não por pessoa. Cada mês de um conjunto dá um de quatro resultados, e uma competência não declarada é *desconhecida*, não um défice. A verificação de arranque é uma sugestão, e uma pessoa regista a decisão de começar. O planeamento por cadeia crítica e caminho crítico não está aqui: pertence ao project-portfolio-management.

## Limites

- A parte contingente ainda não tem tendência: precisa de instantâneos de efetivos separados por base (WPM-T139, não construído).
- Um conjunto, na vista contingente, é o departamento até os planos definirem conjuntos.
- O planeamento de turnos e os cálculos do imposto sobre salários não substituem os seus sistemas de escalas ou de salários.
- Várias vistas carregam em memória os registos de uma organização; não foi medido nenhum valor de carga ([readiness-assessment.md](../../../../spec/readiness-assessment.md), lacuna 8).

Veja também a [direção de pessoas](../chief-people-officer/index.md) e a [direção executiva](../chief-executive-officer/index.md).

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
