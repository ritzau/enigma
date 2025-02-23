import SwiftUI
import SwiftData

import EnigmaAuth

struct ListItemView: View {
    var profileImage: String = "person.circle.fill"
    var username: String
    var date: String
    var content: String

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            Image(systemName: profileImage)
                .resizable()
                .scaledToFit()
                .frame(width: 50, height: 50)
                .clipShape(Circle())
                .foregroundColor(.gray)

            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(username)
                        .font(.headline)
                    Spacer()
                    Text(date)
                        .font(.caption)
                        .foregroundColor(.gray)
                }

                Text(content)
                    .font(.body)
                    .foregroundColor(.primary)
            }
        }
        .padding()
    }
}

#Preview {
    ListItemView(
        profileImage: "person.circle.fill",
        username: "foobar",
        date: "1971-03-17",
        content: "Lorem ipsum"
    )
}
