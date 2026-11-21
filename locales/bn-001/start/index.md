# ডেমো নির্দেশিকা

এই নির্দেশিকা দেখায় কীভাবে নিজের কম্পিউটারে WPM চালাতে হয়, কাল্পনিক ডেটা লোড করতে হয় এবং প্রতিটি পৃষ্ঠা ঘুরে দেখার নির্দেশিত পথ অনুসরণ করতে হয়।

> **কেবল কাল্পনিক ডেটা।** নির্দেশিকা চলার সময় সাইন-ইন বন্ধ থাকে। অ্যাপটিকে প্রকৃত ডেটাসহ কোনো সেবার সঙ্গে যুক্ত করবেন না।

## প্রয়োজনীয় জিনিস

- [Rust](https://www.rust-lang.org/tools/install), স্থিতিশীল সংস্করণ
- [Node.js](https://nodejs.org/) এবং [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), নির্দেশিকার জন্য একটি খালি ডেটাবেসসহ
- Python 3

[ইনস্টলেশন নির্দেশিকা](../../../INSTALL.md) প্রয়োজনীয় সংস্করণ ও ডেটাবেস সেটআপ জানায়।

## ধাপ ১। কোড নিন

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## ধাপ ২। সেবা চালু করুন

সেবাটি `DATABASE_URL` পড়ে। ডিফল্ট মান `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; আপনার ডেটাবেসের নাম, ব্যবহারকারী বা পোর্ট আলাদা হলে চলকটি নির্ধারণ করুন। একটি খালি ডেটাবেস প্রস্তুত করে API চালু করুন। সাইন-ইন কেবল এই ইনস্ট্যান্সের জন্য বন্ধ হয়, এবং নির্দেশিকার সময়ের জন্য অনুরোধ-সীমা বাড়ানো হয়।

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## ধাপ ৩। কাল্পনিক ডেটা লোড করুন

দ্বিতীয় একটি টার্মিনালে, রিপোজিটরির মূল ফোল্ডার থেকে:

```sh
workforce-demo-data http://localhost:1241
```

## ধাপ ৪। অ্যাপ চালু করুন

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## ধাপ ৫। পথ ধরে এগোন

অ্যাপ যে ঠিকানা দেখায় তা খুলুন এবং **example@example.com** হিসেবে সাইন ইন করুন। কোনো ইমেইল পাঠানো হয় না। সাইন-ইনের পর `/demo` খোলে, যা সব পৃষ্ঠার মধ্য দিয়ে একটি নির্দেশিত পথ। প্রতিটি পৃষ্ঠার ব্যানার মনে করিয়ে দেয় যে ডেটা কাল্পনিক, এবং "পূর্ববর্তী" ও "পরবর্তী" লিংক দেয়।

## এরপর কী পড়বেন

- [ভূমিকাভিত্তিক নথি](../for/index.md)
- [পণ্যভিত্তিক তুলনা](../../../docs/compare/index.md)
- [বিনির্দেশ](../../../spec/index.md)
- [কীভাবে মতামত দেবেন](../../../RFC.md)
