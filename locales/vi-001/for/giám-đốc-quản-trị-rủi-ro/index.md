# Dành cho Giám đốc quản trị rủi ro

**Góc nhìn:** điều gì có thể hỏng ở hệ thống này, điều gì ngăn chặn, và điều gì
chưa ai kiểm tra?

## Điểm xuất phát trung thực

[Đánh giá mức độ sẵn sàng](../../../../spec/readiness-assessment.md) ngày 2026-10-11 đưa ra kết luận **chưa sẵn sàng** cho một
triển khai đòi hỏi độ bảo đảm cao. Nó tìm thấy một trở ngại bằng cách thăm dò dịch vụ đang chạy: bất kỳ người nào đã đăng nhập đều đọc được
đơn nghỉ ốm của đồng nghiệp cùng lý do ghi tự do. Lỗi đó đã được sửa và kiểm thử. Các cửa ải chỉ con người mới đóng được
vẫn còn mở.

## Bạn nhận được gì

| Mối quan tâm | Hiện có gì | Đọc |
| --- | --- | --- |
| Mối đe dọa ở từng ranh giới | Một mô hình mối đe dọa: tài sản, các tác nhân, ranh giới, biện pháp kiểm soát như đã xây dựng, và những gì **chưa được đánh giá** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Đọc trái phép | Một lớp bảo vệ theo nguyên tắc chỉ cần biết, phân loại mọi tuyến đọc; một bài kiểm thử sẽ thất bại nếu có tuyến chưa phân loại | [auth.md](../../../../spec/auth.md) (WPM-R114) |
| Can thiệp và trách nhiệm giải trình | Nhật ký kiểm toán chỉ được thêm vào, nối bằng chuỗi băm, có điểm cuối xác minh; các mục không chứa văn bản lý do hay giá trị | [audit.md](../../../../spec/audit.md) (WPM-R115) |
| Dữ liệu nhạy cảm | Không lưu chẩn đoán; các tính năng dữ liệu đặc biệt tắt theo mặc định; có ngưỡng cho nhóm nhỏ | [regulatory.md](../../../../spec/regulatory.md) |
| Đường đi không an toàn | Sản xuất từ chối khởi động khi đăng nhập tắt, không có nguồn khóa token, kết nối cơ sở dữ liệu không mã hóa đến máy khác, hoặc Redis từ xa không mã hóa | [production-readiness.md](../../../../spec/production-readiness.md) |
| Mất dữ liệu | Sao lưu được mã hóa, một buổi diễn tập khôi phục chạy trong CI, một sổ cái xóa dữ liệu | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Thiên lệch trong chấm điểm | Một mẫu đánh giá tác động về bình đẳng, và sổ đăng ký mọi điểm số kèm một bản đánh giá nháp | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Tự đánh giá kiểm soát | Một danh sách kiểm tra mà mỗi câu trả lời là đạt, đạt một phần, không đạt, hoặc do bên triển khai trả lời | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |

## Các rủi ro còn mở, như đã ghi nhận

Việc ghim và ký chuỗi cung ứng, bằng chứng về tải, kiểm tra nhật ký xem có dữ liệu cá nhân không, giới hạn tốc độ dùng chung giữa các
phiên bản, bộ vai trò cơ sở dữ liệu với đặc quyền tối thiểu, và hướng dẫn thu hồi token là các mục còn mở trong đánh giá mức
độ sẵn sàng, mỗi mục có một nhiệm vụ. Các bài kiểm thử do tác giả viết; không bài nào là rà soát độc lập.

## Chỉ con người mới làm được

Một cuộc kiểm thử xâm nhập độc lập, một cuộc kiểm toán khả năng tiếp cận độc lập, đánh giá tác động của bên triển khai được phê duyệt,
một buổi diễn tập khôi phục trên nền tảng của bạn, và việc phê duyệt các đánh giá tác động về bình đẳng. Kho mã nêu những điều này
là các cửa ải và không đánh dấu chúng là đã xong.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
