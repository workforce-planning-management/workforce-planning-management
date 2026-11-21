# Kiểm thử bảo mật độc lập

**Góc nhìn:** làm sao để những người không viết phần mềm này tấn công nó, nên chỉ cho họ
những gì, và làm gì với những gì họ tìm thấy?

Chưa có cuộc kiểm thử độc lập nào. [Mô hình mối đe dọa](../../../../spec/governance/threat-model.md) do chính
trợ lý đã viết phần mềm viết ra và không ai khác rà soát, và mọi biện pháp kiểm soát nó nêu đều do
tác giả đó thử nghiệm. [Đánh giá mức độ sẵn sàng](../../../../spec/readiness-assessment.md) liệt kê một cuộc kiểm thử xâm nhập độc lập
là một cửa ải chỉ con người mới đóng được (WPM-T201, **chưa làm**). Trang này là một bản tóm tắt cho người đặt hàng kiểm thử. Nó không phải
kế hoạch kiểm thử của một chuyên gia kiểm thử và không thay thế kế hoạch đó.

## Những gì cần đưa cho người kiểm thử

| Mục | Ở đâu | Ghi chú |
| --- | --- | --- |
| Một bản triển khai để tấn công | Của bạn: [hướng dẫn triển khai](../../../../spec/operations/deployment.md) | Thiết lập sản xuất với đăng nhập **bật**, một nhà cung cấp danh tính thử nghiệm và **chỉ dữ liệu tổng hợp**. Đừng kiểm thử trên hồ sơ của người thật |
| Mã nguồn | Kho mã này | Kiểm thử có mã nguồn kỹ hơn kiểm thử hộp đen; hãy nói rõ bạn muốn loại nào |
| Mô hình mối đe dọa | [governance/threat-model.md](../../../../spec/governance/threat-model.md) | Hai mươi bốn mối đe dọa, biện pháp kiểm soát như đã xây dựng và các khoảng trống. Hãy đưa cho họ **sau** lượt hộp đen đầu tiên nếu bạn muốn xem họ tự tìm ra gì |
| Mô tả API | `GET /api-docs/openapi.json` | Mọi tuyến, kèm `Idempotency-Key` trên mọi thao tác có thay đổi dữ liệu |
| Thiết kế đăng nhập | [auth.md](../../../../spec/auth.md), [operations/entra-sign-in.md](../../../../spec/operations/entra-sign-in.md) | Các nhân vật, thuộc tính và những gì chính sách chỉ quyết định theo hành động |
| Danh tính thử nghiệm | Bạn tự tạo | Tối thiểu: một người lao động không có thuộc tính, một quản lý trực tiếp, một quản lý ngoài chuỗi báo cáo, nhân sự, bộ phận lương, một quản trị viên và một dịch vụ ngang hàng |

## Những gì cần đề nghị họ xem xét

Tác giả không thể kiểm tra những điều này. Mỗi dòng nêu mục riêng của mô hình mối đe dọa, để một phát hiện có thể được truy vết.

| Lĩnh vực | Những gì nên thử | Mối đe dọa |
| --- | --- | --- |
| Token | Nhầm lẫn thuật toán, một bộ khóa xoay vòng trong khi các yêu cầu đang xử lý, một token đã hết hạn hoặc chưa có hiệu lực, một token dành cho đối tượng khác | T1, T2 |
| Chỉ cần biết | Mọi tuyến đọc với tư cách người gọi không có thuộc tính và với tư cách người ở cạnh; một tuyến mà bảng phân loại không biết; một thao tác ghi mà chính sách cho phép nhưng bộ xử lý lẽ ra phải từ chối | T3, T4, T6 |
| Thuộc tính đặc quyền | Một thuộc tính trong token có thể bị thêm, phát lại hoặc kế thừa không; ai có thể tạo ra một thuộc tính; một quản trị viên dùng chung hoặc tồn tại lâu | T24 |
| Chèn mã | Fuzz mọi tham số, đoạn đường dẫn và trường nội dung; các câu lệnh xóa và lưu giữ được dựng từ định danh có kiểu và đã được đọc, chưa được fuzz | T12 |
| Ứng dụng khách trình duyệt | Xử lý phiên bởi máy chủ giao diện, cách nó đổi một phiên lấy token, tấn công chèn tập lệnh chéo trang qua văn bản đã lưu, vượt qua chính sách bảo mật nội dung, trang của trình quản lý nội dung nạp một tập lệnh từ mạng công khai mà không ghim phiên bản | T13, T14 |
| Yêu cầu đi ra | Giả mạo yêu cầu phía máy chủ qua bất kỳ URL nào đã cấu hình (bộ khóa, các dịch vụ anh em, bộ nhớ đệm dùng chung) | T15 |
| Giới hạn | Giới hạn kích thước nội dung, thời gian chờ của yêu cầu, giới hạn tốc độ theo từng phiên bản và các màn hình nạp toàn bộ hồ sơ của một tổ chức vào bộ nhớ | T16, T17 |
| Nhật ký | Có dữ liệu cá nhân, token hay chuỗi truy vấn nào lọt vào nhật ký không | T19 |
| Khóa idempotency | Một khóa được hai người gọi dùng; cùng khóa nhưng đổi nội dung, đường dẫn hoặc truy vấn; một khóa gần như hợp lệ; một nội dung rất lớn; hai yêu cầu chạy đua dưới một khóa (nhận định này được lập luận, chưa được kiểm chứng bằng bài kiểm thử đua); một phản hồi đã lưu có thể bị ai ngoài người gọi đọc không | mới |
| Nhật ký kiểm toán | Mọi cách thay đổi, xóa hoặc bỏ qua một mục; điểm cuối xác minh chuỗi | T7 |
| Sao lưu | Khôi phục từ một bản sao lưu bị can thiệp hoặc sai khóa; người đã bị xóa có quay lại không | T10, T11 |
| Chuỗi cung ứng | Ghim các action dựng, quét bí mật, ký ảnh | T20, T21 |

