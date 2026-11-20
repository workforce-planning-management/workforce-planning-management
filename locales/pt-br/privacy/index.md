# Aviso de privacidade

Este aviso diz o que o sítio web do Workforce Planning Management (WPM) e a aplicação WPM guardam no seu
navegador, porquê, durante quanto tempo e como o remover. O sítio web é
<https://workforce-planning-management.github.io/>; a aplicação é o software de demonstração WPM, onde quer
que seja executado. (Atualizado em 2026-11-17, WPM-R253.)

Nenhum dos dois tem análises de utilização, publicidade, nem scripts, tipos de letra ou imagens de terceiros. Nada
do que um ou outro guarda no seu navegador é usado para o seguir de sítio em sítio.

## O sítio web

O sítio web é um conjunto de páginas estáticas. Não tem início de sessão nem formulários. O que guarda fica no seu próprio navegador:

| O quê | Onde é guardado | Nome | O que é guardado | Porquê |
| --- | --- | --- | --- | --- |
| O seu tema | O armazenamento local do seu navegador | `mxi.wpm.theme` | O nome do tema, por exemplo `dark` | Para que a página seguinte e a sua próxima visita tenham o aspeto que escolheu |
| O seu tema | Um cookie só para este sítio web | `wpm-theme` | O nome do tema, por exemplo `dark` | Para que o tema volte mesmo que o seu navegador guarde os cookies mas apague o armazenamento local, ou o contrário |
| O tamanho do texto | O armazenamento local do seu navegador | `mxi.wpm.text-size` | O nome do tamanho, por exemplo `large` | Para que a página seguinte e a sua próxima visita usem o tamanho de texto que escolheu |
| O seu idioma | Nada é guardado | | | O idioma faz parte do endereço de cada página (por exemplo `/cy-001/`), por isso não há nada a lembrar |
| Onde estava numa página | O armazenamento de sessão do seu navegador, só para este separador | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | As posições de deslocamento e a ordem das páginas que abriu neste separador | Para que Recuar e Avançar o levem de volta ao ponto onde estava. Quem os guarda é a estrutura com que o sítio web é feito (SvelteKit) |

Nenhuma escolha é guardada antes de a fazer. Até escolher, o sítio web segue a definição clara ou escura do seu
navegador e o seu tamanho de texto normal, e não guarda tema nem tamanho de texto. Uma escolha é guardada quando a faz na barra de seleção,
e escolher de novo substitui-a. O armazenamento de sessão é escrito à medida que passa de página em página.

As outras partes do sítio web não guardam nada:

- **A pesquisa** corre no seu navegador, sobre um índice que o sítio web publica, e não guarda nada. As suas palavras são colocadas
  no endereço da página (`?q=`) para que uma página de resultados possa ser partilhada; quando esse endereço é carregado a partir do alojamento web (ao
  recarregar, ou a partir de uma ligação partilhada), o alojamento recebe-o com o pedido, como recebe qualquer endereço.
- **Os controlos de prontidão** no caminho para a produção não guardam nada, e recarregar a página repõe as estimativas
  do autor.
- **Partilhar** abre o sítio que escolher (email, LinkedIn, Reddit, Bluesky ou Mastodon) só quando o escolhe. Passa então a aplicar-se
  o aviso de privacidade desse sítio.

## A aplicação WPM

A aplicação guarda os mesmos tipos de escolhas, e o cookie que mantém a sua sessão iniciada:

| O quê | Onde é guardado | Nome | O que é guardado | Porquê |
| --- | --- | --- | --- | --- |
| O seu tema | O armazenamento local do seu navegador | `mxi.wpm.theme` | O nome do tema | Para que todas as páginas tenham o aspeto que escolheu |
| O tamanho do texto | O armazenamento local do seu navegador | `mxi.wpm.text-size` | O nome do tamanho | Para que todas as páginas usem o tamanho de texto que escolheu |
| O seu idioma | O armazenamento local do seu navegador | `mxi.wpm.locale` | O código do idioma da última página que abriu, por exemplo `cy-001` | Para que um endereço sem idioma abra no seu |
| O seu idioma | Um cookie só para a aplicação | `wpm-locale` | O mesmo código de idioma | Para que o servidor possa enviar um endereço sem idioma para o seu |
| O seu início de sessão | Um cookie só para a aplicação, que os scripts da página não conseguem ler | `__Host-mxi_session` (`mxi_session` no computador de um programador) | Um número aleatório que representa a sua sessão iniciada | Para que a sua sessão se mantenha de página em página; sem ele, teria de iniciar sessão em cada página |
| O idioma do editor de conteúdos | O armazenamento local do seu navegador, só se abrir o editor de conteúdos (`/admin`) | `sveltia-cms.prefs` | As definições do editor, entre elas o seu idioma | Para que o editor abra no idioma da aplicação |
| Onde estava numa página | O armazenamento de sessão do seu navegador, só para este separador | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | As posições de deslocamento e a ordem das páginas que abriu neste separador | Para que Recuar e Avançar o levem de volta ao ponto onde estava |

