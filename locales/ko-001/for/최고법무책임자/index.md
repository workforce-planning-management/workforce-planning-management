# 최고법무책임자 향

**관점:** 이 소프트웨어는 어떤 법적 의무를 만들거나 돕는가, 실제 데이터를 넣기 전에 무엇을 결정해야 하는가?

> 여기에 있는 어떤 것도 법률 자문이 아닙니다. 명세는 입장을 밝힐 뿐 특정 관할권에 대한 결론이 아닙니다.

## 얻을 수 있는 것

| 주제 | 있는 것 | 읽을 문서 |
| --- | --- | --- |
| 입장 | 영국 GDPR과 고용 기록에 관한 입장, 데모 상태, 운영 환경에서 추가로 필요한 것 | [regulatory.md](../../../../spec/regulatory.md) |
| 영향 평가 | 데모용으로 작성된 개인정보 영향 평가 템플릿. 직접 작성해야 하는 부분은 표시되어 있음 | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| 처리 활동 기록과 데이터 흐름 | 조정해서 쓸 수 있는 문서 | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md), [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| 보존 | 기록 종류별 일정과 일부에 대한 정리 작업 | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| 정보 주체의 권리 | 정보 주체 접근 내보내기, 삭제, 삭제 원장 | [regulatory.md](../../../../spec/regulatory.md) (WPM-R30) |
| 특별 범주 데이터 | 평등 모니터링과 직장 건강 요건은 적법 근거를 기록하기 전까지 꺼져 있고, 기록한 뒤에도 제한됨: 자발적, 본인 한정 또는 역할 한정, 하한 이상의 집계, 결정에는 절대 사용하지 않음 | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| 법정 권리 | 결정 기한과 이유를 설정할 수 있는 유연 근무 요청, 설정할 수 있는 사직 통지 | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| 평등 의무 | 평등 영향 템플릿과 모든 점수의 등록부 | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| 접근성과 언어 | 접근성 성명과 공용어 성명 | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| 집단적 기구 | 인정된 기구와 협의에 대한 조사와 제안. 개인의 가입 여부는 다루지 않음 | [worker-unions.md](../../../../spec/worker-unions.md) |
| 계약직 | 계약의 근거, 공급자, 경로, 마스킹된 단가가 기록됩니다. 계약이 적법한지, 올바르게 분류되었는지는 소프트웨어가 판단하지 않습니다 | [engagements.md](../../../../spec/engagements.md) |
| 라이선스 | 라이선스 파일, 의존성 정책, 릴리스 시 소프트웨어 자재 명세서 | [production-readiness.md](../../../../spec/production-readiness.md) |

## 법이 아닌 기본값

여러 설정은 법률 자문이 아니라 기본값입니다. 사직 통지, 유연 근무 결정 기한과 이의 신청 기한, 거부 사유가 그렇습니다. 배포 안내는 관할권에 맞게 **확인하라**고 합니다. 급여세 같은 법정 계산은 구현되어 있지 않습니다.

## 직접 결정할 일

처리 목적별 적법 근거, 개인정보 영향 평가와 그 승인, 처리자 계약, 채택할 보존 일정, 특별 범주 기능을 켤 것인지.

> 이 번역은 AI가 작성했으며 원어민의 검토를 받지 않았습니다. 영어판(`locales/en-001/for/`)과 다르면 영어판이 우선합니다.
