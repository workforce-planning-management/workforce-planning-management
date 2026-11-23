// Request leave (spec/mobile-apps.md, WPM-R283): a kind from the closed list, two dates, and a reason except for sick
// leave, which has no reason field. The days the dates span are shown before sending; the service still decides.
import LilyUI
import SwiftUI
import WPMCore

struct LeaveRequestForm: View {
    let model: TimeOffModel
    @Environment(\.dismiss) private var dismiss
    @State private var kind = LeaveRules.kinds[0]
    @State private var start = Date()
    @State private var end = Date()
    @State private var reason = ""
    @State private var problem: String?

    var draft: LeaveDraft {
        let calendar = Calendar.current
        return LeaveDraft(kind: kind, startOn: CalendarDate(start, in: calendar), endOn: CalendarDate(end, in: calendar),
                          reason: reason)
    }

    var body: some View {
        NavigationStack {
            Form {
                if let problem {
                    ErrorSummary(title: L("error.title"), problems: [.init(problem) {}])
                }
                Section {
                    Picker(L("request.kind"), selection: $kind) {
                        ForEach(LeaveRules.kinds, id: \.self) { Text(Words.kind($0)).tag($0) }
                    }
                    DatePicker(L("request.start"), selection: $start, displayedComponents: .date)
                    DatePicker(L("request.end"), selection: $end, displayedComponents: .date)
                    Text(draft.days.map(Words.days) ?? L("request.noSpan"))
                        .accessibilityLabel(L("request.spanLabel", draft.days.map(Words.days) ?? L("request.noSpan")))
                }
                if LeaveRules.reasonAllowed(kind) {
                    Section {
                        Field(label: L("request.reason"), description: L("request.reason.hint", LeaveRules.reasonMaxLength)) {
                            TextField("", text: $reason, axis: .vertical).lineLimit(3...6)
                                .textFieldStyle(.roundedBorder)
                        }
                    }
                } else {
                    Section { InsetText { Text(L("request.sickNoReason")) } }
                }
                Section {
                    Button(L("request.send")) { Task { await send() } }
                        .disabled(model.sending)
                        .frame(minHeight: 44)
                }
            }
            .navigationTitle(L("request.title"))
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button(L("action.cancel")) { dismiss() } }
            }
        }
    }

    func send() async {
        problem = await model.submit(draft)
        if problem == nil { dismiss() }
    }
}
