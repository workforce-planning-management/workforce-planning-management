# Dành cho Giám đốc an ninh thông tin

**Góc nhìn:** nó được bảo vệ ra sao, làm sao chúng ta biết nếu nó bị tấn công hoặc lạm dụng, và
điều gì chưa ai kiểm thử?

## Bạn nhận được gì

| Mối quan tâm | Hiện có gì | Đọc |
| --- | --- | --- |
| Mối đe dọa | Một mô hình mối đe dọa: tài sản, các tác nhân, ranh giới tin cậy, biện pháp kiểm soát như đã xây dựng, và những gì **chưa được đánh giá** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Xác thực | Đăng nhập bật theo mặc định; sản xuất từ chối khởi động nếu nó tắt hoặc không có nguồn khóa token; đăng nhập bằng thư mục doanh nghiệp (OIDC) | [production-readiness.md](../../../../spec/production-readiness.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) |
| Phân quyền | Chính sách chỉ quyết định theo hành động; một bảng phân loại quyết định nguyên tắc chỉ cần biết trên mọi tuyến đọc, và một bài kiểm thử quét mã nguồn sẽ thất bại nếu có tuyến chưa phân loại | [auth.md](../../../../spec/auth.md) |
| Kiểm toán | Một nhật ký chỉ được thêm vào, nối bằng chuỗi băm, có điểm cuối xác minh; các mục không mang văn bản lý do và không mang giá trị | [audit.md](../../../../spec/audit.md) |
| Truyền tải và trình duyệt | Tiêu đề bảo mật trên mọi phản hồi API, một danh sách cho phép CORS tường minh làm khởi động thất bại nếu chưa đặt, `no-store` trên `/api` | WPM-R73 trong [production-readiness.md](../../../../spec/production-readiness.md) |
| Lạm dụng | Giới hạn tốc độ theo người gọi, chặt hơn với các tuyến xóa, nhập, xuất và token | WPM-R74, cùng tệp |
| Cấu hình không an toàn | Khi khởi động, từ chối kết nối cơ sở dữ liệu không mã hóa hoặc Redis không mã hóa đến máy khác, và một nguồn CORS bị thiếu | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Dữ liệu lưu trữ | Sao lưu được mã hóa; bộ nhớ đệm dùng chung chỉ chứa các tra cứu không phải dữ liệu cá nhân và lỗi theo hướng mở | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Chuỗi cung ứng | Kiểm tra giấy phép và danh sách cấm bằng `cargo deny`, các crate dùng chung được ghim phiên bản, bảng kê vật liệu phần mềm khi phát hành | [production-readiness.md](../../../../spec/production-readiness.md) |
| Sự cố | Một sổ tay vận hành gồm các hành động đầu tiên; bạn bổ sung đầu mối liên hệ và nghĩa vụ thông báo | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Tự đánh giá | Một danh sách kiểm tra kiểm soát: đạt, đạt một phần, không đạt, bên triển khai trả lời | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Các khoảng trống đã biết, như đã ghi nhận

- Chưa có cuộc kiểm thử xâm nhập độc lập nào. Các bài kiểm thử do tác giả viết.
- Các action được ghim theo thẻ, không theo mã băm, và ảnh container chưa được ký.
- Giới hạn tốc độ tính theo từng phiên bản.
- Ứng dụng dùng bất kỳ vai trò cơ sở dữ liệu nào được cấp; chưa có bộ vai trò với đặc quyền tối thiểu.
- Việc thu hồi token và tuổi thọ tối đa của token thuộc về nhà cung cấp danh tính của bạn.
- Đường dẫn của trình quản lý nội dung nạp một tập lệnh từ một CDN công khai mà không ghim phiên bản hay mã băm toàn vẹn.
- Nhật ký chưa được kiểm tra xem có dữ liệu cá nhân không, và nhật ký kiểm toán chưa được xuất sang hệ thống giám sát.
- Redis được quản lý (cụm, sentinel, người dùng kiểm soát truy cập) chưa được thử.

## Chỉ con người mới làm được

Đặt hàng cuộc kiểm thử xâm nhập độc lập, rà soát mô hình mối đe dọa cho môi trường của bạn, chạy một buổi diễn tập khôi phục trên
nền tảng của bạn, và đặt bảo vệ nhánh trên nền tảng lưu trữ. Xem
[đánh giá mức độ sẵn sàng](../../../../spec/readiness-assessment.md), có kết luận **chưa sẵn sàng** cho một triển khai đòi hỏi độ bảo đảm cao.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
