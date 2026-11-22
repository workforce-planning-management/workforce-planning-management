# Planeamento de Pessoal

**Uma plataforma de RH de código aberto que cobre o ciclo de vida do trabalhador, da contratação à reforma.**

O Workforce Planning Management (WPM) reúne recrutamento, tempo e ausências, desenvolvimento dos trabalhadores, planeamento de pessoal e processamento salarial numa só aplicação. É software de demonstração, pensado para avaliação e como desenho de referência.

Idioma: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Software de demonstração, ainda não pronto para uso em produção.** O WPM usa apenas dados sintéticos e os seus cálculos legais são ilustrativos. Não introduza dados pessoais reais. [O caminho para a produção](path-to-production/index.md) diz o que o uso em produção exigiria e quão pronta está cada parte. Veja também [a posição regulatória](../../spec/regulatory.md).

[Executar a demonstração](start/index.md) [Documentação por função](for/index.md) [Código-fonte no GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Benefícios

Os dados de RH estão muitas vezes dispersos por ferramentas separadas, como caixas de correio de candidatos, folhas de cálculo de ausências e quadros de escalas. Cada transferência entre elas acrescenta trabalho manual e risco de erro. O WPM foi concebido para reduzir essas transferências.

- **Sistema único.** Os registos de integração, ausências, formação e remuneração ficam numa só aplicação, o que reduz a reintrodução de dados.
- **Minimização de dados.** Os dados que não devem ser guardados não têm campo, e os dados que não devem ser divulgados não têm rota na API. Por exemplo, não existe campo de diagnóstico e as respostas ao inquérito rápido não têm autor.
- **Regras de relato.** Valores desconhecidos não são relatados como zero, os agregados não identificam pessoas e as estimativas indicam em que se baseiam.
- **Localização.** A aplicação está disponível em 25 idiomas e regiões e as fichas por função em 21, incluindo idiomas escritos da direita para a esquerda. As regras de cada país são guardadas como dados. As traduções foram feitas por IA e não foram revistas por falantes nativos.
- **Rastreabilidade.** O código-fonte é aberto e cada comportamento está ligado a um requisito, uma decisão de desenho e uma tarefa numerados.
- **Implementação independente.** A pilha é uma API JSON em Rust, um cliente SvelteKit e PostgreSQL. Compila a partir de um clone novo e as integrações com aplicações irmãs são opcionais.

## Funcionalidades

- **Recrutamento e registos.** Pedidos de vaga, funil de candidatos, integração, fichas de trabalhadores e organigrama.
- **Tempo e escalas.** Tempo e ausências, turnos com verificação do tempo de trabalho, escala de prevenção com trocas e lembretes, e um diretório de pessoal.
- **Desenvolvimento.** Avaliações, feedback de 360°, aprendizagem, análise de lacunas de competências, recomendações de tempo de formação e formação obrigatória.
- **Planeamento e relatórios.** Planeamento estratégico de pessoal, indicadores e perspetivas, e um painel da direção executiva.
- **Entradas e saídas.** Listas de verificação datadas e uma passagem de pasta no último dia, com registo de auditoria.
- **Comunicação.** Avisos com destinatários, ligações e contagem de leituras.
- **Direitos e bem-estar dos trabalhadores.** Sugestões de bem-estar, inquérito rápido anónimo, adaptações, acesso aos dados, apagamento e conservação.
- **Remuneração.** Processamento salarial e compensação, com cálculos legais ilustrativos.

## Comparação

A tabela compara o WPM com três categorias de sistemas de RH. É uma orientação geral, não uma medição comparativa. Veja também a [comparação por produto](../../docs/compare/index.md).

| | WPM | Suites completas (Workday, SAP SuccessFactors, BambooHR) | RH de código aberto (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Ferramentas pontuais (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Âmbito** | Ciclo de vida completo, da contratação à reforma | Âmbito semelhante numa só plataforma | As mesmas funções centrais | Uma só função |
| **Licenciamento** | Código aberto | Comercial | Código aberto | Comercial |
| **Ponto forte** | Minimização de dados, desenho rastreável, análise de lacunas de competências e de tempo de formação | Uma plataforma integrada e com suporte | Mais integrações, localizações e implementações | Profundidade numa só função |
| **Adequação típica** | Avaliação, desenho de referência ou desenvolvimento adicional | Organizações que precisam de um sistema com suporte | Organizações que querem código aberto em produção | Organizações que precisam de profundidade numa função e integram o resto |

## Público-alvo

- **Dirigentes.** As [fichas por função](for/index.md) explicam o que o WPM oferece a cada função de liderança e como ler os resultados.
- **Equipas de RH.** O [tutorial da demonstração](start/index.md) percorre cada página com dados fictícios.
- **Programadores e arquitetos.** Veja a [especificação](../../spec/index.md) e o [guia de instalação](../../INSTALL.md).
- **Avaliadores.** A [avaliação de prontidão](../../spec/readiness-assessment.md) lista as lacunas entre o WPM e o uso em produção.

## Obter ajuda

- **Comunicar um erro ou fazer uma pergunta** no [registo de problemas](https://github.com/workforce-planning-management/workforce-planning-management/issues). Indique a página, o que fez, o que esperava e o que aconteceu. Não inclua dados pessoais reais.
- **Comunicar um problema de segurança** por correio eletrónico, como descrito na [política de segurança](../../SECURITY.md), e não num problema público.
- **Aprender uma tarefa na aplicação.** A ligação **Ajuda** no cabeçalho da aplicação abre a página de ajuda e os guias para trabalhadores, gestores e RH. Cada mensagem de erro também liga a ela.

## Próximos passos

1. **Leia a ficha da sua função.** Cada ficha descreve o que o WPM oferece e o que continua em aberto.
2. **Execute a demonstração.** A visita corre numa máquina local com dados fictícios.
3. **Envie comentários.** Veja [como comentar](../../RFC.md) e [como contribuir](../../CONTRIBUTING.md).

[Executar a demonstração](start/index.md) [Documentação](../../index.md)

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-001/index.md`), prevalece a versão em inglês.
