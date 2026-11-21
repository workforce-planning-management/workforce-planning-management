# 최고운영책임자 향

**관점:** 적합한 사람이 적합한 곳에 적합한 때에 있는가, 누군가 자리를 비우면 무엇이 무너지는가?

## 얻을 수 있는 것

| 알고 싶은 것 | WPM이 답하는 곳 | 읽을 문서 |
| --- | --- | --- |
| 팀이 결원을 메울 수 있는가 | 사람별 대체자, 교대와 알림이 있는 대기 근무표, 가용한 사람이 없으면 분명히 알려 주는 대체 보기 | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| 근무 시간, 출근, 휴가 | 근무 시간 기록, 승인 경로가 있는 휴가 요청, 발생액, 교대 근무 | [workforce-management.md](../../../../spec/workforce-management.md) |
| 근로 시간 한도가 지켜지고 있는가 | 권고용 근로 시간 가드레일. 경고할 뿐 막지는 않음 | WPM-R27, 같은 파일 |
| 월별 기술별 수용 능력 | 기술 풀, 공급과 프로그램 수요, 파트너 약정, 제약 풀, 제안일 뿐인 시작 점검과 기록된 시작 결정 | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| 무엇이 필요하며 격차는 어디인가 | 인력 계획, 부서별 직무별 예측과 격차 분석 | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| 곧 끝나는 계약은 무엇인가 | 계약 근거별 종료일, 이력이 있는 연장, 종료 전 알림, 종료일이 없는 계약 목록, 전환 계획 | [engagements.md](../../../../spec/engagements.md) |
| 비정규 인력의 비중이 큰 곳은 어디인가 | 계약직과 기간제 각각의 FTE 비중, 그리고 월별로 끝나는 재원 | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| 입사자와 퇴사자 | 날짜가 있는 체크리스트와 마지막 근무일 인수인계 | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| 역량 격차와 교육 시간 | 권고의 근거가 되는 증거를 갖춘 격차 분석 | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## 수용 능력 보기를 읽는 방법

수용 능력은 사람이 아니라 기술 풀 단위로 계획됩니다. 풀의 한 달은 네 가지 결과 중 하나가 되며, 선언되지 않은 기술은 부족이 아니라 *알 수 없음*입니다. 시작 점검은 제안이며, 시작 결정은 사람이 기록합니다. 크리티컬 체인과 크리티컬 패스 일정은 여기에 없으며 project-portfolio-management의 영역입니다.

## 한계

- 비정규 비중에는 아직 추세가 없습니다. 근거별로 나눈 인원수 스냅샷이 필요합니다(WPM-T139, 구축되지 않음).
- 비정규 보기의 풀은 계획이 풀을 정의하기 전까지 부서입니다.
- 교대 근무 일정과 급여세 계산은 귀사의 근무표나 급여 시스템을 대신하지 않습니다.
- 여러 보기가 조직의 기록을 메모리로 읽어 들이며, 부하 수치는 측정된 적이 없습니다([readiness-assessment.md](../../../../spec/readiness-assessment.md), 격차 8).

> 이 번역은 AI가 작성했으며 원어민의 검토를 받지 않았습니다. 영어판(`locales/en-001/for/`)과 다르면 영어판이 우선합니다.
