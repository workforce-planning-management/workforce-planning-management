// Requests to the service and their answers (spec/mobile-apps.md, WPM-R280). Every request carries
// `Accepts-version: 1.0` and `Accept: application/json`; a signed-in one carries the bearer token. The transport is
// passed in, so the client is tested with no network.
import Foundation

public struct APIRequest: Equatable, Sendable {
    public let method: String
    public let url: URL
    public let headers: [String: String]
    public let body: Data?

    public init(method: String, url: URL, headers: [String: String], body: Data?) {
        self.method = method
        self.url = url
        self.headers = headers
        self.body = body
    }
}

public struct APIResponse: Equatable, Sendable {
    public let status: Int
    public let body: Data

    public init(status: Int, body: Data) {
        self.status = status
        self.body = body
    }
}

/// Sends a request and returns the status and body. Thrown errors are "could not reach the service".
public protocol Transport: Sendable {
    func send(_ request: APIRequest) async throws -> APIResponse
}

public enum APIError: Error, Equatable, Sendable {
    /// `401`: the token is missing, expired or revoked. The app signs the person out.
    case unauthorized
    /// The service refused, in its own words when it gave any.
    case refused(status: Int, problem: ProblemDetails?)
    /// The answer was not what the service documents.
    case unreadable
    /// The service could not be reached.
    case unreachable
}

public enum Endpoints {
    public static let versionHeader = "Accepts-version"
    public static let version = "1.0"

    static func headers(token: String?, json: Bool) -> [String: String] {
        var headers = ["Accept": "application/json", versionHeader: version]
        if json { headers["Content-Type"] = "application/json" }
        if let token { headers["Authorization"] = "Bearer \(token)" }
        return headers
    }

    public static func timeOff(_ connection: Connection, token: String, year: Int?) -> APIRequest {
        let query = year.map { [URLQueryItem(name: "year", value: String($0))] } ?? []
        return APIRequest(method: "GET", url: connection.url("/api/me/time-off", query: query),
                          headers: headers(token: token, json: false), body: nil)
    }

    struct LeaveBody: Encodable {
        let kind: String
        let startOn: CalendarDate
        let endOn: CalendarDate
        let reason: String?

        enum CodingKeys: String, CodingKey {
            case kind, reason
            case startOn = "start_on"
            case endOn = "end_on"
        }

        func encode(to encoder: any Encoder) throws {
            var c = encoder.container(keyedBy: CodingKeys.self)
            try c.encode(kind, forKey: .kind)
            try c.encode(startOn, forKey: .startOn)
            try c.encode(endOn, forKey: .endOn)
            try c.encodeIfPresent(reason, forKey: .reason)
        }
    }

    public static func requestLeave(_ connection: Connection, token: String, draft: LeaveDraft) -> APIRequest {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        let body = LeaveBody(kind: draft.kind, startOn: draft.startOn, endOn: draft.endOn, reason: draft.reasonToSend)
        // Encoding strings and dates cannot fail.
        let data = (try? encoder.encode(body)) ?? Data()
        return APIRequest(method: "POST", url: connection.url("/api/me/leave-requests"),
                          headers: headers(token: token, json: true), body: data)
    }

    public static func cancelLeave(_ connection: Connection, token: String, pid: String) -> APIRequest {
        let segment = pid.addingPercentEncoding(withAllowedCharacters: .urlPathAllowed.subtracting(["/"])) ?? pid
        return APIRequest(method: "POST", url: connection.url("/api/me/leave-requests/\(segment)/cancel"),
                          headers: headers(token: token, json: false), body: nil)
    }
}

public struct WPMClient: Sendable {
    public let connection: Connection
    public let token: String
    public let transport: any Transport

    public init(connection: Connection, token: String, transport: any Transport) {
        self.connection = connection
        self.token = token
        self.transport = transport
    }

    public func timeOff(year: Int?) async throws(APIError) -> TimeOff {
        try await Self.decode(TimeOff.self, from: send(Endpoints.timeOff(connection, token: token, year: year)))
    }

    public func requestLeave(_ draft: LeaveDraft) async throws(APIError) -> LeaveRequestCreated {
        try await Self.decode(LeaveRequestCreated.self,
                              from: send(Endpoints.requestLeave(connection, token: token, draft: draft)))
    }

    public func cancelLeave(pid: String) async throws(APIError) {
        _ = try await send(Endpoints.cancelLeave(connection, token: token, pid: pid))
    }

    func send(_ request: APIRequest) async throws(APIError) -> APIResponse {
        let response: APIResponse
        do {
            response = try await transport.send(request)
        } catch {
            throw .unreachable
        }
        try Self.check(response)
        return response
    }

    /// A 2xx passes; a 401 is `unauthorized`; any other status is `refused` with the problem details when readable.
    public static func check(_ response: APIResponse) throws(APIError) {
        if (200..<300).contains(response.status) { return }
        if response.status == 401 { throw .unauthorized }
        let problem = try? JSONDecoder().decode(ProblemDetails.self, from: response.body)
        throw .refused(status: response.status, problem: problem)
    }

    public static func decode<T: Decodable>(_ type: T.Type, from response: APIResponse) throws(APIError) -> T {
        do {
            return try JSONDecoder().decode(type, from: response.body)
        } catch {
            throw .unreadable
        }
    }
}
