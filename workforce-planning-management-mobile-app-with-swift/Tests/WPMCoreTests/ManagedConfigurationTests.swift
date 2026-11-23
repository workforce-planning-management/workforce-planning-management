import Foundation
import Testing
@testable import WPMCore

@Suite struct ManagedConfigurationTests {
    private func read(_ values: [String: Any]?) -> Result<ManagedConfiguration, ManagedConfiguration.Problem> {
        ManagedConfiguration.read(values)
    }

    @Test func nothingSetIsNotAnError() {
        #expect(read(nil) == .success(.none))
        #expect(read([:]) == .success(.none))
        #expect(read(["other": 1]) == .success(.none), "unrelated keys are ignored")
    }

    @Test func anAddressAloneIsUsedAndNotLocked() throws {
        let result = try read(["deployment_url": "https://wpm.example.org/"]).get()
        #expect(result.connection?.baseURL.absoluteString == "https://wpm.example.org")
        #expect(result.locked == false)
    }

    @Test func lockedNeedsAnAddress() throws {
        let ok = try read(["deployment_url": "https://wpm.example.org", "deployment_locked": true]).get()
        #expect(ok.locked)
        #expect(read(["deployment_locked": true]) == .failure(.lockedWithoutURL))
        #expect(try read(["deployment_locked": false]).get() == .none, "unlocked with no address is just absent")
    }

    @Test func booleansAreBoolOrTrueFalseText() throws {
        let url = "https://wpm.example.org"
        #expect(try read(["deployment_url": url, "deployment_locked": "true"]).get().locked)
        #expect(try read(["deployment_url": url, "deployment_locked": "false"]).get().locked == false)
        for bad: Any in ["yes", "TRUE", "1", 1, 0, 2.5, ["true"]] {
            #expect(read(["deployment_url": url, "deployment_locked": bad]) == .failure(.deploymentLocked), "\(bad)")
        }
    }

    @Test func aBadAddressIsRefusedAndNamed() {
        #expect(read(["deployment_url": "http://wpm.example.org"]) == .failure(.deploymentURL(.insecureScheme)))
        #expect(read(["deployment_url": ""]) == .failure(.deploymentURL(.empty)), "supplied but empty is refused, not absent")
        #expect(read(["deployment_url": "https://a.org?x=1"]) == .failure(.deploymentURL(.queryOrFragment)))
        #expect(read(["deployment_url": 5]) == .failure(.deploymentURLNotText))
    }

    @Test func aBadLockedFlagIsRefusedEvenWithAGoodAddress() {
        #expect(read(["deployment_url": "https://wpm.example.org", "deployment_locked": "maybe"]) == .failure(.deploymentLocked))
    }
}