## Các quy tắc cần thống nhất trước khi họ bắt đầu

Hãy điền những mục này cùng đội bảo mật và đội pháp lý của bạn. Ở đây không quyết định điều gì.

- **Thẩm quyền:** ai xác nhận cuộc kiểm thử được ủy quyền, và cho những hệ thống và địa chỉ nào.
- **Khung thời gian:** ngày và giờ, tính theo ngày dương lịch, và gọi ai để dừng lại.
- **Phạm vi:** những gì nằm trong (API, giao diện, luồng đăng nhập, tập lệnh sao lưu) và những gì nằm ngoài (dịch vụ riêng của
  nhà cung cấp danh tính, mạng của bạn, mọi thứ dùng chung với hệ thống khác).
- **Dữ liệu:** chỉ dữ liệu tổng hợp. Nếu người kiểm thử tìm thấy dữ liệu cá nhân thật: ai được thông báo và xử lý ra sao.
- **Kiểm thử mang tính phá hủy:** có cho phép từ chối dịch vụ, ghi hàng loạt và xóa dữ liệu không, và ở đâu.
- **Báo cáo:** định dạng, thang mức độ nghiêm trọng, ai nhận, họ được giữ bao lâu và các bí mật
  họ tìm thấy được xử lý ra sao.
- **Kiểm thử lại:** hợp đồng bao gồm việc kiểm tra các bản sửa.

## Làm gì với các phát hiện

1. Ghi từng phát hiện thành một nhiệm vụ trong [tasks.md](../../../../spec/tasks.md) kèm mã tham chiếu của người kiểm thử, bằng chứng và
   mức độ nghiêm trọng họ đưa ra. Một phát hiện bị tranh cãi vẫn được ghi là bị tranh cãi, kèm lý do.
2. Sửa bằng một bài kiểm thử thất bại trước. Nếu bản sửa là một quy tắc, hãy thêm nó vào cột khoảng trống của mô hình mối đe dọa và chỉ đóng nó
   ở đó khi lần kiểm thử lại đạt.
3. Đừng đánh dấu WPM-T201 là xong cho đến khi có báo cáo trong tay, mọi phát hiện đều có người phụ trách và một lần kiểm thử lại đã hoàn tất. Cập nhật
   [đánh giá mức độ sẵn sàng](../../../../spec/readiness-assessment.md) và danh sách các cửa ải (WPM-T203) trong cùng thay đổi.
4. Đừng đưa báo cáo vào kho mã nếu nó chứa điều gì có thể bị khai thác; hãy liên kết đến nơi nó được lưu giữ.

## Một báo cáo sạch có nghĩa là gì và không có nghĩa là gì

Một cuộc kiểm thử không có phát hiện cho thấy rằng những người kiểm thử này, trong khung thời gian này, với phạm vi này và những
danh tính này, đã không tìm thấy gì.
Nó không cho thấy không có sai sót, nó không bao gồm một cấu hình bạn đã thay đổi sau đó,
và nó không thay thế các cửa ải khác: một cuộc kiểm toán khả năng tiếp cận, đánh giá tác động bảo vệ
dữ liệu của bạn được phê duyệt, và một buổi diễn tập khôi phục trên nền tảng của bạn.

## Trang này không làm gì

Nó không nêu tên nhà cung cấp, không định giá và không hứa một tiêu chuẩn nào. Chương trình hay chứng nhận nào mà ngành của bạn mong đợi ở một
người kiểm thử là việc bạn quyết định.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/for/`), bản tiếng Anh được ưu tiên.
