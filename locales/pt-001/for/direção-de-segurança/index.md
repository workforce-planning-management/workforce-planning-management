# Para a direção de segurança

**A pergunta que provavelmente faz:** como está protegido, como saberíamos se fosse atacado ou mal utilizado e o que ninguém testou?

## O que obtém

| Assunto | O que existe | Leia |
| --- | --- | --- |
| Ameaças | Um modelo de ameaças: ativos, atores, fronteiras de confiança, o controlo tal como foi construído e o que **não foi avaliado** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Autenticação | Início de sessão ligado por defeito; a produção recusa arrancar se estiver desligado ou sem fonte de chaves de token; início de sessão com diretório empresarial (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Autorização | A política decide apenas por ação; uma tabela classificada decide a necessidade de conhecer em cada rota de leitura, e um teste de análise do código-fonte falha perante uma rota não classificada | [auth.md](../../../../spec/auth.md) |
| Auditoria | Registo só de acréscimo, encadeado por resumo, com um ponto de verificação; as entradas não levam texto de motivo nem valor | [audit.md](../../../../spec/audit.md) |
| Transporte e navegador | Cabeçalhos de segurança em cada resposta da API, uma lista de CORS explícita que impede o arranque se faltar, `no-store` em `/api` | WPM-R73 em [production-readiness.md](../../../../spec/production-readiness.md) |
| Abuso | Limites de taxa por chamador, mais rígidos para apagamento, importação, exportação e rotas de token | WPM-R74, o mesmo ficheiro |
| Configuração insegura | O arranque recusa uma ligação à base de dados ou ao Redis em claro para outro anfitrião e uma origem CORS em falta | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Dados em repouso | Cópias cifradas; a cache partilhada guarda apenas consultas não pessoais e falha em aberto | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Cadeia de fornecimento | Verificações de licenças e proibições com `cargo deny`, crates partilhados incluídos, uma lista de materiais de software em cada publicação | [production-readiness.md](../../../../spec/production-readiness.md) |
| Incidentes | Um manual operacional das primeiras ações; acrescenta contactos e deveres de notificação | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Autoavaliação | Uma lista de verificação de controlos: cumprida, parcialmente cumprida, não cumprida, a responder por quem implementa | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Lacunas conhecidas, conforme registadas

- Não foi feito um teste de intrusão independente. Os testes foram escritos pelo autor.
- As Actions estão fixadas por etiqueta, não por resumo, e as imagens não estão assinadas.
- Os limites de taxa são por instância.
- A aplicação usa o papel de base de dados que lhe for dado; não é fornecido um conjunto de papéis de privilégio mínimo.
- A revogação de tokens e a sua idade máxima são do seu fornecedor de identidade.
- O caminho do gestor de conteúdos carrega um script de uma CDN pública sem versão fixada nem resumo de integridade.
- Os registos não foram verificados quanto a dados pessoais, e o registo de auditoria não é exportado para um sistema de monitorização.
- O Redis gerido (cluster, sentinela, utilizadores de controlo de acesso) não foi experimentado.

## O que só as pessoas podem fazer

Encomendar o teste de intrusão independente, rever o modelo de ameaças para o seu ambiente, executar um exercício de restauro na sua plataforma e definir a proteção de ramos na plataforma de alojamento. Veja a [avaliação de prontidão](../../../../spec/readiness-assessment.md), cujo veredicto é **não está pronto** para uma implementação de alta garantia.

Veja também as páginas da [direção de riscos](../direção-de-riscos/index.md) e da [direção de informação](../direção-de-informação/index.md).

## Quão atual é isto

Escrito em 2026-10-11 a partir da especificação, e não executando o software para esta página. Os números que mudam (testes, migrações, tarefas concluídas) não são copiados aqui; leia-os em [implementation-status.md](../../../../spec/implementation-status.md), gerado por um script. O WPM é software de demonstração com dados sintéticos: esta página diz o que ele faz e o que não faz, e não afirma que seja adequado à sua organização. Nada aqui é aconselhamento jurídico.

> Esta tradução foi escrita por uma IA e não foi revista por um falante nativo. Se diferir da versão em inglês (`locales/en-gb-oxendict/for/`), prevalece a versão em inglês.
