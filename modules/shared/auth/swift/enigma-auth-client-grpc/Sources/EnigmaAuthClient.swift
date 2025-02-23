@available(iOS 13, *)
public protocol EnigmaAuthClient {
    func loadSession() async
    func validateSession() async throws -> UserInfo
    func listAccounts() async throws -> [EnigmaAccount]
    func login(username: String, password: String) async throws -> UserId
}

@available(iOS 13, *)
public class MockAuthClient: EnigmaAuthClient {

    public init() {}

    public func loadSession() async {
        // Ok
    }

    public func validateSession() async throws -> UserInfo{
        UserInfo(
            userId: UserId(value: 1),
            username: Username(
                value:"oof"
            ),
            roles: [Role(value: "user")]
        )
    }

    public func login(username: String, password: String) async throws -> UserId {
        UserId(value: 42)
    }

    public func listAccounts() async throws -> [EnigmaAccount] {
        [
            EnigmaAccount(accountId: UserId(value: 1), accountname: Username(value: "foo")),
            EnigmaAccount(accountId: UserId(value: 2), accountname: Username(value: "bar")),
            EnigmaAccount(accountId: UserId(value: 3), accountname: Username(value: "baz")),
            EnigmaAccount(accountId: UserId(value: 4), accountname: Username(value: "qux")),
            EnigmaAccount(accountId: UserId(value: 5), accountname: Username(value: "quux")),
            EnigmaAccount(accountId: UserId(value: 6), accountname: Username(value: "quuux")),
        ]
    }
}
