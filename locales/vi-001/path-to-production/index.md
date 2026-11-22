# Lộ trình đến môi trường sản xuất

WPM là phần mềm trình diễn. Trang này nêu việc sử dụng trong sản xuất sẽ có ý nghĩa gì đối với nó, và liệt kê những gì còn nằm giữa WPM và việc sử dụng đó. Mỗi mục có một ước lượng mức độ sẵn sàng, từ 0 đến 100 phần trăm.

## Môi trường sản xuất nghĩa là gì

Sử dụng trong sản xuất nghĩa là một tổ chức dựa vào WPM cho công việc thật:

- các hồ sơ là của người thật: nhân viên, ứng viên và người liên hệ khẩn cấp của họ;
- tiền lương do WPM tính được chi trả, và luật của nơi làm việc áp dụng cho nó;
- mọi người đăng nhập mỗi ngày, và một số điều họ đọc là nhạy cảm, chẳng hạn nghỉ vì ốm;
- tổ chức phải giải trình với một cơ quan quản lý, một kiểm toán viên và những người có tên trong hồ sơ về cách mình vận hành.

Trang này đo WPM so với trường hợp khó nhất mà nó được đánh giá: một tổ chức y tế hoặc chính phủ lớn ([đánh giá mức độ sẵn sàng](../../../spec/readiness-assessment.md)). Một tổ chức nhỏ hơn có thể cần ít hơn ở một số mục, nhưng vẫn cần cùng các loại bằng chứng.

Môi trường sản xuất cần ba loại bằng chứng, và riêng phần mềm chỉ cung cấp loại đầu tiên:

1. **Phần mềm** làm đúng việc một cách an toàn, với các bài kiểm thử chứng minh điều đó.
2. **Một bên triển khai** lưu trữ và vận hành nó: sao lưu, giám sát, hỗ trợ và ứng phó sự cố.
3. **Những người không phải tác giả** đã kiểm tra nó: một chuyên gia kiểm thử xâm nhập, một chuyên gia kiểm toán khả năng tiếp cận, người dịch và cán bộ bảo vệ dữ liệu của bên triển khai.

## Cách đọc các ước lượng

- Một ước lượng là đánh giá của tác giả vào ngày 2026-11-13, dựa trên đặc tả và các bài kiểm thử. Nó không phải phép đo và không phải chứng nhận.
- 0 phần trăm nghĩa là chưa có gì. 100 phần trăm nghĩa là đã được xây dựng, kiểm thử và một người độc lập kiểm tra trong một triển khai thật. Không mục nào đạt 100 phần trăm.
- Mỗi mục nêu ước lượng của nó dựa trên cơ sở nào và còn lại gì.
- Các ước lượng không được lấy trung bình. Chỉ một mục có ước lượng thấp cũng đủ giữ WPM ngoài môi trường sản xuất.

## Phần mềm

- **75%** **Đăng nhập và truy cập theo nguyên tắc chỉ cần biết.** Đăng nhập bật theo mặc định, và dịch vụ từ chối khởi động trong sản xuất nếu thiếu nó (WPM-R72). Việc đọc giới hạn ở những người có nhu cầu biết (WPM-T188), và một token có tuổi thọ tối đa (WPM-T200). Dựa trên: các bài kiểm thử cưỡng chế chạy trong CI. Còn lại: chưa có danh sách thu hồi token, và chưa chạy cuộc kiểm thử xâm nhập độc lập nào.
- **70%** **Một nhật ký kiểm toán đầy đủ và không bị sửa đổi.** Nhật ký kiểm toán chỉ được thêm vào và nối bằng chuỗi băm, và có thể xuất sang hệ thống giám sát (WPM-T189, WPM-T197). Dựa trên: các bài kiểm thử được cố ý làm hỏng để chứng minh chúng thất bại. Còn lại: phần đuôi bị cắt của chuỗi không bị phát hiện, và chủ sở hữu cơ sở dữ liệu vẫn có thể tắt các trình kích hoạt.
- **70%** **Bảo vệ dữ liệu ngay từ thiết kế.** Con người được xác định bằng tham chiếu, không có cột nhân khẩu học, và có truy cập dữ liệu của chủ thể, xóa dữ liệu và lưu giữ cho từng loại hồ sơ. Dựa trên: các bài kiểm thử và mục 1 của đánh giá mức độ sẵn sàng. Còn lại: đánh giá và cơ sở pháp lý riêng của bên triển khai (bên dưới).
- **75%** **Kết nối và sao lưu được mã hóa.** Trong sản xuất, dịch vụ từ chối kết nối cơ sở dữ liệu không mã hóa và ngắt một yêu cầu bị treo (WPM-T191), và các bản sao lưu được mã hóa (WPM-T192). Dựa trên: các bài kiểm thử của cả hai. Còn lại: việc quản lý khóa và chứng chỉ thuộc về bên triển khai.
- **10%** **Các phép tính theo luật định.** Các phép tính theo luật định chỉ mang tính minh họa, và thuế lương là phần giữ chỗ ([quan điểm về quy định pháp lý](../../../spec/regulatory.md)). Dựa trên: đặc tả, vốn nói rõ điều đó. Còn lại: các quy tắc của từng nơi làm việc, do một người có chuyên môn viết và kiểm tra. Đừng trả lương cho ai từ WPM.
- **50%** **Quy mô.** Trên một máy tính xách tay, các trang danh bạ và màn hình năng lực không hỏng ở mức 50.000 người lao động, và các màn hình hoạch định từ chối nhiều hơn 10.000 (WPM-T196, WPM-T230). Dựa trên: một lần chạy đã đo ([BENCHMARKS.md](../../../BENCHMARKS.md)). Còn lại: màn hình năng lực vẫn tăng tỷ lệ thuận với lực lượng lao động, và chưa đo gì trên phần cứng của bên triển khai.
- **65%** **Chuỗi cung ứng.** Mọi action CI được ghim vào một commit và lịch sử được quét tìm bí mật. Ảnh container được ký và mang theo nguồn gốc của nó, và điều này đã chạy cho bản phát hành v0.1.2 (WPM-T194, WPM-T195). Dựa trên: lần chạy phát hành đó. Còn lại: gói ảnh ở chế độ riêng tư, nên chưa ai ngoài dự án xác minh được.
- **60%** **Giám sát.** Mọi phản hồi mang một mã yêu cầu, và các dòng nhật ký không mang chuỗi truy vấn hay định danh cá nhân (WPM-T197). Dựa trên: các bài kiểm thử của nhật ký yêu cầu. Còn lại: bảng điều khiển và cảnh báo thuộc về bên triển khai, và giới hạn tốc độ chỉ áp dụng cho một phiên bản (WPM-T198).
- **60%** **Sao lưu và khôi phục.** Sao lưu được mã hóa, một buổi diễn tập khôi phục chạy trong CI và một sổ tay vận hành. Dựa trên: buổi diễn tập. Còn lại: một buổi diễn tập khôi phục trên nền tảng riêng của bên triển khai trong 90 ngày dương lịch gần nhất (cửa ải WPM-G4).
- **40%** **Kiểm tra khả năng tiếp cận.** Các kiểm tra tự động theo WCAG 2.2 AA chạy trên ứng dụng và trên trang web này, ở giao diện sáng và tối. Dựa trên: các bài kiểm thử Playwright và axe. Còn lại: cuộc kiểm toán độc lập (bên dưới); chưa dùng trình đọc màn hình nào.
- **20%** **Bản dịch.** Ứng dụng có 25 ngôn ngữ và vùng, còn các bản tóm tắt vai trò có 21. Dựa trên: văn bản đã tồn tại. Còn lại: một AI đã viết các bản dịch, và chưa ai rà soát chúng một cách chuyên nghiệp (WPM-T155).

