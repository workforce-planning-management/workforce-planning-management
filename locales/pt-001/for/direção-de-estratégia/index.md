# Para a direção de estratégia

**A pergunta que provavelmente faz:** o nosso quadro de pessoal está moldado para servir a estratégia, de que vamos precisar e o que devemos fazer quanto à lacuna?

## O que obtém

| Quer saber | Onde o WPM responde | Leia |
| --- | --- | --- |
| O que uma função exige | Perfis de função com as competências exigidas e um nível mínimo, para que a lacuna se meça face a uma função e não a uma pessoa | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| De que vamos precisar | Planos de efetivos como mundos de rascunho, vários lado a lado, com pressupostos declarados e a procura por departamento, função e período | WPM-R36, o mesmo ficheiro |
| Onde está a lacuna | Oferta projetada face à procura planeada: uma lacuna de efetivos e uma de competências, com alavancas listadas como sugestões e a sua evidência | WPM-R37 |
| Se serve a estratégia | Objetivos ligados à procura: objetivos sem recursos, procura sem alinhamento, funções críticas sem banco | WPM-R38 |
| Se podemos entregar um programa | Conjuntos de competências, procura dos programas, compromissos de parceiros, o conjunto restritivo, uma verificação de arranque que é uma sugestão | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Quanto dependemos de pessoal não permanente | A parte dos ETC que são prestadores e a que é a termo, em separado, e quanto financiamento termina em cada um dos próximos doze meses | `GET /api/workforce-intelligence/contingent` (WPM-T109, parcial) |
| O que pretendemos fazer quando os contratos terminam | Planos de conversão: a decisão de uma pessoa de converter, prorrogar ou terminar, com o financiamento ao lado do fim do contrato | [engagements.md](../../../../spec/engagements.md) |
| O que mais está planeado | O roteiro e a lista de tarefas em aberto | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Como ler uma previsão

- É uma regra declarada aplicada a entradas declaradas, não um modelo, e cada resultado nomeia os seus pressupostos.
- Com pouco histórico de efetivos, diz `insufficient_history` em vez de extrapolar.
- Um plano só guarda efetivos hipotéticos. Nunca lê nem altera o registo de uma pessoa.
- As alavancas são sugestões. Quem planeia escolhe, e o software não decide nada.

## Limites

- A alavanca «converter pessoal contingente em permanente» nos planos está especificada (WPM-R63) e ainda não está no código.
- A parte contingente ainda não tem tendência: precisa de instantâneos separados por base (WPM-T139, não construído).
- O planeamento por cadeia crítica e caminho crítico pertence ao project-portfolio-management, não a esta aplicação.
- Ainda não há interface para o financiamento nem para os planos de conversão.

Veja também as páginas da [direção executiva](../direção-executiva/index.md) e da [direção de operações](../direção-de-operações/index.md).

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
