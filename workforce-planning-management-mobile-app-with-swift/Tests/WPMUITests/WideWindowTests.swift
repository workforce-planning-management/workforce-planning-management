import Testing
@testable import WPMUI

/// WPM-R285: the layout follows the window's width, not the device's name.
@Suite struct WideWindowTests {
    @Test func theBoundaryIsSixHundredPoints() {
        #expect(WideWindow.isWide(width: 599.9) == false)
        #expect(WideWindow.isWide(width: 600))
        #expect(WideWindow.isWide(width: 1024))
        #expect(WideWindow.minimumWidth == 600)
    }

    @Test func aPhoneInPortraitStacksAndATabletInLandscapeDoesNot() {
        #expect(WideWindow.isWide(width: 393) == false)   // iPhone, portrait
        #expect(WideWindow.isWide(width: 744) == true)    // iPad mini, portrait
        #expect(WideWindow.isWide(width: 320) == false)   // iPad Slide Over
    }
}
