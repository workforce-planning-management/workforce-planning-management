# Thông báo về quyền riêng tư

Thông báo này nêu trang web Hoạch định nhân lực (Workforce Planning Management, WPM) và ứng dụng WPM lưu gì trong
trình duyệt của bạn, vì sao, trong bao lâu và cách xóa chúng. Trang web là
<https://workforce-planning-management.github.io/>; ứng dụng là phần mềm trình diễn WPM, dù nó
được chạy ở đâu. (Cập nhật 2026-11-17, WPM-R253.)

Cả hai đều không có công cụ phân tích, quảng cáo, hay tập lệnh, phông chữ hoặc hình ảnh của bên thứ ba. Không có gì mà cả hai lưu trong
trình duyệt của bạn được dùng để theo dõi bạn từ trang web này sang trang web khác.

## Trang web

Trang web là một tập hợp các trang tĩnh. Nó không có đăng nhập và không có biểu mẫu. Những gì nó lưu nằm trong chính trình duyệt của bạn:

| Cái gì | Lưu ở đâu | Tên | Lưu gì | Vì sao |
| --- | --- | --- | --- | --- |
| Giao diện của bạn | Bộ nhớ cục bộ của trình duyệt | `mxi.wpm.theme` | Tên của giao diện, ví dụ `dark` | Để trang tiếp theo và lần truy cập sau của bạn trông đúng như bạn đã chọn |
| Giao diện của bạn | Một cookie chỉ dành cho trang web này | `wpm-theme` | Tên của giao diện, ví dụ `dark` | Để giao diện quay lại ngay cả khi trình duyệt giữ cookie nhưng xóa bộ nhớ cục bộ, hoặc ngược lại |
| Cỡ chữ của bạn | Bộ nhớ cục bộ của trình duyệt | `mxi.wpm.text-size` | Tên của cỡ chữ, ví dụ `large` | Để trang tiếp theo và lần truy cập sau của bạn dùng cỡ chữ bạn đã chọn |
| Ngôn ngữ của bạn | Không lưu gì | | | Ngôn ngữ là một phần địa chỉ của mỗi trang (ví dụ `/cy-001/`), nên không có gì để ghi nhớ |
| Vị trí của bạn trên một trang | Bộ nhớ phiên của trình duyệt, chỉ cho thẻ này | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Vị trí cuộn và thứ tự các trang bạn đã mở trong thẻ này | Để Quay lại và Tiến tới đưa bạn về chỗ bạn đã ở. Khung phần mềm dùng để xây dựng trang web (SvelteKit) lưu những mục này |

Không có lựa chọn nào được lưu cho đến khi bạn thực hiện. Cho đến khi bạn chọn, trang web theo thiết lập sáng hoặc tối của trình duyệt
và cỡ chữ thông thường của nó, và không lưu giao diện hay cỡ chữ. Một lựa chọn được lưu khi bạn thực hiện nó trong thanh chọn,
và một lựa chọn mới thay thế nó. Bộ nhớ phiên được ghi khi bạn di chuyển giữa các trang.

Các phần khác của trang web không lưu gì:

- **Tìm kiếm** chạy trong trình duyệt của bạn, trên một chỉ mục mà trang web công bố, và không lưu gì. Từ khóa của bạn được đặt trong
  địa chỉ của trang (`?q=`) để có thể chia sẻ một trang kết quả; khi địa chỉ đó được tải từ máy chủ web (khi
  tải lại hoặc từ một liên kết được chia sẻ), máy chủ nhận nó cùng yêu cầu, như với bất kỳ địa chỉ nào.
- **Các thanh trượt mức độ sẵn sàng** trong lộ trình đến môi trường sản xuất không lưu gì, và việc tải lại khôi phục
  các ước lượng của tác giả.
- **Chia sẻ** chỉ mở trang web bạn chọn (email, LinkedIn, Reddit, Bluesky hoặc Mastodon) khi bạn chọn nó. Thông báo về
  quyền riêng tư của trang đó khi ấy áp dụng.

## Ứng dụng WPM

Ứng dụng lưu cùng các loại lựa chọn, và cookie giữ cho bạn đăng nhập:

| Cái gì | Lưu ở đâu | Tên | Lưu gì | Vì sao |
| --- | --- | --- | --- | --- |
| Giao diện của bạn | Bộ nhớ cục bộ của trình duyệt | `mxi.wpm.theme` | Tên của giao diện | Để mọi trang trông đúng như bạn đã chọn |
| Cỡ chữ của bạn | Bộ nhớ cục bộ của trình duyệt | `mxi.wpm.text-size` | Tên của cỡ chữ | Để mọi trang dùng cỡ chữ bạn đã chọn |
| Ngôn ngữ của bạn | Bộ nhớ cục bộ của trình duyệt | `mxi.wpm.locale` | Mã ngôn ngữ của trang gần nhất bạn đã mở, ví dụ `cy-001` | Để một địa chỉ không có ngôn ngữ mở ra bằng ngôn ngữ của bạn |
| Ngôn ngữ của bạn | Một cookie chỉ dành cho ứng dụng | `wpm-locale` | Cùng mã ngôn ngữ đó | Để máy chủ có thể chuyển một địa chỉ không có ngôn ngữ sang ngôn ngữ của bạn |
| Đăng nhập của bạn | Một cookie chỉ dành cho ứng dụng, mà tập lệnh của trang không đọc được | `__Host-mxi_session` (`mxi_session` trên máy riêng của nhà phát triển) | Một số ngẫu nhiên đại diện cho phiên đăng nhập của bạn | Để bạn vẫn đăng nhập khi chuyển trang; nếu không có nó, bạn sẽ phải đăng nhập ở mỗi trang |
| Ngôn ngữ của trình soạn thảo nội dung | Bộ nhớ cục bộ của trình duyệt, chỉ khi bạn mở trình soạn thảo nội dung (`/admin`) | `sveltia-cms.prefs` | Các thiết lập của trình soạn thảo, trong đó có ngôn ngữ của nó | Để trình soạn thảo mở bằng ngôn ngữ của ứng dụng |
| Vị trí của bạn trên một trang | Bộ nhớ phiên của trình duyệt, chỉ cho thẻ này | `sveltekit:history-info`, `sveltekit:navigation-snapshot`, `sveltekit:snapshot` | Vị trí cuộn và thứ tự các trang bạn đã mở trong thẻ này | Để Quay lại và Tiến tới đưa bạn về chỗ bạn đã ở |

