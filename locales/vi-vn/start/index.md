# Hướng dẫn trình diễn

Hướng dẫn này mô tả cách chạy WPM trên máy cục bộ, nạp dữ liệu hư cấu và đi theo chuyến tham quan có hướng dẫn qua từng trang.

> **Chỉ dùng dữ liệu hư cấu.** Đăng nhập được tắt trong chuyến tham quan. Đừng kết nối nó với dịch vụ chứa dữ liệu thật.

## Điều kiện tiên quyết

- [Rust](https://www.rust-lang.org/tools/install), bản ổn định
- [Node.js](https://nodejs.org/) và [pnpm](https://pnpm.io/)
- [PostgreSQL](https://www.postgresql.org/), với một cơ sở dữ liệu trống cho chuyến tham quan
- Python 3

[Hướng dẫn cài đặt](../../../INSTALL.md) nêu các phiên bản cần thiết và thiết lập cơ sở dữ liệu.

## Bước 1: Lấy mã nguồn

```sh
git clone https://github.com/workforce-planning-management/workforce-planning-management.git
cd workforce-planning-management
```

## Bước 2: Khởi động dịch vụ

Dịch vụ đọc biến `DATABASE_URL`. Giá trị mặc định là `postgres://postgres:postgres@localhost:5432/workforce_planning_management_development`; hãy đặt biến này nếu cơ sở dữ liệu của bạn có tên, người dùng hoặc cổng khác. Chuẩn bị cơ sở dữ liệu trống và khởi động API. Đăng nhập chỉ được tắt cho phiên bản này, và giới hạn tốc độ được nâng lên cho chuyến tham quan.

```sh
cd workforce-planning-management-service-with-rust
cargo run -- db migrate
cargo run -- task seed
WORKFORCE_REQUIRE_AUTH=0 WORKFORCE_RATE_LIMIT_PER_MINUTE=6000 cargo run -- start
```

## Bước 3: Nạp dữ liệu hư cấu

Trong một cửa sổ dòng lệnh thứ hai, từ thư mục gốc của kho mã:

```sh
workforce-demo-data http://localhost:1241
```

## Bước 4: Khởi động ứng dụng

```sh
cd workforce-planning-management-ui-with-svelte
pnpm install
WORKFORCE_DEMO_API_URL=http://localhost:1241 pnpm dev
```

## Bước 5: Đi theo chuyến tham quan

Mở địa chỉ mà ứng dụng hiển thị và đăng nhập bằng **example@example.com**. Không có email nào được gửi. Đăng nhập mở `/demo`, một chuyến tham quan qua mọi trang. Một biểu ngữ trên mỗi trang nêu rõ dữ liệu là hư cấu và có các liên kết Trước và Sau.

## Đọc thêm

- [Tóm tắt theo vai trò](../for/index.md)
- [So sánh theo sản phẩm](../../../docs/compare/index.md)
- [Đặc tả](../../../spec/index.md)
- [Cách góp ý](../../../RFC.md)

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/start/`), bản tiếng Anh được ưu tiên.
