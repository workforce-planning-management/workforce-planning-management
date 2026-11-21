# Dành cho Giám đốc vận hành

**Góc nhìn:** chúng ta có đúng người, đúng chỗ, đúng lúc không, và
điều gì sẽ đổ vỡ nếu có người vắng mặt?

## Bạn nhận được gì

| Bạn muốn biết | WPM trả lời ở đâu | Đọc |
| --- | --- | --- |
| Một nhóm có thể bao quát được vắng mặt không | Người dự phòng cho từng người, lịch trực có đổi ca và nhắc nhở, và một màn hình bao quát nói thẳng khi không có ai sẵn sàng | [people-directory-and-cover.md](../../../../spec/people-directory-and-cover.md) |
| Thời gian, chuyên cần và nghỉ phép | Bản ghi thời gian, đơn xin nghỉ có đường phê duyệt, tích lũy và ca làm việc | [workforce-management.md](../../../../spec/workforce-management.md) |
| Giới hạn thời giờ làm việc có được tôn trọng không | Các hàng rào tư vấn về thời giờ làm việc; chúng cảnh báo chứ không chặn | WPM-R27, cùng tệp |
| Năng lực theo kỹ năng, từng tháng | Nhóm kỹ năng, nguồn cung và nhu cầu của chương trình, cam kết của đối tác, nhóm là nút thắt, và một bước kiểm tra trước khi bắt đầu chỉ là gợi ý, kèm các quyết định bắt đầu được ghi lại | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Chúng ta sẽ cần gì và khoảng trống ở đâu | Kế hoạch nhân lực, dự báo và phân tích khoảng trống theo phòng ban và theo vai trò | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) |
| Những hợp đồng nào sắp kết thúc | Ngày kết thúc theo cơ sở hợp tác, gia hạn có lịch sử, nhắc nhở trước khi kết thúc, danh sách hợp đồng thiếu ngày kết thúc và kế hoạch chuyển đổi | [engagements.md](../../../../spec/engagements.md) |
| Nơi nhân sự thời vụ chiếm tỷ trọng lớn | Tỷ lệ FTE là nhà thầu và là hợp đồng có thời hạn, tách riêng từng loại, và nguồn kinh phí kết thúc theo từng tháng | `GET /api/workforce-intelligence/contingent` (WPM-T109) |
| Nhân viên mới và nhân viên nghỉ việc | Danh sách kiểm tra theo ngày và bàn giao ngày cuối | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |
| Khoảng cách kỹ năng và thời gian đào tạo | Phân tích khoảng cách kèm bằng chứng mà một khuyến nghị dựa vào | [skills-and-training.md](../../../../spec/skills-and-training.md) |

## Cách đọc màn hình năng lực

Năng lực được hoạch định theo nhóm kỹ năng, không theo từng người. Một tháng của một nhóm cho ra một trong bốn kết quả, và một
kỹ năng chưa khai báo là *chưa biết*, không phải thiếu hụt. Bước kiểm tra trước khi bắt đầu là một gợi ý, và một người ghi lại quyết định bắt đầu.
Lập lịch theo chuỗi găng và đường găng không có ở đây: chúng thuộc về project-portfolio-management.

## Giới hạn

- Tỷ trọng nhân sự thời vụ chưa có xu hướng: nó cần ảnh chụp số nhân sự chia theo cơ sở hợp tác (WPM-T139, chưa xây dựng).
- Nhóm cho màn hình nhân sự thời vụ là phòng ban cho đến khi các kế hoạch xác định nhóm.
- Lập lịch ca và tính thuế lương không thay thế hệ thống phân ca hay hệ thống lương của bạn.
- Một số màn hình nạp hồ sơ của cả một tổ chức vào bộ nhớ; chưa đo được con số tải nào
  ([readiness-assessment.md](../../../../spec/readiness-assessment.md), khoảng trống 8).

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
