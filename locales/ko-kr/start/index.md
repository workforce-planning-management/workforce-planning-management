# 데모 튜토리얼

이 튜토리얼은 WPM을 로컬 컴퓨터에서 실행하고, 가상의 데이터를 불러와 각 페이지의 안내 투어를 따라가는 방법을 설명합니다.

> **가상의 데이터만 사용합니다.** 투어에서는 로그인이 꺼져 있습니다. 실제 데이터를 보관하는 서비스에는 연결하지 마십시오.

## 사전 준비

- [Rust](https://www.rust-lang.org/tools/install), 안정 버전
- [Node.js](https://nodejs.org/)와 [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), 투어용 빈 데이터베이스 포함
- Python 3

필요한 버전과 데이터베이스 설정은 [설치 안내](../../../INSTALL.md)에 있습니다.

## 1단계: 코드 받기

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## 2단계: 서비스 시작

서비스는 `DATABASE_URL`을 읽습니다. 기본값은 `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`이며, 데이터베이스의 이름, 사용자 또는 포트가 다르면 이 변수를 설정하십시오. 빈 데이터베이스를 준비하고 API를 시작합니다. 로그인은 이 인스턴스에 한해서만 꺼지며, 투어를 위해 요청 제한을 높입니다.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## 3단계: 가상의 데이터 불러오기

두 번째 터미널에서 저장소 루트에서 실행합니다.

```sh
workforce-demo-data http://localhost:1241
```

## 4단계: 애플리케이션 시작

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## 5단계: 투어 따라가기

애플리케이션이 표시하는 주소를 열고 **example@example.com**으로 로그인합니다. 이메일은 발송되지 않습니다. 로그인하면 모든 페이지를 둘러보는 투어인 `/demo`가 열립니다. 각 페이지의 배너가 데이터가 가상임을 알려 주며 이전과 다음 링크를 제공합니다.

## 더 읽을거리

- [역할별 브리프](../for/index.md)
- [제품별 비교](../../../docs/compare/index.md)
- [명세](../../../spec/index.md)
- [의견을 내는 방법](../../../RFC.md)

> 이 번역은 AI가 작성했으며 원어민의 검토를 받지 않았습니다. 영어판(`locales/en-001/start/`)과 다르면 영어판이 우선합니다.
