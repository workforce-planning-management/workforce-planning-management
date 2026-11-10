# डेमो ट्यूटोरियल

यह ट्यूटोरियल बताता है कि WPM को स्थानीय मशीन पर कैसे चलाएँ, काल्पनिक डेटा कैसे लोड करें और हर पृष्ठ के निर्देशित टूर का पालन कैसे करें।

> **केवल काल्पनिक डेटा।** टूर के दौरान साइन-इन बंद रहता है। इसे ऐसी सेवा से न जोड़ें जिसमें वास्तविक डेटा हो।

## पूर्वापेक्षाएँ

- [Rust](https://www.rust-lang.org/tools/install), स्थिर संस्करण
- [Node.js](https://nodejs.org/) और [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), टूर के लिए एक खाली डेटाबेस के साथ
- Python 3

[इंस्टॉल गाइड](../../../INSTALL.md) (अंग्रेज़ी में) आवश्यक संस्करण और डेटाबेस सेटिंग बताती है।

## चरण 1: कोड प्राप्त करें

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## चरण 2: सेवा शुरू करें

सेवा `DATABASE_URL` पढ़ती है। इसका डिफ़ॉल्ट मान `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development` है; यदि आपके डेटाबेस का नाम, उपयोगकर्ता या पोर्ट अलग है तो यह चर सेट करें। खाली डेटाबेस तैयार करें और API शुरू करें। साइन-इन केवल इस इंस्टेंस के लिए बंद रहता है, और टूर के लिए दर सीमा बढ़ाई जाती है।

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WPM_REQUIRE_AUTH=0 WPM_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## चरण 3: काल्पनिक डेटा लोड करें

दूसरे टर्मिनल में, रिपॉज़िटरी के मूल से:

```sh
python3 scripts/demo-data.py http://localhost:5150
```

## चरण 4: एप्लिकेशन शुरू करें

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WPM_DEMO_API_URL=http://localhost:5150 pnpm dev
```

## चरण 5: टूर का पालन करें

एप्लिकेशन द्वारा दिखाया गया पता खोलें और **example@example.com** के रूप में साइन इन करें। कोई ईमेल नहीं भेजा जाता। साइन-इन `/demo` खोलता है, जो हर पृष्ठ का टूर है। हर पृष्ठ पर एक बैनर बताता है कि डेटा काल्पनिक है और पिछला तथा अगला लिंक देता है।

## आगे पढ़ें

- [भूमिका दस्तावेज़](../for/index.md)
- [उत्पाद के अनुसार तुलना](../../../docs/compare/index.md) (अंग्रेज़ी में)
- [विनिर्देश](../../../spec/index.md)
- [टिप्पणी कैसे करें](../../../RFC.md)
