# ڈیمو ٹیوٹوریل

یہ ٹیوٹوریل بتاتا ہے کہ WPM کو مقامی مشین پر کیسے چلایا جائے، فرضی ڈیٹا کیسے لوڈ کیا جائے اور ہر صفحے کے رہنما ٹور کی پیروی کیسے کی جائے۔

> **صرف فرضی ڈیٹا۔** ٹور کے لیے سائن اِن بند ہے۔ اسے ایسی سروس سے نہ جوڑیں جس میں حقیقی ڈیٹا ہو۔

## پیشگی ضروریات

- [Rust](https://www.rust-lang.org/tools/install)، مستحکم ورژن
- [Node.js](https://nodejs.org/) اور [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/)، ٹور کے لیے ایک خالی ڈیٹا بیس کے ساتھ
- Python 3

مطلوبہ ورژن اور ڈیٹا بیس کی ترتیبات [انسٹال گائیڈ](../../../INSTALL.md) میں ہیں۔

## مرحلہ 1: کوڈ حاصل کریں

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## مرحلہ 2: سروس شروع کریں

سروس `DATABASE_URL` پڑھتی ہے۔ اس کی ڈیفالٹ قدر `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development` ہے؛ اگر آپ کے ڈیٹا بیس کا نام، صارف یا پورٹ مختلف ہو تو یہ متغیر طے کریں۔ خالی ڈیٹا بیس تیار کریں اور API شروع کریں۔ سائن اِن صرف اسی انسٹینس کے لیے بند ہے، اور ٹور کے لیے شرحِ حد بڑھا دی گئی ہے۔

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## مرحلہ 3: فرضی ڈیٹا لوڈ کریں

دوسرے ٹرمینل میں، ریپوزٹری کی جڑ سے:

```sh
workforce-demo-data http://localhost:1241
```

## مرحلہ 4: ایپلیکیشن شروع کریں

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## مرحلہ 5: ٹور کی پیروی کریں

ایپلیکیشن کا دکھایا گیا پتہ کھولیں اور **example@example.com** کے طور پر سائن اِن کریں۔ کوئی ای میل نہیں بھیجی جاتی۔ سائن اِن `/demo` کھولتا ہے، جو ہر صفحے کا ٹور ہے۔ ہر صفحے پر ایک بینر بتاتا ہے کہ ڈیٹا فرضی ہے اور پچھلا اور اگلا کے لنک دیتا ہے۔

## مزید مطالعہ

- [کرداروں کے مختصر بیانات](../for/index.md)
- [مصنوعات کے لحاظ سے موازنہ](../../../docs/compare/index.md)
- [تفصیلات](../../../spec/index.md)
- [تبصرہ کیسے کریں](../../../RFC.md)

> یہ ترجمہ ایک AI نے لکھا ہے اور کسی مادری زبان بولنے والے نے اس کا جائزہ نہیں لیا۔ انگریزی ورژن (`locales/en-001/start/`) سے فرق ہو تو انگریزی ورژن معتبر ہے۔
