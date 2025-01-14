//
//  AuthenticationService.swift
//  Foorum
//
//  Created by Tobias Ritzau on 2025-01-13.
//

import Foundation
import GRPC
import NIO
import grpc_client_swift

class AuthenticationService {
    private let client: Auth_AuthNIOClient
    private let eventLoopGroup: EventLoopGroup

    init() {
        // Create an EventLoopGroup
        self.eventLoopGroup = MultiThreadedEventLoopGroup(numberOfThreads: 1)

        let fortytwo = foobar();
        print(fortytwo);

        // Set up the gRPC client connection
        let channel = try! ClientConnection
//            .usingPlatformAppropriateTLS(for: self.eventLoopGroup) // Use TLS if needed
            .insecure(group: self.eventLoopGroup) 
            .connect(host: "127.0.0.1", port: 50051) // Update with your server address and port

        // Initialize the gRPC client
        self.client = Auth_AuthNIOClient(channel: channel)
    }

    deinit {
        // Shut down the EventLoopGroup when the service is deinitialized
        try? self.eventLoopGroup.syncShutdownGracefully()
    }

    func login(username: String, password: String, completion: @escaping (Result<String, Error>) -> Void) {
        // Prepare the gRPC request
        let request = Auth_LoginRequest.with {
            $0.name = username
            $0.password = password
        }

        // Set a timeout of 10 seconds
        let callOptions = CallOptions(timeLimit: .timeout(.seconds(5)))

        // Call the Login RPC with the timeout
        let call = client.login(request, callOptions: callOptions)

        call.response.whenComplete { result in
            switch result {
            case .success(let reply):
                completion(.success(reply.accessToken)) // Return access token on success
            case .failure(let error):
                completion(.failure(error)) // Return error
            }
        }
    }

    func createAccount(username: String, password: String, completion: @escaping (Result<Int64, Error>) -> Void) {
        // Prepare the gRPC request
        let request = Auth_CreateAccountRequest.with {
            $0.username = username
            $0.password = password
        }

        // Set a timeout of 10 seconds
        let callOptions = CallOptions(timeLimit: .timeout(.seconds(5)))

        // Call the Login RPC with the timeout
        let call = client.createAccount(request, callOptions: callOptions)

        call.response.whenComplete { result in
            switch result {
            case .success(let reply):
                completion(.success(reply.userID)) // Return access token on success
            case .failure(let error):
                completion(.failure(error)) // Return error
            }
        }
    }
}
