import Foundation
import GRPC
import NIO

public class EnigmaAuthGrpcClient {
    private let client: Auth_AuthNIOClient
    private let eventLoopGroup: EventLoopGroup
    private let timeout: TimeLimit = .timeout(.seconds(5))

    public init() {
        self.eventLoopGroup = MultiThreadedEventLoopGroup(numberOfThreads: 1)

        let channel = ClientConnection
//            .usingPlatformAppropriateTLS(for: self.eventLoopGroup) // Use TLS if needed
            .insecure(group: self.eventLoopGroup)
            .connect(host: "localhost", port: 50051) // Update with your server address and port

        self.client = Auth_AuthNIOClient(channel: channel)
    }

    deinit {
        try? self.eventLoopGroup.syncShutdownGracefully()
    }

    public func login(username: String, password: String, completion: @escaping @Sendable (Result<String, Error>) -> Void) {
        let request = Auth_LoginRequest.with {
            $0.name = username
            $0.password = password
        }

        let callOptions = CallOptions(timeLimit: timeout)

        let call = client.login(request, callOptions: callOptions)

        call.response.whenComplete { result in
            switch result {
            case .success(let reply):
                completion(.success(reply.accessToken))
            case .failure(let error):
                completion(.failure(error))
            }
        }
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
}
