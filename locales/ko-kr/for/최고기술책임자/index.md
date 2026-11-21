# 최고기술책임자 향

**관점:** 잘 만들어졌는가, 우리 팀이 안전하게 바꿀 수 있는가, 어떻게 알 수 있는가?

## 얻을 수 있는 것

| 관심사 | 있는 것 | 읽을 문서 |
| --- | --- | --- |
| 스택 | API는 Rust + Axum + Axa + PostgreSQL 18, 클라이언트는 Svelte 5와 TypeScript의 SvelteKit 2 | [architecture.md](../../../../spec/architecture.md) |
| 설계 규칙 | 순수한 핵심: 규칙은 데이터베이스와 시계에 의존하지 않고, 철저히 테스트한 뒤 컨트롤러에 연결 | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| 명세가 먼저 | 변경은 요구 사항, 결정, 작업, 코드, 테스트가 함께 들어감. 각 작업 항목에는 **검증하지 않은** 것이 적혀 있음 | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| 테스트 | 규칙에 대한 단위 테스트, 데이터베이스를 쓰는 요청 테스트, 로그인을 켜고 실행하는 강제 바이너리, 프런트엔드 단위 테스트와 브라우저 테스트. 변이 점검: 일부러 코드를 깨뜨려 테스트가 실패하는지 확인 | [testing.md](../../../../spec/testing.md) |
| 개수 | 파일에서 생성되며 손으로 복사하지 않음 | [implementation-status.md](../../../../spec/implementation-status.md) |
| 공급망 | `cargo deny`로 라이선스와 금지 사항 점검, 버전을 고정한 공유 크레이트, 릴리스 시 소프트웨어 자재 명세서 | [production-readiness.md](../../../../spec/production-readiness.md) |
| API 계약 | OpenAPI와, 등록된 경로에 문서가 없으면 실패하는 테스트 | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| 접근 제어 | 정책은 동작만으로 판단하고, 핸들러와 분류된 경로 표가 알 필요성을 판단하며, 분류되지 않은 읽기 경로가 있으면 소스 스캔이 실패 | [auth.md](../../../../spec/auth.md) |
| 성능 | [BENCHMARKS.md](../../../../BENCHMARKS.md)에 측정한 내용이 기록됨. 분산 부하 테스트 프레임워크를 만들었음 | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## 기여하는 방법

먼저 [AGENTS.md](../../../../AGENTS.md)를 읽으십시오. 다섯 가지 규칙이 있고, 이어서 작업별 안내가 있습니다.

> 이 번역은 AI가 작성했으며 원어민의 검토를 받지 않았습니다. 영어판(`locales/en-001/for/`)과 다르면 영어판이 우선합니다.