Ao contrário do sítio web, a aplicação guarda o tema e o tamanho do texto logo que os mostra, antes de
escolher, e guarda o idioma de cada página que abre. As chaves de antes da mudança de nome do projeto (`mxi.hcm.theme`, `mxi.hcm.locale`) passam para os
nomes acima e são apagadas.

O cookie de início de sessão é o único item que o representa. O número aleatório que contém não significa nada por si só: a
sessão iniciada para que aponta é guardada pelo servidor da aplicação, e terminar a sessão põe-lhe fim. O que a aplicação
guarda sobre as pessoas dos seus registos é tratado na penúltima secção, mais abaixo.

## Quem o recebe

O armazenamento local e o armazenamento de sessão nunca saem do seu navegador.

Um cookie é enviado com cada pedido que o seu navegador faz ao sítio que o definiu, e a nenhum outro sítio
(`SameSite=Lax`, e `Secure` sobre HTTPS):

- O cookie `wpm-theme` do sítio web vai para o GitHub Pages, que aloja o sítio web. Nenhum programa do projeto
  o lê.
- O cookie `wpm-locale` e o cookie de início de sessão da aplicação vão para o servidor onde a aplicação corre, que os lê
  para escolher o seu idioma e para saber que tem a sessão iniciada.

O GitHub, que aloja o sítio web, recebe os pedidos que o seu navegador faz, como qualquer alojamento web.
[A declaração de privacidade do GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
diz o que regista. A organização que executa a aplicação decide o que o seu servidor regista.

## Durante quanto tempo é guardado

- O cookie `wpm-theme` do sítio web: 400 dias de calendário depois da última vez que escolheu um tema, o máximo que um navegador guarda um
  cookie.
- O cookie `wpm-locale` da aplicação: 365 dias de calendário depois da última vez que abriu uma página.
- O cookie de início de sessão: até terminar a sessão ou fechar o navegador.
- O armazenamento local: até o apagar.
- O armazenamento de sessão: até fechar o separador.

## Como o remover

Apague os dados do sítio nas definições do seu navegador (cookies e dados de sítios), para o sítio web, para a aplicação ou
para ambos. O sítio web passa então a comportar-se como numa primeira visita; a aplicação pede-lhe que inicie sessão de novo. Terminar a sessão na
aplicação remove o cookie de início de sessão.

## Porque não há um aviso de consentimento

Cada item é guardado para lhe prestar o serviço que pediu: o sítio web ou a aplicação tal como os configurou, no
seu idioma e, na aplicação, com a sessão iniciada. É esse o tipo de armazenamento que as regras do Reino Unido e da UE sobre cookies (o
Regulamento sobre Privacidade e Comunicações Eletrónicas, do Reino Unido, e a Diretiva relativa à privacidade e às comunicações eletrónicas, da UE) tratam como estritamente necessário, o que
não exige consentimento. Esta é a leitura que o projeto faz dessas regras, e não aconselhamento jurídico.

## Os registos que a aplicação guarda

Este aviso trata do que é guardado no seu navegador. A aplicação também guarda no seu servidor registos sobre pessoas (trabalhadores,
candidatos e os seus contactos). Quem executa a aplicação é responsável por esses registos e
redige o aviso de privacidade para as pessoas que neles constam
([a lista de verificação da avaliação](../../../spec/governance/assessment-checklist.md), em inglês, pergunta 3.10). O software de demonstração
guarda apenas dados sintéticos.

## Perguntas

Pergunte no repositório do projeto, <https://github.com/workforce-planning-management/workforce-planning-management>,
ou escreva ao responsável pela manutenção aí indicado.
