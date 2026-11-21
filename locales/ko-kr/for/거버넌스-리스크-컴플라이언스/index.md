# 거버넌스, 리스크, 컴플라이언스

**관점:** 누가 무엇에 책임을 지는가, 무엇이 잘못될 수 있고 무엇이 막는가, 평가자에게 어떤 증거를 보여 줄 수 있는가? 이 페이지는 거버넌스, 리스크, 컴플라이언스 자료를 한곳에 모았습니다. 최고위험관리책임자와 최고법무책임자 역할 페이지는 같은 내용을 역할별로 다룹니다.

## 거버넌스: 누가 결정하고 어떻게 기록되는가

| 주제 | 있는 것 | 읽을 문서 |
| --- | --- | --- |
| 작업의 통제 방식 | 명세가 먼저: 요구 사항, 결정, 작업, 코드, 테스트가 함께 들어감. 각 작업에는 **검증하지 않은** 것이 적혀 있음 | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| 프로젝트 거버넌스 | 거버넌스, 메인테이너, RFC 절차, 코드 소유자 파일 | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| 사람에 대한 결정 | 사람이 결정하고 점수는 조언함. 전환, 사직, 유연 근무 요청은 사람의 결정으로 기록됨 | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 접근과 역할 | 역할과 속성이 접근을 결정하고, 분류된 표가 모든 읽기 경로의 알 필요성을 결정 | [auth.md](../../../../spec/auth.md) |
| AI의 도움을 받은 저작 | AI를 어떻게 사용했는지에 대한 설명 | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## 리스크: 무엇이 잘못될 수 있는가

| 주제 | 있는 것 | 읽을 문서 |
| --- | --- | --- |
| 결론 | 높은 보증이 필요한 배포에 **준비되지 않음**. 탐색으로 한 가지 차단 요인(읽기가 알 필요성에 기반하지 않았음)을 찾아 수정함 | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| 위협 | 위협 모델: 자산, 행위자, 경계, 구현된 통제, 그리고 **평가하지 않은 것** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| 변조 증거 | 검증 엔드포인트가 있는 추가 전용 해시 연결 감사 추적. 항목에는 이유 문구나 값이 없음 | [audit.md](../../../../spec/audit.md) |
| 안전한 시작 | 로그인이 꺼져 있거나, 토큰 키가 없거나, 다른 호스트로의 평문 데이터베이스 연결이거나, 평문 원격 Redis이면 운영 환경은 시작을 거부 | [production-readiness.md](../../../../spec/production-readiness.md) |
| 복구 | 암호화된 백업, CI가 실행하는 복원 훈련, 복원해도 삭제된 사람이 돌아오지 않게 하는 삭제 원장 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 사고 | 첫 대응을 모은 운영 절차서. 연락처와 에스컬레이션 경로는 배포자가 추가 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 점수의 편향 | 평등 영향 템플릿과 초안 평가가 붙은 모든 점수의 등록부 | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| 미해결 위험 | 공급망 고정과 서명, 부하 증거, 개인 데이터에 대한 로그 점검, 공유 요청 제한, 최소 권한 데이터베이스 역할 세트, 토큰 폐기 지침 | [readiness-assessment.md](../../../../spec/readiness-assessment.md)의 3절 |

## 컴플라이언스: 평가자에게 보여 줄 수 있는 것

| 주제 | 있는 것 | 읽을 문서 |
| --- | --- | --- |
| 통제 자체 평가 | 각 답변이 충족, 일부 충족, 미충족, 또는 배포자가 답변인 체크리스트 | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| 개인정보 영향 평가 | 데모용으로 작성된 템플릿. 직접 작성해야 하는 부분은 표시되어 있음 | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| 처리 활동 기록 | 조정해서 쓸 수 있는 기록 | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| 데이터 흐름 | 개인 데이터가 이동하는 곳 | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| 보존 | 기록 종류별 일정 | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| 정보 주체의 권리 | 정보 주체 접근 내보내기와 삭제 | [regulatory.md](../../../../spec/regulatory.md) |
| 특별 범주 데이터 | 평등 모니터링과 직장 건강 요건은 적법 근거가 기록되기 전까지 꺼져 있고, 기록된 뒤에도 제한됨 | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 접근성과 언어 | 접근성 성명과 공용어 성명 | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| 라이선스와 의존성 | 라이선스 파일, `cargo deny`로 점검하는 의존성 정책, 릴리스 시 소프트웨어 자재 명세서 | [production-readiness.md](../../../../spec/production-readiness.md) |

## 사람만 할 수 있는 일

이것들은 코드가 아니라 관문이며, 저장소는 완료로 표시하지 않습니다. 독립적인 침투 테스트, 독립적인 접근성 감사, 담당자가 승인한 개인정보 영향 평가, 귀사 플랫폼에서의 복원 훈련, 번역의 전문가 검토, 평등 영향 평가의 승인입니다.

## 이 페이지의 한계

표는 문서를 가리킬 뿐입니다. 각 문서가 귀사의 배포에 대해 사실인지는 귀사와 평가자가 판단할 일입니다. 작성자의 테스트는 독립적인 검토가 아니며, 여기에 있는 어떤 것도 법률 자문이나 인증이 아닙니다.

> 이 번역은 AI가 작성했으며 원어민의 검토를 받지 않았습니다. 영어판(`locales/en-001/for/`)과 다르면 영어판이 우선합니다.
