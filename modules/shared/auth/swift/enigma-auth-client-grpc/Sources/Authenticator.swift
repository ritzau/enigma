import Foundation
import GRPC
import Security

public enum EnigmaError: Error {
    case missingSession
    case alreadyAuthenticated
    case unauthorizedAction
    case invalidResponse
}

public struct UserId: Sendable {
    public var value: Int64

    public init(value: Int64) {
        self.value = value
    }
}

public struct Username: Sendable {
    public var value: String

    public init(value: String) {
        self.value = value
    }
}

public struct AccessToken : Sendable {
    public var value: UUID
}

public struct RefreshToken : Sendable {
    public var value: UUID
}

@available(iOS 13.0.0, *)
public actor Session {
    private var userId: UserId
    private var username: Username
    private var accessToken: AccessToken
    private var refreshToken: RefreshToken

    init(userId: UserId, username: Username, accessToken: AccessToken, refreshToken: RefreshToken) {
        self.userId = userId
        self.username = username
        self.accessToken = accessToken
        self.refreshToken = refreshToken
    }

    static func load() async -> Session? {
        print("Loading session")

        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrAccount as String: "userSession",
            kSecReturnData as String: kCFBooleanTrue!,
            kSecMatchLimit as String: kSecMatchLimitOne
        ]

        var dataTypeRef: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &dataTypeRef)

        if status == errSecSuccess, let data = dataTypeRef as? Data,
           let sessionData = try? JSONSerialization.jsonObject(with: data, options: []) as? [String: Any],
           let userIdValue = sessionData["userId"] as? Int64,
           let usernameValue = sessionData["username"] as? String,
           let accessTokenString = sessionData["accessToken"] as? String,
           let refreshTokenString = sessionData["refreshToken"] as? String,
           let accessTokenUUID = UUID(uuidString: accessTokenString),
           let refreshTokenUUID = UUID(uuidString: refreshTokenString) {

            return Session(
                userId: UserId(value: userIdValue),
                username: Username(value: usernameValue),
                accessToken: AccessToken(value: accessTokenUUID),
                refreshToken: RefreshToken(value: refreshTokenUUID)
            )
        }

        return nil
    }

    func store() async {
        print("Storing session")
        let sessionData: [String: Any] = [
            "userId": userId.value,
            "username": username.value,
            "accessToken": accessToken.value.uuidString,
            "refreshToken": refreshToken.value.uuidString,
        ]

        if let data = try? JSONSerialization.data(withJSONObject: sessionData, options: []) {
            let query: [String: Any] = [
                kSecClass as String: kSecClassGenericPassword,
                kSecAttrAccount as String: "userSession",
                kSecValueData as String: data,
                kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlock
            ]

            SecItemDelete(query as CFDictionary) // Remove any old session before saving
            SecItemAdd(query as CFDictionary, nil)
        }
    }

    func updateSession(accessToken: AccessToken, refreshToken: RefreshToken) async {
        self.accessToken = accessToken
        self.refreshToken = refreshToken
        await store()
    }

    public func getUserId() async -> UserId {
        return userId
    }

    public func getUsername() async -> Username {
        return username
    }

    func getAccessToken() async -> AccessToken {
        return accessToken
    }

    func getRefreshToken() async -> RefreshToken {
        return refreshToken
    }
}

@available(iOS 13.0.0, *)
public actor Authenticator {
    private var client: Auth_AuthAsyncClient
    private var session: Session?

    init(client: Auth_AuthAsyncClient, session: Session?) {
        self.client = client
        self.session = session
    }

    func loadSession() async {
        self.session = await Session.load()
    }

    public func getSession() async -> Session? {
        return session
    }

    func resetSession(session: Session? = nil) async {
        print("Resetting session \(String(describing: session))")
        self.session = session
    }

    public func authenticatedCall<Ret: Sendable>(
        requestFn: @escaping (CallOptions) async throws -> Ret
    ) async throws -> Ret {
        do {
            return try await callWithAccessToken(requestFn: requestFn)
        } catch let error as GRPCStatus {
            if case .unauthenticated = error.code {
                try await refreshAndUpdateSession()
                return try await callWithAccessToken(requestFn: requestFn)
            } else {
                throw error
            }
        }
    }

    private func callWithAccessToken<Ret: Sendable>(requestFn: (CallOptions) async throws -> (Ret)) async throws -> Ret {
        guard let session else { throw EnigmaError.missingSession }

        let tokenString = await session.getAccessToken().value.uuidString

        var callOptions = CallOptions()
        callOptions.customMetadata.add(name: "authorization", value: "Bearer \(tokenString)")

        return try await requestFn(callOptions)
    }

    private func refreshAndUpdateSession() async throws {
        guard let session else { throw EnigmaError.missingSession }

        let oldRefreshToken = await session.getRefreshToken()
        let (accessToken, refreshToken) = try await refreshSession(refreshToken: oldRefreshToken)
        await session.updateSession(accessToken: accessToken, refreshToken: refreshToken)
    }

    private func refreshSession(refreshToken: RefreshToken) async throws -> (AccessToken, RefreshToken) {
        var request = Auth_RefreshSessionRequest()
        request.refreshToken = refreshToken.value.uuidString

        let response = try await client.refreshSession(request)
        guard let accessTokenUuid = UUID(uuidString: response.accessToken),
              let refreshTokenUuid = UUID(uuidString: response.refreshToken) else {
            print("❌ Failed to parse tokens: accessToken=\(response.accessToken), refreshToken=\(response.refreshToken)")
            throw EnigmaError.invalidResponse
        }

        let accessToken = AccessToken(value: accessTokenUuid)
        let refreshToken = RefreshToken(value: refreshTokenUuid)

        return (accessToken, refreshToken)
    }
}