Khác với trang web, ứng dụng lưu giao diện và cỡ chữ ngay khi nó hiển thị chúng, trước khi bạn
chọn, và lưu ngôn ngữ của mỗi trang bạn mở. Các khóa từ trước khi dự án đổi tên (`mxi.hcm.theme`, `mxi.hcm.locale`) được chuyển sang
các tên nêu trên và bị xóa.

Cookie đăng nhập là mục duy nhất đại diện cho bạn. Số ngẫu nhiên trong đó tự nó không có ý nghĩa gì: phiên
đăng nhập mà nó trỏ tới được máy chủ của ứng dụng lưu giữ, và đăng xuất chấm dứt nó. Những gì ứng dụng
lưu về những người có trong hồ sơ của nó được nêu ở phần áp chót, bên dưới.

## Ai nhận chúng

Bộ nhớ cục bộ và bộ nhớ phiên không bao giờ rời khỏi trình duyệt của bạn.

Một cookie được gửi kèm mỗi yêu cầu mà trình duyệt của bạn gửi đến trang đã đặt nó, và không gửi đến trang nào khác
(`SameSite=Lax`, và `Secure` qua HTTPS):

- Cookie `wpm-theme` của trang web đi đến GitHub Pages, nơi lưu trữ trang web. Không chương trình nào của dự án
  đọc nó.
- Các cookie `wpm-locale` và đăng nhập của ứng dụng đi đến máy chủ mà ứng dụng chạy trên đó, nơi đọc chúng
  để chọn ngôn ngữ của bạn và biết rằng bạn đã đăng nhập.

GitHub, nơi lưu trữ trang web, nhận các yêu cầu mà trình duyệt của bạn gửi, như bất kỳ máy chủ web nào.
[Tuyên bố về quyền riêng tư của GitHub](https://docs.github.com/en/site-policy/privacy-policies/github-general-privacy-statement)
nêu những gì nó ghi lại. Tổ chức vận hành ứng dụng quyết định máy chủ của mình ghi lại những gì.

## Chúng được giữ bao lâu

- Cookie `wpm-theme` của trang web: 400 ngày dương lịch sau lần cuối bạn chọn giao diện, thời gian dài nhất mà một trình duyệt giữ một
  cookie.
- Cookie `wpm-locale` của ứng dụng: 365 ngày dương lịch sau lần cuối bạn mở một trang.
- Cookie đăng nhập: cho đến khi bạn đăng xuất hoặc đóng trình duyệt.
- Bộ nhớ cục bộ: cho đến khi bạn xóa nó.
- Bộ nhớ phiên: cho đến khi bạn đóng thẻ.

## Cách xóa chúng

Xóa dữ liệu của trang trong phần cài đặt của trình duyệt (cookie và dữ liệu trang), cho trang web, cho ứng dụng, hoặc
cho cả hai. Trang web khi đó hoạt động như lần truy cập đầu tiên; ứng dụng yêu cầu bạn đăng nhập lại. Đăng xuất khỏi
ứng dụng sẽ xóa cookie đăng nhập.

## Vì sao không có biểu ngữ xin đồng ý

Mỗi mục được lưu để cung cấp cho bạn dịch vụ mà bạn yêu cầu: trang web hoặc ứng dụng theo cách bạn đã thiết lập, bằng
ngôn ngữ của bạn và, trong ứng dụng, ở trạng thái đã đăng nhập. Đó là loại lưu trữ mà các quy tắc về cookie của Vương quốc Anh và EU (the
Privacy and Electronic Communications Regulations và Chỉ thị ePrivacy) coi là thật sự cần thiết, và không
cần sự đồng ý. Đây là cách dự án hiểu các quy tắc đó, không phải tư vấn pháp lý.

## Các hồ sơ mà ứng dụng lưu giữ

Thông báo này nói về những gì được lưu trong trình duyệt của bạn. Ứng dụng cũng lưu các hồ sơ về con người (nhân viên,
ứng viên và người liên hệ của họ) trên máy chủ của nó. Bên vận hành ứng dụng chịu trách nhiệm về các hồ sơ đó và
viết thông báo về quyền riêng tư cho những người có trong đó
([danh sách kiểm tra đánh giá](../../../spec/governance/assessment-checklist.md), câu hỏi 3.10). Phần mềm trình diễn
chỉ lưu dữ liệu tổng hợp.

## Câu hỏi

Hãy hỏi trong kho mã của dự án, <https://github.com/workforce-planning-management/workforce-planning-management>,
hoặc viết thư cho người bảo trì được nêu ở đó.

> Bản dịch này do AI thực hiện và chưa được người bản ngữ tiếng Việt rà soát. Nếu có điểm khác với bản tiếng Anh (`locales/en-001/privacy/`), bản tiếng Anh được ưu tiên.
