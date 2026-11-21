# Dành cho Giám đốc nhân sự

**Góc nhìn:** nó có hỗ trợ toàn bộ vòng đời nhân viên không, và tôi có thể dùng nó mà không
phải lưu về con người nhiều hơn mức cần thiết không?

## Bạn nhận được gì

| Giai đoạn | WPM lưu gì | Đọc |
| --- | --- | --- |
| Tuyển dụng | Yêu cầu tuyển dụng, quy trình ứng viên, kho ứng viên, danh sách kiểm tra tiếp nhận | [talent-acquisition.md](../../../../spec/talent-acquisition.md) |
| Hồ sơ | Một hồ sơ việc làm cho mỗi người lao động, sơ đồ tổ chức, tuyến báo cáo, nhóm, điều chuyển giữa các tổ chức | [hr-core.md](../../../../spec/hr-core.md) |
| Tự phục vụ | Người lao động xem và cập nhật thông tin của mình, yêu cầu làm việc linh hoạt, báo trước khi nghỉ việc và xem thời gian nghỉ của mình | [self-service-and-worker-requests.md](../../../../spec/self-service-and-worker-requests.md) |
| Đời sống làm việc | Nghỉ phép, thời gian, ca làm việc, phúc lợi, gợi ý về sức khỏe tinh thần, điều chỉnh hợp lý, đánh giá công thái học, khảo sát nhanh ẩn danh | [hr-core.md](../../../../spec/hr-core.md), [workforce-management.md](../../../../spec/workforce-management.md) |
| Phát triển | Đánh giá hiệu suất, đánh giá 360, khóa học qua danh mục khóa học, bài đánh giá, kèm cặp, kế hoạch phát triển, kế nhiệm, nhân sự mới vào nghề | [talent-development.md](../../../../spec/talent-development.md) |
| Lương | Kỳ tính lương, phiếu lương, đối chuẩn lương, tra cứu thang lương quốc gia, ngạch chức danh | [payroll-compensation.md](../../../../spec/payroll-compensation.md), [pay-scales.md](../../../../spec/pay-scales.md) |
| Nhân sự không chính thức | Ngày kết thúc theo cơ sở hợp tác, gia hạn, đánh giá tình trạng, kế hoạch chuyển đổi do một người quyết định, nguồn kinh phí đặt cạnh ngày kết thúc hợp đồng | [engagements.md](../../../../spec/engagements.md) |
| Nghỉ việc | Đơn nghỉ việc kèm thời gian báo trước, bàn giao ngày cuối, lưu giữ theo từng loại hồ sơ, xóa dữ liệu | [joiners-and-leavers.md](../../../../spec/joiners-and-leavers.md) |

## Nó bảo vệ con người như thế nào

- **Chỉ cần biết khi đọc.** Một quản lý thấy các báo cáo trực tiếp của mình, một người lao động thấy chính mình, bộ phận nhân sự thấy những gì nhân sự cần.
  Một người lạ đã đăng nhập không thấy gì về đồng nghiệp ([auth.md](../../../../spec/auth.md)).
- **Không lưu chẩn đoán.** Nghỉ ốm ghi loại nghỉ, không ghi lý do.
- **Các tính năng thuộc loại dữ liệu đặc biệt tắt theo mặc định.** Theo dõi bình đẳng và yêu cầu sức khỏe nơi làm việc cần một
  cơ sở pháp lý được ghi lại, mang tính tự nguyện hoặc do vai trò sức khỏe nghề nghiệp nắm giữ, và chỉ hiện các nhóm trên một ngưỡng.
  Chúng không bao giờ là đầu vào cho bất kỳ điểm số hay quyết định nào về một người.
- **Điểm số chỉ tư vấn.** Mức sẵn sàng kế nhiệm, khoảng cách kỹ năng và dự báo chỉ gợi ý; một người quyết định, và những quyết định như
  chuyển một nhà thầu thành nhân viên được ghi lại là quyết định của một người, không bao giờ là kết quả tự động.
- **Chủ thể có thể xem và xóa.** Xuất dữ liệu truy cập của chủ thể và xóa dữ liệu bao quát các hồ sơ nêu trên, kèm một sổ cái để
  một lần khôi phục không đưa người đã bị xóa quay lại.

## Giới hạn

- Chưa xây dựng: giao diện người dùng cho kế hoạch chuyển đổi và nguồn kinh phí, xu hướng nhân sự thời vụ và trang
  "thông tin của tôi" của người lao động. Xem [tasks.md](../../../../spec/tasks.md) để biết tình trạng hiện tại.
- Các bản dịch có máy hỗ trợ; việc rà soát chuyên nghiệp là một cửa ải còn mở.
- Thỏa ước tập thể và các tổ chức được công nhận chỉ là nghiên cứu và đề xuất ([worker-unions.md](../../../../spec/worker-unions.md)).
- Thuế lương chỉ là phần giữ chỗ.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
