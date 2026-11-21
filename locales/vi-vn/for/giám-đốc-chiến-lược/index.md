# Dành cho Giám đốc chiến lược

**Góc nhìn:** lực lượng lao động của chúng ta có được định hình để phục vụ chiến lược không, chúng ta sẽ cần gì, và
nên làm gì với khoảng trống?

## Bạn nhận được gì

| Bạn muốn biết | WPM trả lời ở đâu | Đọc |
| --- | --- | --- |
| Một vai trò đòi hỏi gì | Hồ sơ vai trò với các kỹ năng bắt buộc và mức thành thạo tối thiểu, để khoảng cách được đo so với một vai trò chứ không phải một người | [strategic-workforce-planning.md](../../../../spec/strategic-workforce-planning.md) (WPM-R34) |
| Chúng ta sẽ cần gì | Kế hoạch nhân lực dưới dạng thế giới nháp, nhiều phương án đặt cạnh nhau, với các giả định nêu rõ và nhu cầu theo phòng ban, vai trò và kỳ | WPM-R36, cùng tệp |
| Khoảng trống ở đâu | Nguồn cung dự kiến so với nhu cầu đã hoạch định: khoảng trống về số nhân sự và về năng lực, với các đòn bẩy được liệt kê dưới dạng gợi ý kèm bằng chứng | WPM-R37 |
| Nó có phục vụ chiến lược không | Các mục tiêu gắn với nhu cầu: mục tiêu chưa có nguồn lực, nhu cầu không phù hợp, vai trò then chốt không có người dự bị | WPM-R38 |
| Chúng ta có thể thực hiện một chương trình không | Nhóm kỹ năng, nhu cầu của chương trình, cam kết của đối tác, nhóm là nút thắt, một bước kiểm tra trước khi bắt đầu chỉ là gợi ý | [delivery-capacity.md](../../../../spec/delivery-capacity.md) |
| Chúng ta phụ thuộc bao nhiêu vào nhân sự không chính thức | Tỷ lệ FTE là nhà thầu và có thời hạn, tách riêng từng loại, và bao nhiêu kinh phí kết thúc trong mỗi tháng của mười hai tháng tới | `GET /api/workforce-intelligence/contingent` (WPM-T109, một phần) |
| Chúng ta định làm gì khi hợp đồng kết thúc | Kế hoạch chuyển đổi: quyết định của một người về việc chuyển đổi, gia hạn hay kết thúc, với nguồn kinh phí đặt cạnh ngày kết thúc hợp đồng | [engagements.md](../../../../spec/engagements.md) |
| Còn gì khác được lên kế hoạch | Lộ trình và danh sách nhiệm vụ đang mở | [roadmap.md](../../../../spec/roadmap.md), [tasks.md](../../../../spec/tasks.md) |

## Cách đọc một bản dự báo

- Đó là một quy tắc được nêu rõ áp dụng cho các đầu vào được nêu rõ, không phải một mô hình, và mỗi kết quả nêu tên các giả định của nó.
- Với quá ít lịch sử số nhân sự, nó báo `insufficient_history` thay vì ngoại suy.
- Một kế hoạch chỉ chứa số nhân sự giả định. Nó không bao giờ đọc hay thay đổi hồ sơ của một người.
- Các đòn bẩy là gợi ý. Người lập kế hoạch chọn, và phần mềm không quyết định điều gì.

## Giới hạn

- Đòn bẩy "chuyển nhân sự thời vụ thành chính thức" trong các kế hoạch đã được đặc tả (WPM-R63) nhưng chưa có trong mã.
- Tỷ trọng nhân sự thời vụ chưa có xu hướng: nó cần ảnh chụp chia theo cơ sở hợp tác (WPM-T139, chưa xây dựng).
- Lập lịch theo chuỗi găng và đường găng thuộc về project-portfolio-management, không phải ở đây.
- Chưa có giao diện người dùng cho nguồn kinh phí hay kế hoạch chuyển đổi.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
