# O caminho para a produção

O WPM é software de demonstração. Esta página diz o que significaria usá-lo em produção e enumera o que separa o WPM desse uso. Cada item tem uma estimativa de quão pronto está, de 0 a 100 por cento.

## O que significa produção

Uso em produção significa que uma organização depende do WPM para trabalho real:

- os registos são de pessoas reais: trabalhadores, candidatos e os seus contactos de emergência;
- a remuneração calculada pelo WPM é paga, e aplica-se-lhe a lei do local de trabalho;
- as pessoas iniciam sessão todos os dias, e parte do que leem é sensível, como as ausências por doença;
- a organização responde perante um regulador, um auditor e as pessoas dos registos pela forma como é gerida.

Esta página mede o WPM face ao caso mais exigente para o qual foi avaliado: uma grande organização médica ou governamental ([a avaliação de prontidão](../../../spec/readiness-assessment.md), em inglês). Uma organização mais pequena pode precisar de menos em alguns itens, mas precisa dos mesmos tipos de evidência.

A produção exige três tipos de evidência, e o software, por si só, fornece apenas o primeiro:

1. **Software** que faz o trabalho com segurança, com testes que o demonstram.
2. **Uma entidade implementadora** que o aloja e opera: cópias de segurança, monitorização, suporte e resposta a incidentes.
3. **Pessoas que não o autor** que o verificaram: um responsável por testes de intrusão, um auditor de acessibilidade, tradutores e o encarregado da proteção de dados da entidade implementadora.

## Como ler as estimativas

- Uma estimativa é o juízo do autor em 2026-11-13, formado a partir da especificação e dos testes. Não é uma medição nem uma certificação.
- 0 por cento significa que ainda não existe nada. 100 por cento significa construído, testado e verificado por alguém independente numa implementação real. Nenhum item está a 100 por cento.
- Cada item diz em que assenta a sua estimativa e o que falta.
- Não se calcula a média das estimativas. Basta um item com uma estimativa baixa para manter o WPM fora da produção.

## O software

- **75%** **Início de sessão e acesso por necessidade de conhecer.** O início de sessão está ligado por predefinição, e o serviço recusa arrancar em produção sem ele (WPM-R72). As leituras limitam-se às pessoas com necessidade de conhecer (WPM-T188), e um token tem uma validade máxima (WPM-T200). Assenta em: os testes de imposição executados na CI. Falta: não existe lista de revogação de tokens, e não foi realizado nenhum teste de intrusão independente.
- **70%** **Um registo de auditoria completo e inalterado.** O registo de auditoria só admite acréscimos e está encadeado por hash, e pode ser exportado para um sistema de monitorização (WPM-T189, WPM-T197). Assenta em: testes que foram partidos de propósito para provar que falham. Falta: um final de cadeia truncado não é detetado, e o proprietário da base de dados ainda pode desativar os triggers.
- **70%** **Proteção de dados desde a conceção.** As pessoas são identificadas por referências, não há colunas demográficas, e existe acesso do titular, apagamento e conservação para cada tipo de registo. Assenta em: os testes e a secção 1 da avaliação de prontidão. Falta: a avaliação e o fundamento jurídico da própria entidade implementadora (abaixo).
- **75%** **Ligações e cópias de segurança cifradas.** Em produção, o serviço recusa uma ligação à base de dados em texto simples e interrompe por tempo limite um pedido que fique pendente (WPM-T191), e as cópias de segurança são cifradas (WPM-T192). Assenta em: os testes de ambos. Falta: a gestão de chaves e certificados cabe à entidade implementadora.
- **10%** **Cálculos legais.** Os cálculos legais são ilustrativos, e o imposto sobre a folha de pagamentos é um esboço ([a posição regulamentar](../../../spec/regulatory.md), em inglês). Assenta em: a especificação, que o diz. Falta: as regras de cada local de trabalho, escritas e verificadas por uma pessoa qualificada. Não pague a ninguém a partir do WPM.
- **50%** **Escala.** Num único portátil, as páginas do diretório e a vista de capacidade não falham com 50 000 trabalhadores, e as vistas de planeamento recusam mais de 10 000 (WPM-T196, WPM-T230). Assenta em: uma execução medida ([BENCHMARKS.md](../../../BENCHMARKS.md), em inglês). Falta: a vista de capacidade continua a crescer em proporção ao efetivo, e nada foi medido no hardware de uma entidade implementadora.
- **65%** **Cadeia de fornecimento.** Cada ação da CI está fixada a um commit e o histórico é analisado em busca de segredos. A imagem do contentor está assinada e inclui a sua proveniência, e isto foi executado para a versão v0.1.2 (WPM-T194, WPM-T195). Assenta em: essa execução de publicação. Falta: o pacote da imagem é privado, pelo que ainda ninguém fora do projeto o pode verificar.
- **60%** **Monitorização.** Cada resposta inclui um identificador de pedido, e as linhas de registo não incluem cadeia de consulta nem identificador pessoal (WPM-T197). Assenta em: os testes do registo de pedidos. Falta: os painéis e os alertas cabem à entidade implementadora, e os limites de taxa aplicam-se apenas a uma instância (WPM-T198).
- **60%** **Cópia de segurança e restauro.** Cópias de segurança cifradas, um exercício de restauro executado na CI e um manual de procedimentos. Assenta em: o exercício. Falta: um exercício de restauro na própria plataforma da entidade implementadora nos últimos 90 dias de calendário (barreira WPM-G4).
- **40%** **Verificações de acessibilidade.** São executadas verificações automáticas face a WCAG 2.2 AA na aplicação e neste sítio, nos temas claro e escuro. Assenta em: os testes de Playwright e axe. Falta: a auditoria independente (abaixo); não foi usado nenhum leitor de ecrã.
- **20%** **Traduções.** A aplicação tem 17 locales e a documentação por função tem 21. Assenta em: o texto existe. Falta: as traduções foram escritas por uma IA, e ninguém as reviu profissionalmente (WPM-T155).

