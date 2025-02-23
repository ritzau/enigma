import SwiftUI
import SwiftData

import EnigmaAuth

struct FeedView: View {
    @Environment(\.modelContext) private var modelContext
    @Query private var posts: [EnigmaPostEntity]

    @State private var isLoading = false

    var body: some View {
        NavigationView {
            List(posts) { post in
                ListItemView(
                    username: post.author?.displayName ?? "Unknown",
                    date: post.timestamp,
                    content: post.content
                )
            }
            .navigationTitle("Feed")
        }
    }
}



#Preview {
    FeedView()
        .modelContainer(MockModelContainer.shared)
}
