# Quản trị, rủi ro, tuân thủ

**Góc nhìn:** ai chịu trách nhiệm về điều gì, điều gì có thể hỏng và điều gì ngăn chặn, và
tôi có thể đưa ra bằng chứng nào cho người thẩm định? Trang này tập hợp tài liệu về quản trị, rủi ro và tuân thủ ở một nơi.
Các trang vai trò của Giám đốc quản trị rủi ro và Giám đốc pháp chế bao quát
cùng những mảng này theo từng vai trò.

## Quản trị: ai quyết định, và việc đó được ghi lại ra sao

| Chủ đề | Hiện có gì | Đọc |
| --- | --- | --- |
| Công việc được quản trị thế nào | Đặc tả trước: yêu cầu, quyết định, nhiệm vụ, mã và kiểm thử cùng được bàn giao; mỗi nhiệm vụ nêu điều gì **chưa** được xác minh | [spec-driven-delivery](../../../../AGENTS/spec-driven-delivery.md) |
| Quản trị dự án | Quản trị, người bảo trì, quy trình RFC và một tệp chủ sở hữu mã | [GOVERNANCE.md](../../../../GOVERNANCE.md), [RFC.md](../../../../RFC.md), [MAINTAINERS.md](../../../../MAINTAINERS.md) |
| Quyết định về con người | Một người quyết định; điểm số chỉ tư vấn. Chuyển đổi, đơn nghỉ việc và yêu cầu làm việc linh hoạt được ghi lại là quyết định của một người | [engagements.md](../../../../spec/engagements.md), [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Truy cập và vai trò | Vai trò và thuộc tính quyết định quyền truy cập; một bảng phân loại quyết định nguyên tắc chỉ cần biết trên mọi tuyến đọc | [auth.md](../../../../spec/auth.md) |
| Quyền tác giả có AI hỗ trợ | Một tuyên bố về cách AI đã được sử dụng | [AI_STATEMENT.md](../../../../AI_STATEMENT.md) |

## Rủi ro: điều gì có thể hỏng

| Chủ đề | Hiện có gì | Đọc |
| --- | --- | --- |
| Kết luận | **Chưa sẵn sàng** cho một triển khai đòi hỏi độ bảo đảm cao; một trở ngại (các lượt đọc không theo nguyên tắc chỉ cần biết) được tìm ra bằng cách thăm dò và đã sửa | [readiness-assessment.md](../../../../spec/readiness-assessment.md) |
| Mối đe dọa | Một mô hình mối đe dọa: tài sản, các tác nhân, ranh giới, biện pháp kiểm soát như đã xây dựng, và những gì **chưa được đánh giá** | [governance/threat-model.md](../../../../spec/governance/threat-model.md) |
| Dấu hiệu bị can thiệp | Một nhật ký kiểm toán chỉ được thêm vào, nối bằng chuỗi băm, có điểm cuối xác minh; các mục không chứa văn bản lý do hay giá trị | [audit.md](../../../../spec/audit.md) |
| Khởi động an toàn | Sản xuất từ chối khởi động khi đăng nhập tắt, không có khóa token, kết nối cơ sở dữ liệu không mã hóa đến máy khác, hoặc Redis từ xa không mã hóa | [production-readiness.md](../../../../spec/production-readiness.md) |
| Khôi phục | Sao lưu được mã hóa, một buổi diễn tập khôi phục do CI chạy, một sổ cái xóa dữ liệu để một lần khôi phục không đưa người đã bị xóa quay lại | [operations/backup-and-restore.md](../../../../spec/operations/backup-and-restore.md) |
| Sự cố | Một sổ tay vận hành gồm các hành động đầu tiên; bên triển khai bổ sung đầu mối liên hệ và đường chuyển lên cấp trên | [operations/runbook.md](../../../../spec/operations/runbook.md) |
| Thiên lệch trong chấm điểm | Một mẫu đánh giá tác động về bình đẳng và sổ đăng ký mọi điểm số, mỗi điểm kèm một bản đánh giá nháp | [equality-impact/register.md](../../../../spec/equality-impact/register.md) |
| Rủi ro còn mở | Ghim và ký chuỗi cung ứng, bằng chứng về tải, kiểm tra nhật ký xem có dữ liệu cá nhân không, giới hạn tốc độ dùng chung, bộ vai trò cơ sở dữ liệu với đặc quyền tối thiểu, hướng dẫn thu hồi token | [readiness-assessment.md](../../../../spec/readiness-assessment.md), mục 3 |

## Tuân thủ: bạn có thể đưa gì cho người thẩm định

| Chủ đề | Hiện có gì | Đọc |
| --- | --- | --- |
| Tự đánh giá kiểm soát | Một danh sách kiểm tra mà mỗi câu trả lời là đạt, đạt một phần, không đạt, hoặc do bên triển khai trả lời | [governance/assessment-checklist.md](../../../../spec/governance/assessment-checklist.md) |
| Đánh giá tác động bảo vệ dữ liệu | Một mẫu, đã hoàn thành cho bản trình diễn, với các phần bạn phải hoàn thành được đánh dấu | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Hồ sơ xử lý | Một hồ sơ có thể điều chỉnh | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md) |
| Luồng dữ liệu | Nơi dữ liệu cá nhân di chuyển | [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Lưu giữ | Một lịch trình cho từng loại hồ sơ | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Quyền của chủ thể dữ liệu | Xuất dữ liệu truy cập của chủ thể và xóa dữ liệu | [regulatory.md](../../../../spec/regulatory.md) |
| Dữ liệu thuộc loại đặc biệt | Theo dõi bình đẳng và yêu cầu sức khỏe nơi làm việc tắt cho đến khi một cơ sở pháp lý được ghi lại, sau đó bị giới hạn | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Khả năng tiếp cận và ngôn ngữ | Một tuyên bố về khả năng tiếp cận và một tuyên bố về ngôn ngữ chính thức | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Giấy phép và phụ thuộc | Các tệp giấy phép, chính sách về phụ thuộc được `cargo deny` kiểm tra và bảng kê vật liệu phần mềm khi phát hành | [production-readiness.md](../../../../spec/production-readiness.md) |

## Chỉ con người mới làm được

Đây là các cửa ải, không phải mã, và kho mã không đánh dấu chúng là đã xong: một cuộc kiểm thử xâm nhập độc lập, một
cuộc kiểm toán khả năng tiếp cận độc lập, đánh giá tác động bảo vệ dữ liệu của bạn được cán bộ của bạn phê duyệt, một buổi diễn tập khôi phục
trên nền tảng của bạn, một cuộc rà soát chuyên nghiệp các bản dịch, và việc bạn phê duyệt các đánh giá tác động về bình đẳng.

## Giới hạn của trang này

Các bảng chỉ trỏ tới tài liệu. Việc mỗi tài liệu có đúng với triển khai của bạn hay không là việc của bạn và người thẩm định.
Các bài kiểm thử của tác giả không phải là rà soát độc lập, và không có gì ở đây là tư vấn pháp lý hay chứng nhận.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
