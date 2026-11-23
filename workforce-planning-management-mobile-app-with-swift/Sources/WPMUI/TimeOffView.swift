// My time off (spec/mobile-apps.md, WPM-R282): balances, the year's requests and past absence, in calendar days, with
// the service's derivation. A missing allowance is "not recorded", never 0. Cancel where the service would allow it.
import LilyUI
import SwiftUI
import WPMCore

struct TimeOffView: View {
    let session: Session
    @State private var model: TimeOffModel
    @State private var requesting = false
    @State private var confirming: LeaveRequest?

    init(session: Session) {
        self.session = session
        _model = State(initialValue: TimeOffModel(session: session))
    }

    /// For previews and tests: a model already holding an answer.
    init(session: Session, model: TimeOffModel) {
        self.session = session
        _model = State(initialValue: model)
    }

    var body: some View {
        content
            .navigationTitle(L("timeOff.title"))
            .toolbar { toolbar }
            .task { await model.load() }
            .refreshable { await model.load() }
            .sheet(isPresented: $requesting) { LeaveRequestForm(model: model) }
            .confirmationDialog(
                L("cancel.confirm.title"), isPresented: Binding(get: { confirming != nil },
                                                                set: { if !$0 { confirming = nil } }),
                titleVisibility: .visible, presenting: confirming
            ) { request in
                Button(L("cancel.confirm.action"), role: .destructive) { Task { await model.cancel(request) } }
                Button(L("cancel.confirm.keep"), role: .cancel) {}
            } message: { request in
                Text(L("cancel.confirm.message", Words.kind(request.kind), Words.range(request.startOn, request.endOn)))
            }
    }

    @ViewBuilder var content: some View {
        switch model.state {
        case .loading:
            ProgressView(L("timeOff.loading"))
        case .failed(let message):
            Form {
                ErrorSummary(title: L("error.title"), problems: [.init(message) { Task { await model.load() } }])
                Button(L("action.retry")) { Task { await model.load() } }.frame(minHeight: 44)
            }
        case .loaded(let timeOff):
            TimeOffContent(timeOff: timeOff, model: model) { confirming = $0 }
        }
    }

    @ToolbarContentBuilder var toolbar: some ToolbarContent {
        ToolbarItem(placement: .primaryAction) {
            Button(L("request.open")) { requesting = true }
                .disabled(!isLoaded)
        }
        ToolbarItem(placement: .secondaryAction) {
            Button(L("signOut.action"), role: .destructive) { session.signOut() }
        }
    }

    var isLoaded: Bool {
        if case .loaded = model.state { return true }
        return false
    }
}

struct TimeOffContent: View {
    let timeOff: TimeOff
    let model: TimeOffModel
    let askCancel: (LeaveRequest) -> Void

    var body: some View {
        List {
            Section {
                HStack {
                    Button { Task { await model.show(year: timeOff.year - 1) } } label: {
                        Image(systemName: "chevron.backward").frame(minWidth: 44, minHeight: 44)
                    }
                    .accessibilityLabel(L("year.previous", timeOff.year - 1))
                    Spacer()
                    Text(String(timeOff.year)).font(.headline).accessibilityAddTraits(.isHeader)
                    Spacer()
                    Button { Task { await model.show(year: timeOff.year + 1) } } label: {
                        Image(systemName: "chevron.forward").frame(minWidth: 44, minHeight: 44)
                    }
                    .accessibilityLabel(L("year.next", timeOff.year + 1))
                }
                .buttonStyle(.borderless)
                Text(L("timeOff.unit")).font(.footnote)
            }

            if let problem = model.actionProblem {
                Section { ErrorSummary(title: L("error.title"), problems: [.init(problem) {}]) }
            }

            Section(L("timeOff.balances")) {
                if timeOff.balances.isEmpty { Text(L("timeOff.noBalances")) }
                ForEach(timeOff.balances) { balance in
                    SummaryList(label: Words.kind(balance.kind)) {
                        Text(Words.kind(balance.kind)).font(.headline).accessibilityAddTraits(.isHeader)
                        SummaryListItem(term: L("balance.allowance")) { Text(Words.figure(DayFigure(balance.entitled))) }
                        SummaryListItem(term: L("balance.taken")) { Text(Words.days(balance.taken)) }
                        SummaryListItem(term: L("balance.booked")) { Text(Words.days(balance.booked)) }
                        SummaryListItem(term: L("balance.requested")) { Text(Words.days(balance.requested)) }
                        SummaryListItem(term: L("balance.remaining")) {
                            Text(Words.figure(DayFigure(balance.remaining)))
                        }
                    }
                }
            }

            Section(L("timeOff.requests")) {
                if timeOff.requests.isEmpty { Text(L("timeOff.noRequests")) }
                ForEach(timeOff.requests) { request in
                    RequestRow(request: request, cancellable: model.canCancel(request),
                               disabled: model.sending) { askCancel(request) }
                }
            }

            Section(L("timeOff.history")) {
                if timeOff.history.isEmpty { Text(L("timeOff.noHistory")) }
                ForEach(timeOff.history) { year in
                    SummaryList(label: String(year.year)) {
                        Text(String(year.year)).font(.headline).accessibilityAddTraits(.isHeader)
                        ForEach(year.absences) { total in
                            SummaryListItem(term: Words.kind(total.kind)) {
                                Text(L("history.total", total.absences, Words.days(total.days)))
                            }
                        }
                    }
                }
            }

            Section(L("timeOff.derivation")) {
                InsetText { Text(timeOff.derivation) }
                Text(L("timeOff.asOf", Words.date(timeOff.asOf))).font(.footnote)
            }
        }
    }
}

struct RequestRow: View {
    let request: LeaveRequest
    let cancellable: Bool
    let disabled: Bool
    let cancel: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline) {
                Text(Words.kind(request.kind)).font(.headline)
                Spacer()
                Tag(Words.status(request.status), label: L("request.status"))
            }
            Text(Words.range(request.startOn, request.endOn))
            Text(Words.days(request.days)).font(.subheadline)
            // The service sends no reason for sick leave (WPM-D73).
            if let reason = request.reason, !reason.isEmpty {
                Text(reason).font(.subheadline)
            }
            if cancellable {
                Button(L("cancel.open"), role: .destructive, action: cancel)
                    .disabled(disabled)
                    .frame(minHeight: 44)
                    .buttonStyle(.borderless)
            }
        }
        .padding(.vertical, 4)
    }
}
