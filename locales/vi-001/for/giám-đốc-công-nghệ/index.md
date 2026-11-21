# Dành cho Giám đốc công nghệ

**Góc nhìn:** nó có được xây dựng tốt không, đội của tôi có thể thay đổi nó một cách an toàn không, và làm sao tôi biết?

## Bạn nhận được gì

| Mối quan tâm | Hiện có gì | Đọc |
| --- | --- | --- |
| Ngăn xếp | Rust + Axum + Axa + PostgreSQL 18 cho API; SvelteKit 2 với Svelte 5 và TypeScript cho ứng dụng khách | [architecture.md](../../../../spec/architecture.md) |
| Quy tắc thiết kế | Một lõi thuần túy: các quy tắc không phụ thuộc cơ sở dữ liệu và đồng hồ, được kiểm thử toàn diện, rồi mới nối vào các bộ điều khiển | [AGENTS/backend-rust.md](../../../../AGENTS/backend-rust.md) |
| Đặc tả trước | Một thay đổi gồm yêu cầu, quyết định, nhiệm vụ, mã và kiểm thử cùng nhau; mỗi mục nhiệm vụ nêu điều gì **chưa** được xác minh | [AGENTS/spec-driven-delivery.md](../../../../AGENTS/spec-driven-delivery.md) |
| Kiểm thử | Kiểm thử đơn vị cho các quy tắc, kiểm thử yêu cầu dựa trên cơ sở dữ liệu, các tệp thực thi kiểm tra cưỡng chế chạy với đăng nhập bật, kiểm thử đơn vị giao diện và kiểm thử trên trình duyệt. Kiểm tra đột biến: cố ý làm hỏng mã và xác nhận một bài kiểm thử thất bại | [testing.md](../../../../spec/testing.md) |
| Số liệu | Được tạo từ các tệp, không bao giờ sao chép thủ công | [implementation-status.md](../../../../spec/implementation-status.md) |
| Chuỗi cung ứng | `cargo deny` cho giấy phép và danh sách cấm, các crate dùng chung được ghim phiên bản, bảng kê vật liệu phần mềm khi phát hành | [production-readiness.md](../../../../spec/production-readiness.md) |
| Hợp đồng API | OpenAPI, và một bài kiểm thử thất bại nếu một tuyến đã đăng ký không được ghi tài liệu | [workforce-planning-management-service-with-rust/AGENTS.md](../../../../workforce-planning-management-service-with-rust/AGENTS.md) |
| Kiểm soát truy cập | Chính sách chỉ quyết định theo hành động; các bộ xử lý và một bảng tuyến đã phân loại quyết định nguyên tắc chỉ cần biết, và một lần quét mã nguồn thất bại nếu có tuyến đọc chưa phân loại | [auth.md](../../../../spec/auth.md) |
| Hiệu năng | [BENCHMARKS.md](../../../../BENCHMARKS.md) ghi lại những gì đã đo; khung kiểm thử tải phân tán đã được tạo | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |

## Cách đóng góp

Hãy đọc [AGENTS.md](../../../../AGENTS.md) trước: năm quy tắc, rồi một bản đồ theo từng nhiệm vụ.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
