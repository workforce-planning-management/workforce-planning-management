# Dành cho Giám đốc pháp chế

**Góc nhìn:** phần mềm này tạo ra hoặc giúp thực hiện những nghĩa vụ pháp lý nào, và chúng ta
phải quyết định điều gì trước khi nhập dữ liệu thật?

> Không có gì ở đây là tư vấn pháp lý. Đặc tả nêu một quan điểm, không phải kết luận về bất kỳ khu vực pháp lý nào.

## Bạn nhận được gì

| Chủ đề | Hiện có gì | Đọc |
| --- | --- | --- |
| Quan điểm | Quan điểm về UK GDPR và hồ sơ lao động; tình trạng bản trình diễn; sản xuất sẽ cần thêm những gì | [regulatory.md](../../../../spec/regulatory.md) |
| Đánh giá tác động | Một mẫu đánh giá tác động bảo vệ dữ liệu, đã hoàn thành cho bản trình diễn, với các phần bạn phải hoàn thành được đánh dấu | [governance/dpia.md](../../../../spec/governance/dpia.md) |
| Hồ sơ xử lý và luồng dữ liệu | Các tài liệu có thể điều chỉnh | [governance/record-of-processing.md](../../../../spec/governance/record-of-processing.md), [governance/data-flow.md](../../../../spec/governance/data-flow.md) |
| Lưu giữ | Một lịch trình cho từng loại hồ sơ, có đợt dọn dẹp cho một số loại | [governance/retention-schedule.md](../../../../spec/governance/retention-schedule.md) |
| Quyền của chủ thể dữ liệu | Xuất dữ liệu truy cập của chủ thể, xóa dữ liệu và sổ cái xóa dữ liệu | [regulatory.md](../../../../spec/regulatory.md) (WPM-R30) |
| Dữ liệu thuộc loại đặc biệt | Theo dõi bình đẳng và yêu cầu sức khỏe nơi làm việc tắt cho đến khi bạn ghi lại cơ sở pháp lý, sau đó bị giới hạn: tự nguyện, chỉ bản thân hoặc giới hạn theo vai trò, số liệu tổng hợp trên một ngưỡng, không bao giờ dùng trong một quyết định | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Quyền theo luật định | Yêu cầu làm việc linh hoạt với thời hạn quyết định và lý do do bạn cấu hình; thời gian báo trước khi nghỉ việc do bạn cấu hình | [operations/deployment.md](../../../../spec/operations/deployment.md) |
| Nghĩa vụ về bình đẳng | Một mẫu đánh giá tác động về bình đẳng và sổ đăng ký mọi điểm số | [equality-impact/template.md](../../../../spec/equality-impact/template.md) |
| Khả năng tiếp cận và ngôn ngữ | Một tuyên bố về khả năng tiếp cận và một tuyên bố về ngôn ngữ chính thức | [governance/accessibility-statement.md](../../../../spec/governance/accessibility-statement.md), [governance/official-language-statement.md](../../../../spec/governance/official-language-statement.md) |
| Các tổ chức tập thể | Nghiên cứu và đề xuất cho các tổ chức được công nhận và tham vấn, không bao giờ cho tư cách thành viên cá nhân | [worker-unions.md](../../../../spec/worker-unions.md) |
| Nhà thầu | Cơ sở hợp tác, nhà cung cấp, tuyến và mức giá được che đều được ghi lại. Một hợp đồng có hợp pháp hay được phân loại đúng hay không không phải điều phần mềm quyết định | [engagements.md](../../../../spec/engagements.md) |
| Giấy phép | Các tệp giấy phép, chính sách về phụ thuộc và bảng kê vật liệu phần mềm khi phát hành | [production-readiness.md](../../../../spec/production-readiness.md) |

## Mặc định không phải là luật

Một số thiết lập là mặc định, không phải tư vấn pháp lý: thời gian báo trước khi nghỉ việc, thời hạn quyết định yêu cầu làm việc linh hoạt và
thời hạn kháng nghị, cùng các lý do từ chối. Hướng dẫn triển khai yêu cầu bạn **xác nhận chúng** cho khu vực pháp lý của mình.
Các phép tính theo luật định như thuế lương chưa được triển khai.

## Các quyết định thuộc về bạn

Cơ sở pháp lý cho từng mục đích xử lý, đánh giá tác động bảo vệ dữ liệu và việc phê duyệt nó, các thỏa thuận
với bên xử lý, lịch trình lưu giữ bạn áp dụng, và có bật bất kỳ tính năng nào về dữ liệu thuộc loại đặc biệt hay không.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
