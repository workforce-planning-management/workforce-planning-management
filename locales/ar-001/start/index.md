# الدليل التعليمي للعرض التجريبي

يشرح هذا الدليل كيفية تشغيل WPM على جهاز محلي وتحميل بيانات خيالية واتباع الجولة الإرشادية في كل صفحة.

> **بيانات خيالية فقط.** تسجيل الدخول متوقف أثناء الجولة. لا توصلها بخدمة تحتفظ ببيانات حقيقية.

## المتطلبات المسبقة

- [Rust](https://www.rust-lang.org/tools/install)، الإصدار المستقر
- [Node.js](https://nodejs.org/) و[pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/)، مع قاعدة بيانات فارغة للجولة
- Python 3

يذكر [دليل التثبيت](../../../INSTALL.md) (بالإنجليزية) الإصدارات المطلوبة وإعدادات قاعدة البيانات.

## الخطوة 1: احصل على الشفرة

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## الخطوة 2: شغّل الخدمة

تقرأ الخدمة `DATABASE_URL`. وقيمته الافتراضية `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`؛ اضبط هذا المتغير إذا كان لقاعدة بياناتك اسم أو مستخدم أو منفذ آخر. جهّز قاعدة البيانات الفارغة وشغّل واجهة البرمجة. يتوقف تسجيل الدخول لهذه النسخة فقط، ويُرفع حد المعدل من أجل الجولة.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## الخطوة 3: حمّل البيانات الخيالية

في طرفية ثانية، من جذر المستودع:

```sh
workforce-demo-data http://localhost:1241
```

## الخطوة 4: شغّل التطبيق

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## الخطوة 5: اتبع الجولة

افتح العنوان الذي يعرضه التطبيق وسجّل الدخول باسم **example@example.com**. لا يُرسل أي بريد إلكتروني. يفتح تسجيل الدخول `/demo`، وهي جولة في كل الصفحات. ويذكر شريط في كل صفحة أن البيانات خيالية ويقدم روابط السابق والتالي.

## لمزيد من القراءة

- [وثائق الأدوار](../for/index.md)
- [المقارنة حسب المنتج](../../../docs/compare/index.md) (بالإنجليزية)
- [المواصفات](../../../spec/index.md)
- [كيفية التعليق](../../../RFC.md)
