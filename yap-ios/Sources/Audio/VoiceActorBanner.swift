import SwiftUI

struct VoiceActorBanner: View {
    @Environment(AudioPlayer.self) private var audio
    var body: some View {
        if let credit = audio.voiceCredit {
            Label(credit, systemImage: "mic.fill")
                .font(.footnote).padding(14)
                .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 16))
                .padding().allowsHitTesting(false)
                .accessibilityAddTraits(.updatesFrequently)
        }
    }
}
