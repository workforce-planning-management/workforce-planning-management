# Hoạch định nhân lực

**Một nền tảng nhân sự mã nguồn mở bao quát toàn bộ vòng đời nhân viên, từ tuyển dụng đến nghỉ hưu.**

Hoạch định nhân lực (Workforce Planning Management, WPM) kết hợp tuyển dụng, thời gian và nghỉ phép, phát triển nhân viên, hoạch định nhân lực và tính lương trong một ứng dụng duy nhất. Đây là phần mềm trình diễn, dùng để đánh giá và làm thiết kế tham chiếu.

Ngôn ngữ: العربية, বাংলা, Cymraeg, English, Español, हिन्दी, 日本語, 한국어, Русский, اردو, 中文

> **Phần mềm trình diễn, chưa sẵn sàng cho môi trường sản xuất.** WPM chỉ dùng dữ liệu tổng hợp, và các phép tính theo luật định chỉ mang tính minh họa. Đừng nhập dữ liệu cá nhân thật. [Lộ trình đến môi trường sản xuất](path-to-production/index.md) nêu những gì việc sử dụng trong sản xuất đòi hỏi và mỗi phần đã sẵn sàng đến đâu. Xem thêm [quan điểm về quy định pháp lý](../../spec/regulatory.md).

[Chạy bản trình diễn](start/index.md) [Tóm tắt theo vai trò](for/index.md) [Mã nguồn trên GitHub](https://github.com/workforce-planning-management/workforce-planning-management)

## Lợi ích

Dữ liệu nhân sự thường nằm rải rác ở nhiều công cụ riêng biệt, như hộp thư của ứng viên, bảng tính nghỉ phép và bảng phân ca. Mỗi lần chuyển dữ liệu giữa chúng lại thêm công việc thủ công và nguy cơ sai sót. WPM được thiết kế để giảm những lần chuyển đó.

- **Một hệ thống duy nhất.** Hồ sơ tiếp nhận nhân viên, nghỉ phép, đào tạo và lương được lưu trong một ứng dụng, nhờ đó giảm việc nhập lại dữ liệu.
- **Tối thiểu hóa dữ liệu.** Dữ liệu không nên lưu thì không có trường để lưu, và dữ liệu không nên tiết lộ thì không có tuyến API. Chẳng hạn, không có trường chẩn đoán bệnh, và các phản hồi khảo sát nhanh không có tác giả.
- **Quy tắc báo cáo.** Giá trị chưa biết không bị báo cáo thành số không, số liệu tổng hợp không nhận diện được cá nhân, và các ước tính nêu rõ cơ sở của chúng.
- **Bản địa hóa.** Ứng dụng có 25 ngôn ngữ và vùng, còn phần tóm tắt theo vai trò có 21, kể cả các ngôn ngữ viết từ phải sang trái. Quy tắc riêng của từng quốc gia được lưu dưới dạng dữ liệu. Các bản dịch do AI tạo ra và chưa được người bản ngữ rà soát.
- **Truy vết được.** Mã nguồn mở, và mỗi hành vi đều được truy về một yêu cầu, một quyết định thiết kế và một nhiệm vụ có đánh số.
- **Triển khai độc lập.** Ngăn xếp gồm API JSON viết bằng Rust, ứng dụng khách SvelteKit và PostgreSQL. Nó dựng được từ một bản sao mới, và việc tích hợp với các ứng dụng anh em là tùy chọn.

## Tính năng

- **Tuyển dụng và hồ sơ.** Yêu cầu tuyển dụng, quy trình ứng viên, tiếp nhận nhân viên mới, hồ sơ người lao động và sơ đồ tổ chức.
- **Thời gian và xếp lịch.** Thời gian và nghỉ phép, ca làm việc có kiểm tra thời giờ làm việc, lịch trực có đổi ca và nhắc nhở, cùng danh bạ nhân viên.
- **Phát triển.** Đánh giá hiệu suất, phản hồi 360°, học tập, phân tích khoảng cách kỹ năng, khuyến nghị thời gian đào tạo và đào tạo bắt buộc.
- **Hoạch định và báo cáo.** Hoạch định nhân lực chiến lược, chỉ số và nhận định, cùng bảng điều khiển cho tổng giám đốc.
- **Nhân viên mới và nhân viên nghỉ việc.** Danh sách kiểm tra theo ngày và bàn giao ngày cuối kèm nhật ký kiểm toán.
- **Truyền thông.** Thông báo theo đối tượng nhận, có liên kết và số lượt đọc.
- **Quyền của nhân viên và sức khỏe tinh thần.** Gợi ý về sức khỏe tinh thần, khảo sát nhanh ẩn danh, điều chỉnh hợp lý, truy cập dữ liệu cá nhân, xóa dữ liệu và lưu giữ.
- **Lương.** Tính lương và đãi ngộ, kèm các phép tính theo luật định mang tính minh họa.

## So sánh

Bảng dưới đây so sánh WPM với ba loại hệ thống nhân sự. Đây là định hướng chung, không phải phép đo chuẩn. Xem thêm [so sánh theo sản phẩm](../../docs/compare/index.md).

| | WPM | Bộ ứng dụng trọn gói (Workday, SAP SuccessFactors, BambooHR) | Nhân sự mã nguồn mở (frappe/hrms, OrangeHRM, Horilla, Odoo HR) | Công cụ chuyên biệt (Greenhouse, Lever, Gusto, ADP) |
| --- | --- | --- | --- | --- |
| **Phạm vi** | Toàn bộ vòng đời, từ tuyển dụng đến nghỉ hưu | Phạm vi tương tự trên một nền tảng | Các chức năng cốt lõi giống nhau | Một chức năng |
| **Giấy phép** | Mã nguồn mở | Thương mại | Mã nguồn mở | Thương mại |
| **Thế mạnh chính** | Tối thiểu hóa dữ liệu, thiết kế truy vết được, phân tích khoảng cách kỹ năng và thời gian đào tạo | Một nền tảng tích hợp, có hỗ trợ | Nhiều tích hợp, bản địa hóa và triển khai hơn | Chiều sâu ở một chức năng |
| **Phù hợp điển hình** | Đánh giá, thiết kế tham chiếu hoặc phát triển thêm | Tổ chức cần một hệ thống có hỗ trợ | Tổ chức muốn dùng mã nguồn mở trong sản xuất | Tổ chức cần chiều sâu ở một chức năng và tự tích hợp phần còn lại |

## Đối tượng dự kiến

- **Lãnh đạo.** [Các bản tóm tắt theo vai trò](for/index.md) nêu những gì WPM cung cấp cho từng vai trò lãnh đạo và cách đọc kết quả của nó.
- **Đội ngũ nhân sự.** [Hướng dẫn trình diễn](start/index.md) đi qua từng trang bằng dữ liệu hư cấu.
- **Nhà phát triển và kiến trúc sư.** Xem [đặc tả](../../spec/index.md) và [hướng dẫn cài đặt](../../INSTALL.md).
- **Người thẩm định.** [Đánh giá mức độ sẵn sàng](../../spec/readiness-assessment.md) liệt kê những khoảng cách giữa WPM và việc sử dụng trong sản xuất.

## Nhận trợ giúp

- **Báo lỗi hoặc đặt câu hỏi** trong [trình theo dõi vấn đề](https://github.com/workforce-planning-management/workforce-planning-management/issues). Nêu trang nào, bạn đã làm gì, bạn mong đợi điều gì và điều gì đã xảy ra. Đừng đưa dữ liệu cá nhân thật vào.
- **Báo vấn đề bảo mật** qua email, như mô tả trong [chính sách bảo mật](../../SECURITY.md), không báo trong một vấn đề công khai.
- **Học một tác vụ trong ứng dụng.** Liên kết **Trợ giúp** ở đầu trang ứng dụng mở trang trợ giúp và các hướng dẫn từng bước dành cho nhân viên, quản lý và bộ phận nhân sự. Mỗi thông báo lỗi cũng có liên kết đến đó.

## Các bước tiếp theo

1. **Đọc bản tóm tắt cho vai trò của bạn.** Mỗi bản mô tả những gì WPM cung cấp và những gì còn bỏ ngỏ.
2. **Chạy bản trình diễn.** Chuyến tham quan chạy trên máy cục bộ với dữ liệu hư cấu.
3. **Gửi phản hồi.** Xem [cách góp ý](../../RFC.md) và [cách đóng góp](../../CONTRIBUTING.md).

[Chạy bản trình diễn](start/index.md) [Tài liệu](../../index.md)

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/`), bản tiếng Anh được ưu tiên.
