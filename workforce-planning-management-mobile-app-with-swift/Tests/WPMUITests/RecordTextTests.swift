import Foundation
import Testing
@testable import WPMUI

/// WPM-R286: record text is never copied onto the device. The rule is held two ways: the one decision about hiding the
/// window is a function, and the sources contain no call that copies, shares, drags, prints or hands off text.
@Suite struct RecordTextTests {
    @Test func theWindowIsHiddenUnlessTheSceneIsActiveAndNotCaptured() {
        #expect(PrivacyShield.hides(sceneActive: true, captured: false) == false)
        #expect(PrivacyShield.hides(sceneActive: false, captured: false))
        #expect(PrivacyShield.hides(sceneActive: true, captured: true))
        #expect(PrivacyShield.hides(sceneActive: false, captured: true))
    }

    /// Every source file of the app and its core.
    static func sources() throws -> [(name: String, text: String)] {
        let root = URL(fileURLWithPath: #filePath).deletingLastPathComponent().deletingLastPathComponent()
            .deletingLastPathComponent().appendingPathComponent("Sources")
        let files = try #require(FileManager.default.enumerator(at: root, includingPropertiesForKeys: nil))
        return try files.compactMap { $0 as? URL }.filter { $0.pathExtension == "swift" }
            .map { ($0.lastPathComponent, try String(contentsOf: $0, encoding: .utf8)) }
    }

    @Test func noSourceCopiesSharesDragsPrintsOrHandsOffText() throws {
        let banned = [
            "UIPasteboard", "NSPasteboard", "ShareLink", "UIActivityViewController", "NSUserActivity", ".userActivity(",
            "CSSearchableItem", "CoreSpotlight", ".draggable(", ".onDrag", ".copyable(", "UIPrintInteractionController",
            "UNMutableNotificationContent", "WidgetKit", "textSelection(.enabled)", ".textSelection(.enabled",
            "UIMenuController", "UIEditMenu",
        ]
        let found = try Self.sources().flatMap { file in
            banned.filter { file.text.contains($0) }.map { "\(file.name): \($0)" }
        }
        #expect(found.isEmpty, "record text must not leave the app: \(found)")
    }

    @Test func theRootViewSwitchesTextSelectionOffForEveryScreen() throws {
        let root = try #require(Self.sources().first { $0.name == "RootView.swift" })
        #expect(root.text.contains(".textSelection(.disabled)"))
    }

    @Test func theReasonFieldIsANoLearningField() throws {
        let form = try #require(Self.sources().first { $0.name == "LeaveRequestForm.swift" })
        #expect(form.text.contains(".autocorrectionDisabled()"))
    }
}
