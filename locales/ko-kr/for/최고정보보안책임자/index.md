# 최고정보보안책임자 향

**관점:** 어떻게 보호되는가, 공격이나 오용이 있다면 어떻게 알아차리는가, 아무도 시험하지 않은 것은 무엇인가?

## 얻을 수 있는 것

| 관심사 | 있는 것 | 읽을 문서 |
| --- | --- | --- |
| 위협 | 위협 모델: 자산, 행위자, 신뢰 경계, 구현된 통제, 그리고 **평가하지 않은 것** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| 인증 | 로그인은 기본으로 켜져 있음. 꺼져 있거나 토큰 키 출처가 없으면 운영 환경은 시작을 거부. 기업 디렉터리 로그인(OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| 인가 | 정책은 동작만으로 판단하고, 분류된 표가 모든 읽기 경로의 알 필요성을 판단하며, 분류되지 않은 경로가 있으면 소스 스캔 테스트가 실패 | [auth.md](../../../../spec/auth.md) |
| 감사 | 검증 엔드포인트가 있는 추가 전용 해시 연결 추적. 항목에는 이유 문구도 값도 없음 | [audit.md](../../../../spec/audit.md) |
| 전송과 브라우저 | 모든 API 응답의 보안 헤더, 설정하지 않으면 부팅이 실패하는 명시적 CORS 허용 목록, `/api`의 `no-store` | [production-readiness.md](../../../../spec/production-readiness.md)의 WPM-R73 |
| 악용 | 호출자별 요청 제한. 삭제, 가져오기, 내보내기, 토큰 경로는 더 엄격함 | WPM-R74, 같은 파일 |
| 안전하지 않은 설정 | 다른 호스트로의 평문 데이터베이스 연결이나 평문 Redis, 누락된 CORS 출처가 있으면 부팅을 거부 | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 저장 데이터 | 암호화된 백업. 공유 캐시는 개인과 무관한 조회만 보관하며 실패 시 열림(fail open) | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 공급망 | `cargo deny`로 라이선스와 금지 사항 점검, 버전을 고정한 공유 크레이트, 릴리스 시 소프트웨어 자재 명세서 | [production-readiness.md](../../../../spec/production-readiness.md) |
| 사고 | 첫 대응을 모은 운영 절차서. 연락처와 통지 의무는 직접 추가 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 자체 평가 | 통제 체크리스트: 충족, 일부 충족, 미충족, 배포자가 답변 | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## 기록된 알려진 격차

- 독립적인 침투 테스트는 이루어지지 않았습니다. 테스트는 작성자가 썼습니다.
- 액션은 다이제스트가 아니라 태그로 고정되어 있으며 이미지는 서명되지 않았습니다.
- 요청 제한은 인스턴스별입니다.
- 애플리케이션은 주어진 어떤 데이터베이스 역할이든 사용하며, 최소 권한 역할 세트는 제공되지 않습니다.
- 토큰 폐기와 토큰 최대 수명은 사용하는 ID 공급자의 몫입니다.
- 콘텐츠 관리자 경로는 버전 고정이나 무결성 해시 없이 공개 CDN에서 스크립트를 불러옵니다.
- 로그에 개인 데이터가 있는지는 점검되지 않았으며, 감사 추적은 모니터링 시스템으로 내보내지지 않습니다.
- 관리형 Redis(클러스터, 센티넬, 접근 제어 사용자)는 시험해 보지 않았습니다.

## 사람만 할 수 있는 일

독립적인 침투 테스트 의뢰, 귀사 환경에 맞는 위협 모델 검토, 귀사 플랫폼에서의 복원 훈련 실행, 호스팅 플랫폼의 브랜치 보호 설정. [준비 상태 평가](../../../../spec/readiness-assessment.md)를 보십시오. 높은 보증이 필요한 배포에 대한 결론은 **준비되지 않음**입니다.

> 이 번역은 AI가 작성했으며 원어민의 검토를 받지 않았습니다. 영어판(`locales/en-001/for/`)과 다르면 영어판이 우선합니다.
