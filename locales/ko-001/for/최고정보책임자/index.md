# 최고정보책임자 향

**관점:** 이것은 우리 IT 환경에 어떻게 맞는가, 누가 로그인할 수 있는가, 데이터는 어디에 있는가, 어떻게 운영하고 복구하는가?

## 얻을 수 있는 것

| 관심사 | 있는 것 | 읽을 문서 |
| --- | --- | --- |
| 구조 | JSON API와 별도의 브라우저 클라이언트. 데이터는 PostgreSQL에 보관 | [architecture.md](../../../../spec/architecture.md) |
| 신원 | WPM은 신원을 보관하지 않습니다. 사람, 근로자, 조직은 다른 서비스에 대한 참조이며 이름으로만 해석되고 캐시됩니다 | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| 로그인 | 기본으로 켜져 있음. 기업 디렉터리 로그인(OIDC, Entra 호환, 직접 또는 Keycloak 경유). 역할과 속성이 접근을 결정 | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| 배포 | 데모용 컨테이너 빌드와 compose 파일. 운영 환경에서는 TLS를 종단하는 프록시, 필수 설정, 안전하지 않은 구성을 거부하는 부팅 점검 | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 공유 캐시 | 개인과 무관한 조회에만 쓰는 선택적 Redis. 모든 호출은 실패 시 열림(fail open) | 배포 안내의 `REDIS_URL`(WPM-D78) |
| 백업과 복구 | 암호화된 백업, 복원 스크립트, CI가 실행하는 복원 훈련 | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| 업그레이드 | 마이그레이션과 업그레이드 안내 | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| 문제가 생겼을 때 | 첫 대응을 모은 운영 절차서 | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| 데이터 이동 | 데이터 흐름 문서 | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## 계획하기 전에 알아 둘 것

- UI는 **컨테이너화되어 있지 않습니다.** 사용하는 플랫폼에 맞는 어댑터를 직접 고르십시오.
- 요청 제한은 인스턴스별입니다. 공유 저장소는 직접 제공해야 합니다.
- 애플리케이션은 `DATABASE_URL`이 지정하는 어떤 데이터베이스 역할이든 사용합니다. 최소 권한 역할 세트는 아직 제공되지 않습니다.
- 토큰 폐기와 토큰 최대 수명은 사용하는 ID 공급자의 몫입니다.
- 관리형 Redis(클러스터, 센티넬, 접근 제어 사용자)는 시험해 보지 않았습니다.
- 부하 측정은 없으며, 일부 보기는 조직 전체를 메모리로 읽어 들입니다.
- 이미지 빌드와 compose 파일은 작성되었지만 먼저 귀사 환경에서 시험해 보아야 합니다.

> 이 번역은 AI가 작성했으며 원어민의 검토를 받지 않았습니다. 영어판(`locales/en-001/for/`)과 다르면 영어판이 우선합니다.
