# Ar gyfer y Prif Swyddog Technoleg

**Y cwestiwn rydych chi'n debygol o'i ofyn:** a yw wedi'i adeiladu'n dda, a all fy nhîm ei newid yn ddiogel, a sut ydw i'n gwybod?

## Beth gewch chi

| Pryder | Beth sy'n bodoli | Darllenwch |
| --- | --- | --- |
| Pentwr | Rust gyda Loco (Axum, SeaORM) a PostgreSQL 18 ar gyfer yr API; SvelteKit 2 gyda Svelte 5 a TypeScript ar gyfer y cleient | [architecture.md](../../../../spec/architecture.md) |
| Rheol dylunio | Craidd pur: mae rheolau'n rhydd o gronfa ddata a chlociau, wedi'u profi'n drylwyr, ac yna'n cael eu cysylltu â rheolyddion | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Manyleb yn gyntaf | Mae newid yn ofyniad, penderfyniad, tasg, cod a phrofion gyda'i gilydd; mae pob cofnod tasg yn dweud beth **na** chafodd ei wirio | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Profion | Profion uned ar y rheolau, profion ceisiadau â chronfa ddata, deuaidd gorfodi a redir gyda mewngofnodi ymlaen, profion uned a phorwr y pen blaen. Gwiriadau treiglo: torri'r cod yn fwriadol a chadarnhau bod prawf yn methu | [testing.md](../../../../spec/testing.md) |
| Cyfrifiadau | Wedi'u cynhyrchu o'r ffeiliau, byth wedi'u copïo â llaw | [implementation-status.md](../../../../spec/implementation-status.md) |
| Cadwyn gyflenwi | `cargo deny` ar gyfer trwyddedau a gwaharddiadau, crates a rennir wedi'u cynnwys, bil defnyddiau meddalwedd wrth ryddhau | [production-readiness.md](../../../../spec/production-readiness.md) |
| Contract API | OpenAPI, a phrawf sy'n methu os nad oes dogfen ar gyfer llwybr cofrestredig | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Rheoli mynediad | Mae'r polisi'n penderfynu yn ôl gweithred yn unig; mae trinwyr a thabl llwybrau wedi'i ddosbarthu yn penderfynu'r angen i wybod, ac mae sgan o'r ffynhonnell yn methu ar lwybr darllen heb ei ddosbarthu | [auth.md](../../../../spec/auth.md) |
| Perfformiad | Mae [BENCHMARKS.md](../../../../BENCHMARKS.md) yn cofnodi'r hyn a fesurwyd; nid oes prawf llwyth | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Dyled peirianneg hysbys

- Mae sawl golwg yn llwytho sefydliadau cyfan i'r cof.
- Rhesymwyd am gloeon rhes ar gynlluniau trosi, ni phrofwyd hwy â phrawf.
- Mae CI, rhyddhau a'r sgan deinamig wedi'u hysgrifennu ond heb redeg ar y platfform cynnal.
- Mae Actions wedi'u pinio gan dag, nid gan grynodeb, ac nid yw delweddau wedi'u llofnodi.
- Mae rhai profion Redis â chyfrinair a TLS angen newidynnau ac awdurdod tystysgrifau, felly nid yw CI yn eu rhedeg.
- Mae coeden `spec/` yn fawr; ffeil statws wedi'i chynhyrchu yw'r ffordd i weld beth a ddatganwyd o'i gymharu â'r hyn sy'n pasio.

## Sut i gyfrannu

Darllenwch [AGENTS.md](../../../../AGENTS.md) yn gyntaf: pum rheol, yna map yn ôl tasg.

Gweler hefyd dudalen y [Prif Swyddog Gwybodaeth](../chief-information-officer/index.md) am y golwg gweithredol.

## Pa mor gyfredol yw hyn

Ysgrifennwyd ar 2026-10-11 o'r fanyleb, nid o redeg y meddalwedd ar gyfer y dudalen hon. Nid yw'r ffigurau sy'n newid (profion, mudo, tasgau a gwblhawyd) wedi'u copïo yma; darllenwch nhw yn [implementation-status.md](../../../../spec/implementation-status.md), a gynhyrchir gan sgript. Meddalwedd arddangos yw WPM, gyda data synthetig: mae'r dudalen hon yn dweud beth mae'n ei wneud a beth nad yw'n ei wneud, ac nid yw'n honni ei fod yn addas ar gyfer eich sefydliad. Nid yw dim yma yn gyngor cyfreithiol.

> Ysgrifennwyd y cyfieithiad hwn gan AI ac nid yw siaradwr brodorol wedi'i adolygu. Os yw'n wahanol i'r fersiwn Saesneg (`locales/en-gb-oxendict/for/`), y fersiwn Saesneg sy'n gywir.
