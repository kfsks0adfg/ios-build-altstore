import SwiftUI

struct ContentView: View {
    var body: some View {
        ZStack {
            LinearGradient(
                colors: [.indigo, .black],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .ignoresSafeArea()

            VStack(spacing: 18) {
                Image(systemName: "swift")
                    .font(.system(size: 64, weight: .semibold))
                Text("Hello Swift")
                    .font(.largeTitle.bold())
                Text("Compilé sans signature par GitHub Actions")
                    .multilineTextAlignment(.center)
                    .foregroundStyle(.secondary)
            }
            .padding(32)
        }
        .preferredColorScheme(.dark)
    }
}

#Preview {
    ContentView()
}

