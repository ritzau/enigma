import Foundation
import GRPC
import NIO

enum EnigmaAuthError: Error {
    case noSession
    case invalidResponse
}

@available(iOS 13, *)
public final class EnigmaAuthGrpcClient: EnigmaAuthClient, Sendable {
    private let client: Auth_AuthNIOClient
    private let asyncClient: Auth_AuthAsyncClient
    private let authenticator: Authenticator
    private let eventLoopGroup: EventLoopGroup
    private let timeout: TimeLimit = .timeout(.seconds(5))

    public init() {
        self.eventLoopGroup = MultiThreadedEventLoopGroup(numberOfThreads: 1)

        let channel = ClientConnection
//            .usingPlatformAppropriateTLS(for: self.eventLoopGroup) // Use TLS if needed
            .insecure(group: self.eventLoopGroup)
            .connect(host: "localhost", port: 50051) // Update with your server address and port

        self.client = Auth_AuthNIOClient(channel: channel)
        self.asyncClient = Auth_AuthAsyncClient(channel: channel)
        self.authenticator = Authenticator(client: self.asyncClient, session: nil)
    }

    deinit {
        try? self.eventLoopGroup.syncShutdownGracefully()
    }

    public func loadSession() async {
        await authenticator.loadSession()
    }

    public func getAuthenticator() -> Authenticator {
        return authenticator
    }

    public func validateSession() async throws -> UserInfo {
        guard let session = await authenticator.getSession() else {
            throw EnigmaAuthError.noSession
        }

        let userId = await session.getUserId()
        return try await getUserInfo(userId: userId)
    }

    public func login(username: String, password: String) async throws -> UserId  {
        let request = Auth_LoginRequest.with {
            $0.name = username
            $0.password = password
        }

        let callOptions = CallOptions(timeLimit: timeout)

        let reply = try await asyncClient.login(request, callOptions: callOptions)

        guard let accessTokenUuid = UUID(uuidString: reply.accessToken),
              let refeshTokenUuid = UUID(uuidString: reply.refreshToken) else {
            await self.authenticator.resetSession()
            throw EnigmaAuthError.invalidResponse
        }

        let userId = UserId(value: reply.userID)

        let session = Session(
            userId: userId,
            username: Username(value: username),
            accessToken: AccessToken(value: accessTokenUuid),
            refreshToken: RefreshToken(value: refeshTokenUuid))

        await self.authenticator.resetSession(session: session)

        return userId
    }

    public func createAccount(username: String, password: String, completion: @escaping @Sendable (Result<Int64, Error>) -> Void) {
        let request = Auth_CreateAccountRequest.with {
            $0.username = username
            $0.password = password
        }

        let callOptions = CallOptions(timeLimit: timeout)

        let call = client.createAccount(request, callOptions: callOptions)

        call.response.whenComplete { result in
            switch result {
            case .success(let reply):
                completion(.success(reply.userID))
            case .failure(let error):
                completion(.failure(error))
            }
        }
    }

    public func getUserInfo(userId: UserId) async throws -> UserInfo {
        let request = Auth_GetUserInfoRequest.with {
            $0.userID = userId.value
        }

        let client = self.asyncClient

        return try await authenticator.authenticatedCall { options in
            try await client.getUserInfo(request, callOptions: options)
        }.toUserInfo()
    }

    public func listAccounts() async throws -> [EnigmaAccount] {
        let request = Auth_ListAccountsRequest()

        let client = self.asyncClient

        let reply = try await authenticator.authenticatedCall { options in
            try await client.listAccounts(request, callOptions: options)
        }

        return reply.users.map {
            EnigmaAccount(accountId: UserId(value: $0.id), accountname: Username(value: $0.name))
        }
    }
}

extension Auth_GetUserInfoReply {
    func toUserInfo() -> UserInfo {
        UserInfo(
            userId: UserId(value: self.userID),
            username: Username(value: self.username),
            roles: self.roles.map { Role(value: $0) }
        )
    }
}

public struct EnigmaAccount: Sendable, Hashable {
    public static func == (lhs: EnigmaAccount, rhs: EnigmaAccount) -> Bool {
        lhs.accountId.value == rhs.accountId.value
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(accountId.value)
    }

    public var accountId: UserId
    public var accountname: Username

    public init(accountId: UserId, accountname: Username) {
        self.accountId = accountId
        self.accountname = accountname
    }
}

public struct Role: Sendable {
    public var value: String
}

public struct UserInfo: Sendable {
    public var userId: UserId
    public var username: Username
    public var roles: [Role]

}
