# Dành cho Giám đốc thông tin

**Góc nhìn:** nó khớp với hạ tầng của chúng ta ra sao, ai đăng nhập được, dữ liệu nằm ở đâu,
và chúng ta vận hành cũng như khôi phục nó thế nào?

## Bạn nhận được gì

| Mối quan tâm | Hiện có gì | Đọc |
| --- | --- | --- |
| Cấu trúc | Một API JSON và một ứng dụng khách trình duyệt riêng; PostgreSQL lưu dữ liệu | [architecture.md](../../../../spec/architecture.md) |
| Danh tính | WPM không lưu danh tính. Một người, một người lao động và một tổ chức là các tham chiếu đến dịch vụ khác, chỉ được phân giải theo tên và được lưu đệm | [integrations.md](../../../../spec/integrations.md), [scope.md](../../../../spec/scope.md) |
| Đăng nhập | Bật theo mặc định. Đăng nhập bằng thư mục doanh nghiệp (OIDC, tương thích Entra, trực tiếp hoặc qua Keycloak); vai trò và thuộc tính quyết định quyền truy cập | [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md), [auth.md](../../../../spec/auth.md) |
| Triển khai | Một bản dựng container và một tệp compose cho bản trình diễn; với sản xuất thì cần một proxy kết thúc TLS, các thiết lập bắt buộc và các bước kiểm tra khi khởi động từ chối cấu hình không an toàn | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Bộ nhớ đệm dùng chung | Redis tùy chọn, chỉ cho các tra cứu không chứa dữ liệu cá nhân; mọi lệnh gọi đều lỗi theo hướng mở | `REDIS_URL` trong hướng dẫn triển khai (WPM-D78) |
| Sao lưu và khôi phục | Sao lưu được mã hóa, một tập lệnh khôi phục và một buổi diễn tập khôi phục do CI chạy | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Nâng cấp | Các bản di chuyển dữ liệu và hướng dẫn nâng cấp | [operations/upgrade-and-migration.md](../../../../spec/operations/upgrade-and-migration.md) |
| Khi sự cố xảy ra | Sổ tay vận hành gồm các hành động đầu tiên | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Luồng dữ liệu | Một tài liệu về luồng dữ liệu | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |

## Điều cần biết trước khi bạn lập kế hoạch

- Giao diện người dùng **không được đóng gói container**; bạn chọn một bộ điều hợp cho nền tảng của mình.
- Giới hạn tốc độ tính theo từng phiên bản. Kho lưu trữ dùng chung do bạn cung cấp.
- Ứng dụng dùng bất kỳ vai trò cơ sở dữ liệu nào mà `DATABASE_URL` nêu tên; chưa có bộ vai trò với đặc quyền tối thiểu.
- Việc thu hồi token và tuổi thọ tối đa của token thuộc về nhà cung cấp danh tính của bạn.
- Redis được quản lý (cụm, sentinel, người dùng kiểm soát truy cập) chưa được thử.
- Chưa có phép đo tải nào, và một số màn hình đọc cả một tổ chức vào bộ nhớ.
- Bản dựng ảnh và tệp compose đã được viết nhưng nên thử trước trong môi trường của bạn.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
