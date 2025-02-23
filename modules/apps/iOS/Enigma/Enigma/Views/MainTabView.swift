import SwiftUI

import EnigmaAuth
import EnigmaProfiles

struct MainTabView: View {
    var body: some View {
        TabView {
            FeedView()
                .tabItem {
                    Label("Feed", systemImage: "house")
                }

            ConnectionsView()
                .tabItem {
                    Label("Connections", systemImage: "person.2")
                }

            NotificationsView()
                .tabItem {
                    Label("Notifications", systemImage: "bell")
                }

            ProfileView()
                .tabItem {
                    Label("Profile", systemImage: "person")
                }
        }
    }
}

struct NotificationsView: View {
    var body: some View {
        Label("Notifications", systemImage: "house")
    }
}

#Preview {
    MainTabView()
        .sampleClients()
}
