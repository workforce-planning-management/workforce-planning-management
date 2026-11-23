// What a management tool pre-sets (spec/mobile-apps.md, WPM-R285): the deployment's address and whether it may be changed.
// Read from the platform's managed app configuration (`com.apple.configuration.managed`). Absent is not an error: the person
// types the address. Supplied but invalid is refused and named, never replaced by a default or by asking (RCA-IN-2, RCA-IN-4).
import Foundation

public struct ManagedConfiguration: Equatable, Sendable {
    /// The address the tool set, or nil when it set none.
    public let connection: Connection?
    /// True when the tool asked that the address not be changed.
    public let locked: Bool

    public static let urlKey = "deployment_url"
    public static let lockedKey = "deployment_locked"
    /// The key iOS keeps the tool's values under in the app's defaults.
    public static let defaultsKey = "com.apple.configuration.managed"

    /// The setting that was wrong, and why.
    public enum Problem: Error, Equatable, Sendable {
        case deploymentURL(Connection.Problem)
        case deploymentURLNotText
        case deploymentLocked
        /// `deployment_locked` is `true` and there is no `deployment_url`, which would leave no way to connect.
        case lockedWithoutURL
    }

    public static let none = ManagedConfiguration(connection: nil, locked: false)

    /// Reads the tool's values. A boolean is a `Bool` or the text `true` or `false`; nothing else is a boolean.
    public static func read(_ values: [String: Any]?) -> Result<ManagedConfiguration, Problem> {
        guard let values else { return .success(.none) }
        var connection: Connection?
        if let raw = values[urlKey] {
            guard let text = raw as? String else { return .failure(.deploymentURLNotText) }
            switch Connection.parse(text) {
            case .failure(let problem): return .failure(.deploymentURL(problem))
            case .success(let c): connection = c
            }
        }
        var locked = false
        if let raw = values[lockedKey] {
            if let flag = raw as? Bool, !(raw is NSNumber) || CFGetTypeID(raw as CFTypeRef) == CFBooleanGetTypeID() {
                locked = flag
            } else if let text = raw as? String, text == "true" || text == "false" {
                locked = text == "true"
            } else {
                return .failure(.deploymentLocked)
            }
        }
        if locked && connection == nil { return .failure(.lockedWithoutURL) }
        return .success(ManagedConfiguration(connection: connection, locked: locked))
    }
}