## Verificações por pessoas que não o autor

- **0%** **Teste de intrusão independente.** Não foi realizado nenhum teste de intrusão a uma instância implementada (WPM-T201). Assenta em: a tarefa está em aberto. Falta: o teste e a correção do que encontrar.
- **0%** **Auditoria de acessibilidade independente.** Não foi feita nenhuma auditoria face a WCAG 2.2 AA com pessoas que usam tecnologias de apoio (WPM-T202). Assenta em: a tarefa está em aberto. Falta: a auditoria e a correção do que encontrar.
- **25%** **Modelo de ameaças.** Existe um modelo de ameaças inicial ([threat-model.md](../../../spec/governance/threat-model.md), em inglês), escrito pelo autor. Assenta em: o documento. Falta: uma equipa independente refazê-lo para uma implementação real.

## A parte da entidade implementadora

- **20%** **Avaliação de impacto sobre a proteção de dados.** Existe um conjunto de documentos para adaptar: uma avaliação de impacto, um fluxo de dados, um registo das atividades de tratamento e um calendário de conservação ([spec/governance](../../../spec/governance/dpia.md), em inglês). Assenta em: o conjunto de documentos. Falta: não existe entidade implementadora, pelo que nenhum encarregado da proteção de dados aprovou uma avaliação, um fundamento jurídico, um calendário de conservação ou um acordo com o subcontratante (barreira WPM-G5).
- **15%** **Avaliações de impacto sobre a igualdade.** Existem um modelo e um registo ([equality-impact](../../../spec/equality-impact/template.md), em inglês). Assenta em: o modelo. Falta: a entidade implementadora escrever e aprovar uma avaliação para o seu próprio uso.
- **40%** **Alojamento e operação.** Existem uma configuração de produção, uma imagem de contentor e documentos para a implementação e as atualizações. Assenta em: esses documentos e o exercício de restauro. Falta: nada foi executado num servidor reforçado, e uma entidade implementadora tem de operar o serviço.
- **10%** **Suporte.** Existem um sistema de registo de problemas e uma política de segurança ([SECURITY.md](../../../SECURITY.md), em inglês). Assenta em: esses dois. Falta: não existe contrato de suporte, nível de serviço, escala de prevenção nem organização por trás do WPM.

## O que esta página não afirma

A página não regista nenhuma decisão de entrada em funcionamento. Essa decisão cabe a uma entidade implementadora e é tomada fora do WPM. As estimativas abrangem apenas os itens enumerados, e a lista que a própria avaliação de prontidão faz do que não verificou aplica-se também aqui. Quando um sistema clínico estiver ligado, uma revisão de segurança clínica é decisão da entidade implementadora.