## Các kiểm tra do người không phải tác giả thực hiện

- **0%** **Kiểm thử xâm nhập độc lập.** Chưa chạy cuộc kiểm thử xâm nhập nào trên một phiên bản đã triển khai (WPM-T201). Dựa trên: nhiệm vụ đang mở. Còn lại: cuộc kiểm thử, và sửa những gì nó tìm ra.
- **0%** **Kiểm toán khả năng tiếp cận độc lập.** Chưa có cuộc kiểm toán theo WCAG 2.2 AA cùng những người dùng công nghệ hỗ trợ (WPM-T202). Dựa trên: nhiệm vụ đang mở. Còn lại: cuộc kiểm toán, và sửa những gì nó tìm ra.
- **25%** **Mô hình mối đe dọa.** Có một mô hình mối đe dọa khởi đầu ([threat-model.md](../../../spec/governance/threat-model.md)), do tác giả viết. Dựa trên: tài liệu. Còn lại: một đội độc lập làm lại nó cho một triển khai thật.

## Phần của bên triển khai

- **20%** **Đánh giá tác động bảo vệ dữ liệu.** Có một bộ tài liệu để điều chỉnh: một bản đánh giá tác động, một luồng dữ liệu, một hồ sơ xử lý và một lịch trình lưu giữ ([spec/governance](../../../spec/governance/dpia.md)). Dựa trên: bộ tài liệu. Còn lại: chưa có bên triển khai, nên chưa có cán bộ bảo vệ dữ liệu nào phê duyệt một bản đánh giá, một cơ sở pháp lý, một lịch trình lưu giữ hay một thỏa thuận với bên xử lý (cửa ải WPM-G5).
- **15%** **Đánh giá tác động về bình đẳng.** Có một mẫu và một sổ đăng ký ([equality-impact](../../../spec/equality-impact/template.md)). Dựa trên: mẫu. Còn lại: bên triển khai viết và phê duyệt một bản đánh giá cho mục đích sử dụng của mình.
- **40%** **Lưu trữ và vận hành.** Có một cấu hình sản xuất, một ảnh container và các tài liệu về triển khai và nâng cấp. Dựa trên: các tài liệu đó và buổi diễn tập khôi phục. Còn lại: chưa chạy gì trên một máy chủ được gia cố, và một bên triển khai phải vận hành dịch vụ.
- **10%** **Hỗ trợ.** Có một trình theo dõi vấn đề và một chính sách bảo mật ([SECURITY.md](../../../SECURITY.md)). Dựa trên: hai thứ đó. Còn lại: chưa có hợp đồng hỗ trợ, mức dịch vụ, lịch trực hay tổ chức nào đứng sau WPM.

## Trang này không khẳng định gì

Trang này không ghi nhận quyết định đưa vào vận hành. Quyết định đó thuộc về một bên triển khai và được đưa ra ngoài WPM. Các ước lượng chỉ bao quát những mục được liệt kê, và danh sách những gì đánh giá mức độ sẵn sàng không kiểm tra cũng áp dụng ở đây. Nơi một hệ thống lâm sàng được kết nối, một cuộc rà soát an toàn lâm sàng là quyết định của bên triển khai.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/path-to-production/`), bản tiếng Anh được ưu tiên.
