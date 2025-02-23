import Foundation
import SwiftData

import EnigmaProfiles

class MockModelContainer {
    static let shared: ModelContainer = {
        let schema = Schema([EnigmaPostEntity.self, EnigmaProfileEntity.self])
        let configuration = ModelConfiguration(schema: schema, isStoredInMemoryOnly: true)
        let container = try! ModelContainer(for: schema, configurations: [configuration])

        Task { @MainActor in
            let context = container.mainContext
            let sampleProfiles = [
                EnigmaProfileEntity(
                  userId: 1,
                  username: "SampleUser",
                  legalName: "Sample User",
                  displayName: "Sample",
                  birthDay: "42"
              ),
                EnigmaProfileEntity(
                  userId: 2,
                  username: "SampleUser2",
                  legalName: "Sample User2",
                  displayName: "Sample2",
                  birthDay: "24"
              ),
            ]

            for profile in sampleProfiles {
                context.insert(profile)
            }

            let samplePosts = [
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[0], timestamp: "Feb 11, 2025", content: "Hello, this is a test post."),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[0], timestamp: "Feb 10, 2025", content: "Another test message for the list."),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[1], timestamp: "Feb 9, 2025", content: "SwiftUI is awesome! 🚀"),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[0], timestamp: "Feb 8, 2025", content: "Just tried SwiftData, really cool."),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[1], timestamp: "Feb 7, 2025", content: "Coffee and coding ☕️💻"),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[0], timestamp: "Feb 6, 2025", content: "Is Swift 6 coming soon?"),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[1], timestamp: "Feb 5, 2025", content: "Finally finished my SwiftUI project!"),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[1], timestamp: "Feb 4, 2025", content: "Looking for tips on Combine framework."),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[1], timestamp: "Feb 3, 2025", content: "This community is amazing!"),
                EnigmaPostEntity(postId: UUID(), author: sampleProfiles[0], timestamp: "Feb 2, 2025", content: "Can't decide between UIKit and SwiftUI."),
            ]

            for post in samplePosts {
                context.insert(post)
            }

            try? context.save()
        }

        return container
    }()
}
